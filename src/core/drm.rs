//! Pure-Rust Linux DRM KMS Backend with Atomic Modesetting
//! Directly controls the Apple Touch Bar panel (appletbdrm).

use anyhow::{anyhow, Result};
use drm::{
    buffer::DrmFourcc,
    control::{
        atomic, connector,
        dumbbuffer::DumbBuffer,
        framebuffer, property, AtomicCommitFlags, ClipRect, Device as ControlDevice, Mode,
        ResourceHandle,
    },
    ClientCapability, Device as DrmDevice,
};
use std::{
    fs::{File, OpenOptions},
    os::unix::io::{AsFd, BorrowedFd},
    path::Path,
};

struct Card(File);

impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl ControlDevice for Card {}
impl DrmDevice for Card {}

impl Card {
    fn open(path: &Path) -> Result<Self> {
        let mut options = OpenOptions::new();
        options.read(true);
        options.write(true);
        Ok(Card(options.open(path)?))
    }
}

pub struct DrmBackend {
    card: Card,
    #[allow(dead_code)]
    mode: Mode,
    db: DumbBuffer,
    fb: framebuffer::Handle,
    disp_width: u16,
    disp_height: u16,
}

impl Drop for DrmBackend {
    fn drop(&mut self) {
        let _ = self.card.destroy_framebuffer(self.fb);
        let _ = self.card.destroy_dumb_buffer(self.db);
    }
}

fn find_prop_id<T: ResourceHandle>(
    card: &Card,
    handle: T,
    name: &'static str,
) -> Result<property::Handle> {
    let props = card.get_properties(handle)?;
    for id in props.as_props_and_values().0 {
        let info = card.get_property(*id)?;
        if let Ok(prop_name) = info.name().to_str() {
            if prop_name == name {
                return Ok(*id);
            }
        }
    }
    Err(anyhow!("Property {name} not found"))
}

impl DrmBackend {
    pub fn open_touchbar(path: &Path) -> Result<Self> {
        let card = Card::open(path)?;
        card.set_client_capability(ClientCapability::UniversalPlanes, true)?;
        card.set_client_capability(ClientCapability::Atomic, true)?;
        let _ = card.acquire_master_lock();

        let res = card.resource_handles()?;
        let coninfo = res
            .connectors()
            .iter()
            .flat_map(|con| card.get_connector(*con, true))
            .collect::<Vec<_>>();
        let crtcinfo = res
            .crtcs()
            .iter()
            .flat_map(|crtc| card.get_crtc(*crtc))
            .collect::<Vec<_>>();

        let con = coninfo
            .iter()
            .find(|&i| i.state() == connector::State::Connected)
            .ok_or_else(|| anyhow!("No connected connectors found"))?;

        let &mode = con.modes().first().ok_or_else(|| anyhow!("No modes found"))?;
        let (disp_width, disp_height) = mode.size();

        let crtc = crtcinfo.first().ok_or_else(|| anyhow!("No crtcs found"))?;
        let fmt = DrmFourcc::Xrgb8888;

        // Hardware panel is 60x2008 in portrait; stride must be aligned to 64 pixels (256 bytes)
        let db = card.create_dumb_buffer((64, disp_height.into()), fmt, 32)?;
        let fb = card.add_framebuffer(&db, 24, 32)?;
        let plane = *card
            .plane_handles()?
            .first()
            .ok_or_else(|| anyhow!("No planes found"))?;

        let mut atomic_req = atomic::AtomicModeReq::new();
        atomic_req.add_property(
            con.handle(),
            find_prop_id(&card, con.handle(), "CRTC_ID")?,
            property::Value::CRTC(Some(crtc.handle())),
        );
        let blob = card.create_property_blob(&mode)?;

        atomic_req.add_property(
            crtc.handle(),
            find_prop_id(&card, crtc.handle(), "MODE_ID")?,
            blob,
        );
        atomic_req.add_property(
            crtc.handle(),
            find_prop_id(&card, crtc.handle(), "ACTIVE")?,
            property::Value::Boolean(true),
        );
        atomic_req.add_property(
            plane,
            find_prop_id(&card, plane, "FB_ID")?,
            property::Value::Framebuffer(Some(fb)),
        );
        atomic_req.add_property(
            plane,
            find_prop_id(&card, plane, "CRTC_ID")?,
            property::Value::CRTC(Some(crtc.handle())),
        );
        atomic_req.add_property(
            plane,
            find_prop_id(&card, plane, "SRC_X")?,
            property::Value::UnsignedRange(0),
        );
        atomic_req.add_property(
            plane,
            find_prop_id(&card, plane, "SRC_Y")?,
            property::Value::UnsignedRange(0),
        );
        atomic_req.add_property(
            plane,
            find_prop_id(&card, plane, "SRC_W")?,
            property::Value::UnsignedRange((disp_width as u64) << 16),
        );
        atomic_req.add_property(
            plane,
            find_prop_id(&card, plane, "SRC_H")?,
            property::Value::UnsignedRange((disp_height as u64) << 16),
        );
        atomic_req.add_property(
            plane,
            find_prop_id(&card, plane, "CRTC_X")?,
            property::Value::SignedRange(0),
        );
        atomic_req.add_property(
            plane,
            find_prop_id(&card, plane, "CRTC_Y")?,
            property::Value::SignedRange(0),
        );
        atomic_req.add_property(
            plane,
            find_prop_id(&card, plane, "CRTC_W")?,
            property::Value::UnsignedRange(disp_width as u64),
        );
        atomic_req.add_property(
            plane,
            find_prop_id(&card, plane, "CRTC_H")?,
            property::Value::UnsignedRange(disp_height as u64),
        );

        card.atomic_commit(AtomicCommitFlags::ALLOW_MODESET, atomic_req)?;

        Ok(Self {
            card,
            mode,
            db,
            fb,
            disp_width,
            disp_height,
        })
    }

    pub fn draw_pixels(&mut self, landscape_pixels: &[u32]) -> Result<()> {
        let mut mapping = self.card.map_dumb_buffer(&mut self.db)?;
        let hw_buf: &mut [u8] = mapping.as_mut();

        // Interpret hardware buffer as u32 slice
        let hw_u32: &mut [u32] = unsafe {
            std::slice::from_raw_parts_mut(
                hw_buf.as_mut_ptr() as *mut u32,
                hw_buf.len() / 4,
            )
        };

        let hw_stride = 64usize;
        let landscape_w = 2008usize;
        let landscape_h = 60usize;

        // Transpose landscape 2008x60 into hardware portrait buffer 64x2008
        // User x in 0..2008 maps to hw row x (or 2007 - x)
        // User y in 0..60 maps to hw col y
        for ly in 0..landscape_h {
            for lx in 0..landscape_w {
                let pixel = landscape_pixels[ly * landscape_w + lx];
                // In hardware orientation: row is (2007 - lx), col is ly
                let hw_row = 2007 - lx;
                let hw_col = ly;
                let idx = hw_row * hw_stride + hw_col;
                if idx < hw_u32.len() {
                    hw_u32[idx] = pixel;
                }
            }
        }

        // Notify DRM that the entire physical touchbar area is dirty
        let clip = ClipRect::new(0, 0, self.disp_width, self.disp_height);
        let _ = self.card.dirty_framebuffer(self.fb, &[clip]);

        Ok(())
    }
}

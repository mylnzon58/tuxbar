//! Pure-Rust Linux DRM Direct Rendering Manager KMS Backend
//! Manages dumb buffer allocation and page flipping for the Touch Bar panel

use anyhow::{anyhow, Result};
use drm::{
    buffer::DrmFourcc,
    control::{
        connector,
        dumbbuffer::DumbBuffer,
        framebuffer, Device as ControlDevice, Mode,
    },
    Device as DrmDevice,
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
    mode: Mode,
    db: DumbBuffer,
    fb: framebuffer::Handle,
    crtc: drm::control::crtc::Handle,
    connector: connector::Handle,
}

impl Drop for DrmBackend {
    fn drop(&mut self) {
        let _ = self.card.destroy_framebuffer(self.fb);
        let _ = self.card.destroy_dumb_buffer(self.db);
    }
}

impl DrmBackend {
    pub fn open_touchbar(path: &Path) -> Result<Self> {
        let card = Card::open(path)?;

        let res = card.resource_handles()?;
        let connector_handle = *res.connectors().first().ok_or_else(|| anyhow!("No connector found"))?;
        let con_info = card.get_connector(connector_handle, true)?;

        let mode = *con_info.modes().first().ok_or_else(|| anyhow!("No video modes found on Touch Bar"))?;
        let (width, height) = mode.size();

        let crtc_handle = *res.crtcs().first().ok_or_else(|| anyhow!("No CRTC found"))?;

        // Allocate dumb buffer (ARGB8888 32bpp)
        let db = card.create_dumb_buffer((width as u32, height as u32), DrmFourcc::Xrgb8888, 32)?;
        let fb = card.add_framebuffer(&db, 24, 32)?;

        let backend = Self {
            card,
            mode,
            db,
            fb,
            crtc: crtc_handle,
            connector: connector_handle,
        };

        // Attach CRTC to connector
        backend.card.set_crtc(
            backend.crtc,
            Some(backend.fb),
            (0, 0),
            &[backend.connector],
            Some(backend.mode),
        )?;

        Ok(backend)
    }

    pub fn draw_pixels(&mut self, pixels: &[u32]) -> Result<()> {
        let mut mapping = self.card.map_dumb_buffer(&mut self.db)?;
        let slice: &mut [u8] = mapping.as_mut();

        // Convert u32 ARGB pixels to u8 slice
        let byte_len = (pixels.len() * 4).min(slice.len());
        let src_bytes = unsafe {
            std::slice::from_raw_parts(pixels.as_ptr() as *const u8, byte_len)
        };
        slice[..byte_len].copy_from_slice(src_bytes);

        // Notify DRM that framebuffer changed
        let _ = self.card.dirty_framebuffer(self.fb, &[]);

        Ok(())
    }
}

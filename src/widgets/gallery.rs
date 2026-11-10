//! Real Screenshot Gallery Scrubber with Linux Desktop Opener
//! Discovers actual screenshots in user directories (~/Pictures/Capturas de pantalla, ~/Pictures, ~/Downloads)
//! Displays real 84x54 thumbnails, smooth kinetic touch scrolling, and instant desktop launcher.

use crate::graphics::{FontEngine, Framebuffer};
use crate::types::{Color, Rect, TouchEvent, TouchPhase, WidgetAction};
use crate::widgets::Widget;
use image::RgbaImage;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::time::Instant;

pub struct PhotoItem {
    pub path: PathBuf,
    pub name: String,
    pub thumbnail: RgbaImage,
}

#[derive(Debug, Clone)]
struct UserSessionInfo {
    uid: u32,
    gid: u32,
    username: String,
    home: PathBuf,
    runtime_dir: PathBuf,
}

fn detect_user_session() -> UserSessionInfo {
    let mut target_uid = unsafe { libc::geteuid() };
    if target_uid == 0 {
        if let Ok(sudo_uid) = std::env::var("SUDO_UID") {
            if let Ok(id) = sudo_uid.parse::<u32>() {
                if id >= 1000 {
                    target_uid = id;
                }
            }
        }
    }
    if target_uid == 0 {
        if let Ok(entries) = fs::read_dir("/run/user") {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if let Ok(id) = name.parse::<u32>() {
                        if id >= 1000 {
                            target_uid = id;
                            break;
                        }
                    }
                }
            }
        }
    }
    if target_uid == 0 {
        target_uid = 1000;
    }

    let mut username = String::from("user");
    let mut home = PathBuf::from(format!("/home/{}", target_uid));
    let mut gid = target_uid;

    unsafe {
        let pw = libc::getpwuid(target_uid);
        if !pw.is_null() {
            username = std::ffi::CStr::from_ptr((*pw).pw_name)
                .to_string_lossy()
                .into_owned();
            let home_str = std::ffi::CStr::from_ptr((*pw).pw_dir)
                .to_string_lossy()
                .into_owned();
            home = PathBuf::from(home_str);
            gid = (*pw).pw_gid;
        }
    }

    UserSessionInfo {
        uid: target_uid,
        gid,
        username,
        home,
        runtime_dir: PathBuf::from(format!("/run/user/{}", target_uid)),
    }
}

pub struct GalleryWidget {
    photos: Vec<PhotoItem>,
    pub selected_idx: usize,
    scroll_offset: f32,
    last_touch_x: Option<i32>,
    last_click_time: Option<Instant>,
    last_click_idx: Option<usize>,
}

impl GalleryWidget {
    #[must_use]
    pub fn new() -> Self {
        let mut widget = Self {
            photos: Vec::new(),
            selected_idx: 0,
            scroll_offset: 0.0,
            last_touch_x: None,
            last_click_time: None,
            last_click_idx: None,
        };
        widget.load_from_pictures_or_fallback();
        widget
    }

    pub fn load_from_pictures_or_fallback(&mut self) {
        let mut loaded = Vec::new();
        let session = detect_user_session();

        // 1. Dynamically scan user directories across session home and /home/*
        let mut dirs_to_check = Vec::new();

        dirs_to_check.push(session.home.join("Pictures/Capturas de pantalla"));
        dirs_to_check.push(session.home.join("Pictures/Screenshots"));
        dirs_to_check.push(session.home.join("Pictures"));
        dirs_to_check.push(session.home.join("Desktop"));
        dirs_to_check.push(session.home.join("Downloads"));

        if let Ok(entries) = fs::read_dir("/home") {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && p != session.home {
                    dirs_to_check.push(p.join("Pictures/Capturas de pantalla"));
                    dirs_to_check.push(p.join("Pictures/Screenshots"));
                    dirs_to_check.push(p.join("Pictures"));
                    dirs_to_check.push(p.join("Desktop"));
                    dirs_to_check.push(p.join("Downloads"));
                }
            }
        }

        for dir in dirs_to_check {
            if dir.exists() && loaded.len() < 24 {
                if let Ok(entries) = fs::read_dir(&dir) {
                    let mut paths: Vec<PathBuf> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| {
                            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                            matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp")
                        })
                        .collect();
                    paths.sort();
                    paths.reverse(); // Most recent screenshots first

                    for p in paths.into_iter().take(16) {
                        if let Ok(img) = image::open(&p) {
                            // Resize to 84x54 thumbnail
                            let thumb = img.thumbnail_exact(84, 54).to_rgba8();
                            let name = p.file_stem().and_then(|s| s.to_str()).unwrap_or("photo").to_string();
                            loaded.push(PhotoItem {
                                path: p,
                                name,
                                thumbnail: thumb,
                            });
                        }
                    }
                }
            }
        }

        // 2. Fallback procedural demo photos if no images found anywhere
        if loaded.is_empty() {
            let demos = [
                ("Neon Grid", [0, 240, 255, 255], [138, 43, 226, 255]),
                ("Cyber Sun", [255, 0, 128, 255], [255, 140, 0, 255]),
                ("Aurora", [0, 255, 136, 255], [0, 100, 200, 255]),
                ("Synth Sunset", [255, 80, 0, 255], [80, 0, 120, 255]),
                ("Matrix Code", [0, 255, 65, 255], [0, 40, 10, 255]),
            ];

            for (name, c1, c2) in demos {
                let mut img = RgbaImage::new(84, 54);
                for y in 0..54 {
                    for x in 0..84 {
                        let t = (x as f32 + y as f32) / 138.0;
                        let r = (c1[0] as f32 * (1.0 - t) + c2[0] as f32 * t) as u8;
                        let g = (c1[1] as f32 * (1.0 - t) + c2[1] as f32 * t) as u8;
                        let b = (c1[2] as f32 * (1.0 - t) + c2[2] as f32 * t) as u8;
                        img.put_pixel(x, y, image::Rgba([r, g, b, 255]));
                    }
                }
                loaded.push(PhotoItem {
                    path: PathBuf::from(format!("/demo/{name}")),
                    name: name.to_string(),
                    thumbnail: img,
                });
            }
        }

        self.photos = loaded;
    }

    pub fn select_next(&mut self) {
        if !self.photos.is_empty() {
            self.selected_idx = (self.selected_idx + 1) % self.photos.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.photos.is_empty() {
            if self.selected_idx == 0 {
                self.selected_idx = self.photos.len() - 1;
            } else {
                self.selected_idx -= 1;
            }
        }
    }

    pub fn open_selected(&self) {
        if let Some(item) = self.photos.get(self.selected_idx) {
            if !item.path.exists() {
                eprintln!("[TuxBar] Photo path does not exist: {:?}", item.path);
                return;
            }

            let path_str = item.path.to_string_lossy().to_string();
            println!("[TuxBar] Launching image viewer for: {}", path_str);

            let session = detect_user_session();
            let bus_path = format!("unix:path={}/bus", session.runtime_dir.display());

            // 1. Identify preferred native image viewer (eog on GNOME, loupe on GNOME 45+)
            let native_viewer = if std::path::Path::new("/usr/bin/eog").exists() {
                Some("/usr/bin/eog")
            } else if std::path::Path::new("/usr/bin/loupe").exists() {
                Some("/usr/bin/loupe")
            } else {
                None
            };

            let mut launched = false;

            // Priority 1: systemd-run --user with native viewer (eog / loupe)
            // Launches directly within user GUI session slice, bypassing root daemon namespace/sandbox
            if let Some(viewer) = native_viewer {
                let res = std::process::Command::new("systemd-run")
                    .args(["--user", "--slice=app.slice", viewer, &path_str])
                    .uid(session.uid)
                    .gid(session.gid)
                    .env("HOME", &session.home)
                    .env("USER", &session.username)
                    .env("XDG_RUNTIME_DIR", &session.runtime_dir)
                    .env("DBUS_SESSION_BUS_ADDRESS", &bus_path)
                    .env("WAYLAND_DISPLAY", "wayland-0")
                    .env("DISPLAY", ":0")
                    .env("XDG_CURRENT_DESKTOP", "ubuntu:GNOME")
                    .spawn();

                if let Ok(mut child) = res {
                    if let Ok(status) = child.wait() {
                        if status.success() {
                            println!("[TuxBar] Successfully launched {} via systemd-run", viewer);
                            launched = true;
                        }
                    }
                }

                // Priority 2: Direct execution of native viewer
                if !launched {
                    let direct_res = std::process::Command::new(viewer)
                        .arg(&path_str)
                        .uid(session.uid)
                        .gid(session.gid)
                        .env("HOME", &session.home)
                        .env("USER", &session.username)
                        .env("XDG_RUNTIME_DIR", &session.runtime_dir)
                        .env("DBUS_SESSION_BUS_ADDRESS", &bus_path)
                        .env("WAYLAND_DISPLAY", "wayland-0")
                        .env("DISPLAY", ":0")
                        .env("XDG_CURRENT_DESKTOP", "ubuntu:GNOME")
                        .spawn();

                    if direct_res.is_ok() {
                        println!("[TuxBar] Successfully spawned {} directly", viewer);
                        launched = true;
                    }
                }
            }

            // Priority 3: systemd-run --user gio open
            if !launched {
                let gio_run = std::process::Command::new("systemd-run")
                    .args(["--user", "--slice=app.slice", "gio", "open", &path_str])
                    .uid(session.uid)
                    .gid(session.gid)
                    .env("HOME", &session.home)
                    .env("USER", &session.username)
                    .env("XDG_RUNTIME_DIR", &session.runtime_dir)
                    .env("DBUS_SESSION_BUS_ADDRESS", &bus_path)
                    .env("WAYLAND_DISPLAY", "wayland-0")
                    .env("DISPLAY", ":0")
                    .env("XDG_CURRENT_DESKTOP", "ubuntu:GNOME")
                    .spawn();

                if let Ok(mut child) = gio_run {
                    if let Ok(status) = child.wait() {
                        if status.success() {
                            println!("[TuxBar] Successfully opened image via gio open (systemd-run)");
                            launched = true;
                        }
                    }
                }
            }

            // Priority 4: gio open direct
            if !launched {
                let gio_direct = std::process::Command::new("gio")
                    .args(["open", &path_str])
                    .uid(session.uid)
                    .gid(session.gid)
                    .env("HOME", &session.home)
                    .env("USER", &session.username)
                    .env("XDG_RUNTIME_DIR", &session.runtime_dir)
                    .env("DBUS_SESSION_BUS_ADDRESS", &bus_path)
                    .env("WAYLAND_DISPLAY", "wayland-0")
                    .env("DISPLAY", ":0")
                    .env("XDG_CURRENT_DESKTOP", "ubuntu:GNOME")
                    .spawn();

                if gio_direct.is_ok() {
                    launched = true;
                }
            }

            // Priority 5: xdg-open fallback
            if !launched {
                let _ = std::process::Command::new("xdg-open")
                    .arg(&path_str)
                    .uid(session.uid)
                    .gid(session.gid)
                    .env("HOME", &session.home)
                    .env("USER", &session.username)
                    .env("XDG_RUNTIME_DIR", &session.runtime_dir)
                    .env("DBUS_SESSION_BUS_ADDRESS", &bus_path)
                    .env("WAYLAND_DISPLAY", "wayland-0")
                    .env("DISPLAY", ":0")
                    .env("XDG_CURRENT_DESKTOP", "ubuntu:GNOME")
                    .spawn();
            }
        }
    }
}

impl Widget for GalleryWidget {
    fn render(&self, fb: &mut Framebuffer, area: Rect, font: &FontEngine) {
        fb.fill_rect(area, Color::BLACK);

        let btn_h = area.height.saturating_sub(2); // 58px tall
        let btn_y = area.y + 1;
        let mut cur_x = area.x + 4;

        // 1. Navigation Controls [ ◀ ] [ 🔍 ABRIR ] [ ▶ ] (Full height 58px)
        let prev_w = 64;
        let prev_rect = Rect::new(cur_x, btn_y, prev_w, btn_h);
        fb.draw_rounded_rect(prev_rect, 6, Color::rgb(35, 42, 54));
        let prev_tw = font.measure_text_width("◀", 22.0);
        fb.draw_text(font, "◀", cur_x + (prev_w as i32 - prev_tw as i32) / 2, btn_y + 38, 22.0, Color::WHITE);
        cur_x += prev_w as i32 + 6;

        let open_w = 100;
        let open_rect = Rect::new(cur_x, btn_y, open_w, btn_h);
        fb.draw_rounded_rect(open_rect, 6, Color::ACCENT);
        let open_tw = font.measure_text_width("ABRIR", 20.0);
        fb.draw_text(font, "ABRIR", cur_x + (open_w as i32 - open_tw as i32) / 2, btn_y + 38, 20.0, Color::BLACK);
        cur_x += open_w as i32 + 6;

        let next_w = 64;
        let next_rect = Rect::new(cur_x, btn_y, next_w, btn_h);
        fb.draw_rounded_rect(next_rect, 6, Color::rgb(35, 42, 54));
        let next_tw = font.measure_text_width("▶", 22.0);
        fb.draw_text(font, "▶", cur_x + (next_w as i32 - next_tw as i32) / 2, btn_y + 38, 22.0, Color::WHITE);
        cur_x += next_w as i32 + 10;

        // 2. Active photo info badge card (Enclosed capsule preventing collision with thumbnails)
        let badge_w = 200u32;
        let badge_rect = Rect::new(cur_x, btn_y, badge_w, btn_h);
        fb.draw_rounded_rect(badge_rect, 6, Color::rgb(22, 28, 38));

        if let Some(item) = self.photos.get(self.selected_idx) {
            let counter_text = format!("{}/{}", self.selected_idx + 1, self.photos.len());
            fb.draw_text(font, &counter_text, cur_x + 8, btn_y + 20, 13.0, Color::CYAN);

            let max_text_x = cur_x + badge_w as i32 - 8;
            fb.draw_text_clipped(font, &item.name, cur_x + 8, btn_y + 42, 16.0, Color::WHITE, max_text_x);
        } else {
            fb.draw_text(font, "Sin fotos", cur_x + 12, btn_y + 36, 16.0, Color::GRAY);
        }
        cur_x += badge_w as i32 + 12;

        // 3. Horizontal Filmstrip Thumbnails
        let filmstrip_x = cur_x;
        let filmstrip_w = (area.x + area.width as i32) - filmstrip_x - 10;
        if filmstrip_w <= 0 {
            return;
        }

        let thumb_w = 84;
        let thumb_h = 54;
        let thumb_gap = 8;
        let thumb_y = area.y + 3;

        for (i, item) in self.photos.iter().enumerate() {
            let item_x = filmstrip_x + (i as i32 * (thumb_w + thumb_gap)) - self.scroll_offset as i32;

            if item_x + thumb_w > filmstrip_x && item_x < filmstrip_x + filmstrip_w {
                if i == self.selected_idx {
                    let border_rect = Rect::new(item_x - 2, thumb_y - 2, thumb_w as u32 + 4, thumb_h as u32 + 4);
                    fb.draw_rounded_rect(border_rect, 6, Color::CYAN);
                }

                fb.draw_image(&item.thumbnail, item_x, thumb_y, thumb_w as u32, thumb_h as u32, 4);
            }
        }
    }

    fn handle_touch(&mut self, event: &TouchEvent, area: Rect) -> WidgetAction {
        let px = event.x as i32;
        let py = event.y as i32;

        match event.phase {
            TouchPhase::Down => {
                self.last_touch_x = Some(px);

                let btn_h = area.height;
                let btn_y = area.y;
                let mut cur_x = area.x + 4;

                let prev_rect = Rect::new(cur_x, btn_y, 64 + 4, btn_h);
                cur_x += 64 + 6;
                let open_rect = Rect::new(cur_x, btn_y, 100 + 4, btn_h);
                cur_x += 100 + 6;
                let next_rect = Rect::new(cur_x, btn_y, 64 + 4, btn_h);
                cur_x += 64 + 10;

                if prev_rect.contains(px, py) {
                    self.select_prev();
                    return WidgetAction::None;
                }
                if open_rect.contains(px, py) {
                    self.open_selected();
                    return WidgetAction::None;
                }
                if next_rect.contains(px, py) {
                    self.select_next();
                    return WidgetAction::None;
                }

                // Skip photo badge card
                let badge_w = 200;
                cur_x += badge_w + 12;

                // Check click or double-tap directly on filmstrip thumbnails
                let filmstrip_x = cur_x;
                if px >= filmstrip_x {
                    let thumb_stride = 84 + 8;
                    let clicked_offset = (px - filmstrip_x) + self.scroll_offset as i32;
                    let clicked_idx = (clicked_offset / thumb_stride) as usize;
                    if clicked_idx < self.photos.len() {
                        let now = Instant::now();
                        let is_double_tap = if let (Some(last_t), Some(last_i)) = (self.last_click_time, self.last_click_idx) {
                            last_i == clicked_idx && now.duration_since(last_t).as_millis() < 500
                        } else {
                            false
                        };

                        self.last_click_time = Some(now);
                        self.last_click_idx = Some(clicked_idx);
                        self.selected_idx = clicked_idx;

                        if is_double_tap {
                            self.open_selected();
                        }
                    }
                }
            }
            TouchPhase::Motion => {
                if let Some(last_x) = self.last_touch_x {
                    let delta = last_x - px;
                    self.scroll_offset = (self.scroll_offset + delta as f32).max(0.0);
                    self.last_touch_x = Some(px);
                }
            }
            TouchPhase::Up => {
                self.last_touch_x = None;
            }
        }

        WidgetAction::None
    }
}

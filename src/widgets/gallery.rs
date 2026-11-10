//! Touch Bar Image Viewer & Filmstrip Scrubber Widget
//! Re-creates the iconic macOS Touch Bar Photo Scrubber in 100% pure Rust for Linux.
//! Ergonomically optimized to take advantage of the full 60px height of the Touch Bar.

use std::fs;
use std::path::PathBuf;
use image::RgbaImage;

use crate::graphics::{FontEngine, Framebuffer};
use crate::types::{Color, Rect, TouchEvent, TouchPhase, WidgetAction};
use crate::widgets::Widget;

pub struct PhotoItem {
    pub path: PathBuf,
    pub name: String,
    pub thumbnail: RgbaImage,
}

pub struct GalleryWidget {
    photos: Vec<PhotoItem>,
    pub selected_idx: usize,
    scroll_offset: f32,
    last_touch_x: Option<i32>,
}

impl GalleryWidget {
    #[must_use]
    pub fn new() -> Self {
        let mut widget = Self {
            photos: Vec::new(),
            selected_idx: 0,
            scroll_offset: 0.0,
            last_touch_x: None,
        };
        widget.load_from_pictures_or_fallback();
        widget
    }

    pub fn load_from_pictures_or_fallback(&mut self) {
        let mut loaded = Vec::new();

        // 1. Scan user directories (/home/*/Pictures/Capturas de pantalla, etc.)
        let mut dirs_to_check = Vec::new();

        if let Ok(entries) = fs::read_dir("/home") {
            for entry in entries.flatten() {
                let user_dir = entry.path();
                if user_dir.is_dir() {
                    dirs_to_check.push(user_dir.join("Pictures/Capturas de pantalla"));
                    dirs_to_check.push(user_dir.join("Pictures"));
                    dirs_to_check.push(user_dir.join("Downloads"));
                    dirs_to_check.push(user_dir.join("Desktop"));
                }
            }
        }

        if let Ok(home) = std::env::var("HOME") {
            let h = PathBuf::from(home);
            dirs_to_check.push(h.join("Pictures/Capturas de pantalla"));
            dirs_to_check.push(h.join("Pictures"));
            dirs_to_check.push(h.join("Downloads"));
        }

        for dir in dirs_to_check {
            if dir.exists() && loaded.len() < 16 {
                if let Ok(entries) = fs::read_dir(&dir) {
                    let mut paths: Vec<PathBuf> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| {
                            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                            ext == "png" || ext == "jpg" || ext == "jpeg"
                        })
                        .collect();
                    paths.sort();
                    paths.reverse(); // Most recent screenshots first

                    for p in paths.into_iter().take(12) {
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
            if item.path.exists() {
                let path_str = item.path.to_string_lossy();
                let user = std::env::var("SUDO_USER").unwrap_or_else(|_| "zeeroj".to_string());
                let _ = std::process::Command::new("su")
                    .args(["-", &user, "-c", &format!("DISPLAY=:0 DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus xdg-open \"{}\"", path_str)])
                    .spawn();
                let _ = std::process::Command::new("xdg-open")
                    .arg(&item.path)
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

        let open_w = 110;
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
        cur_x += next_w as i32 + 12;

        // 2. Active photo info badge
        if let Some(item) = self.photos.get(self.selected_idx) {
            let info_str = format!("{}/{} · {}", self.selected_idx + 1, self.photos.len(), item.name);
            let display_name = if info_str.len() > 18 {
                format!("{}...", &info_str[..16])
            } else {
                info_str
            };
            fb.draw_text(font, &display_name, cur_x, btn_y + 38, 18.0, Color::WHITE);
            cur_x += 160;
        }

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
                let open_rect = Rect::new(cur_x, btn_y, 110 + 4, btn_h);
                cur_x += 110 + 6;
                let next_rect = Rect::new(cur_x, btn_y, 64 + 4, btn_h);
                cur_x += 64 + 12;

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

                // Check click directly on filmstrip thumbnails
                let filmstrip_x = cur_x + 160;
                if px >= filmstrip_x {
                    let thumb_stride = 84 + 8;
                    let clicked_offset = (px - filmstrip_x) + self.scroll_offset as i32;
                    let clicked_idx = (clicked_offset / thumb_stride) as usize;
                    if clicked_idx < self.photos.len() {
                        if clicked_idx == self.selected_idx {
                            self.open_selected();
                        } else {
                            self.selected_idx = clicked_idx;
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

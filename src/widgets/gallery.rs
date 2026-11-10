//! Touch Bar Image Viewer & Filmstrip Scrubber Widget
//! Re-creates the iconic macOS Touch Bar Photo Scrubber in 100% pure Rust for Linux.

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

        // 1. Scan ~/Pictures/Capturas de pantalla or ~/Pictures
        let home = std::env::var("HOME").ok().map(PathBuf::from);
        let dirs_to_check = [
            home.as_ref().map(|h| h.join("Pictures/Capturas de pantalla")),
            home.as_ref().map(|h| h.join("Pictures")),
            home.as_ref().map(|h| h.join("Downloads")),
        ];

        for dir_opt in dirs_to_check.into_iter().flatten() {
            if dir_opt.exists() && loaded.len() < 16 {
                if let Ok(entries) = fs::read_dir(&dir_opt) {
                    let mut paths: Vec<PathBuf> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| {
                            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                            ext == "png" || ext == "jpg" || ext == "jpeg"
                        })
                        .collect();
                    paths.sort();
                    paths.reverse(); // Most recent first

                    for path in paths.into_iter().take(16 - loaded.len()) {
                        if let Ok(img) = image::open(&path) {
                            let thumb = img.resize_exact(76, 48, image::imageops::FilterType::Triangle).to_rgba8();
                            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("photo").to_string();
                            loaded.push(PhotoItem { path, name, thumbnail: thumb });
                        }
                    }
                }
            }
        }

        // 2. Procedural Fallback if no images found
        if loaded.is_empty() {
            loaded = Self::generate_procedural_gallery();
        }

        self.photos = loaded;
    }

    fn generate_procedural_gallery() -> Vec<PhotoItem> {
        let mut items = Vec::new();
        let palettes = [
            ("Cyberpunk Neon", [0, 240, 255], [217, 70, 239]),
            ("Sunset Horizon", [249, 115, 22], [244, 63, 94]),
            ("Emerald Matrix", [16, 185, 129], [6, 182, 212]),
            ("Deep Cosmos", [99, 102, 241], [168, 85, 247]),
            ("Solar Flare", [234, 179, 8], [239, 68, 68]),
            ("Nordic Glacier", [56, 189, 248], [30, 41, 59]),
        ];

        for (name, c1, c2) in palettes {
            let mut img = RgbaImage::new(76, 48);
            for y in 0..48 {
                for x in 0..76 {
                    let t = (x as f32 / 76.0 + y as f32 / 48.0) / 2.0;
                    let r = ((1.0 - t) * c1[0] as f32 + t * c2[0] as f32) as u8;
                    let g = ((1.0 - t) * c1[1] as f32 + t * c2[1] as f32) as u8;
                    let b = ((1.0 - t) * c1[2] as f32 + t * c2[2] as f32) as u8;
                    img.put_pixel(x, y, image::Rgba([r, g, b, 255]));
                }
            }
            items.push(PhotoItem {
                path: PathBuf::from(name),
                name: name.to_string(),
                thumbnail: img,
            });
        }
        items
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
                let _ = std::process::Command::new("xdg-open")
                    .arg(&item.path)
                    .spawn();
            }
        }
    }
}

impl Widget for GalleryWidget {
    fn render(&self, fb: &mut Framebuffer, area: Rect, font: &FontEngine) {
        fb.draw_rounded_rect(area, 6, Color::CARD_BG);

        let mut cur_x = area.x + 10;

        // 1. Header Badge
        let badge_text = "📷 GALLERY";
        let badge_w = font.measure_text_width(badge_text, 11.0) + 12;
        let badge_rect = Rect::new(cur_x, area.y + 16, badge_w, 24);
        fb.draw_rounded_rect(badge_rect, 4, Color::DARK_GRAY);
        fb.draw_text(font, badge_text, cur_x + 6, area.y + 32, 11.0, Color::CYAN);
        cur_x += badge_w as i32 + 14;

        // 2. Active photo info badge
        if let Some(item) = self.photos.get(self.selected_idx) {
            let info_str = format!("{}/{} · {}", self.selected_idx + 1, self.photos.len(), item.name);
            let display_name = if info_str.len() > 24 {
                format!("{}...", &info_str[..22])
            } else {
                info_str
            };
            fb.draw_text(font, &display_name, cur_x, area.y + 32, 12.0, Color::WHITE);
            cur_x += 180;
        }

        // 3. Navigation Controls [ ◀ ] [ 🔍 OPEN ] [ ▶ ]
        let btn_y = area.y + 12;
        let btn_h = 32;

        let prev_rect = Rect::new(cur_x, btn_y, 36, btn_h);
        fb.draw_rounded_rect(prev_rect, 4, Color::DARK_GRAY);
        fb.draw_text(font, "◀", cur_x + 12, btn_y + 21, 13.0, Color::WHITE);
        cur_x += 42;

        let open_rect = Rect::new(cur_x, btn_y, 64, btn_h);
        fb.draw_rounded_rect(open_rect, 4, Color::ACCENT);
        fb.draw_text(font, "OPEN", cur_x + 12, btn_y + 21, 12.0, Color::WHITE);
        cur_x += 70;

        let next_rect = Rect::new(cur_x, btn_y, 36, btn_h);
        fb.draw_rounded_rect(next_rect, 4, Color::DARK_GRAY);
        fb.draw_text(font, "▶", cur_x + 12, btn_y + 21, 13.0, Color::WHITE);
        cur_x += 50;

        // 4. Horizontal Filmstrip Thumbnails
        let filmstrip_x = cur_x;
        let filmstrip_w = (area.x + area.width as i32) - filmstrip_x - 10;
        if filmstrip_w <= 0 {
            return;
        }

        let thumb_w = 76;
        let thumb_h = 48;
        let thumb_gap = 8;
        let thumb_y = area.y + 6;

        for (i, item) in self.photos.iter().enumerate() {
            let item_x = filmstrip_x + (i as i32 * (thumb_w + thumb_gap)) - self.scroll_offset as i32;

            // Only render if visible within filmstrip area
            if item_x + thumb_w > filmstrip_x && item_x < filmstrip_x + filmstrip_w {
                // If selected, draw glowing highlight frame
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

                // Check button hits: Prev, Open, Next
                let cur_x = area.x + 10 + 90 + 180;
                let prev_rect = Rect::new(cur_x, area.y + 12, 36, 32);
                let open_rect = Rect::new(cur_x + 42, area.y + 12, 64, 32);
                let next_rect = Rect::new(cur_x + 112, area.y + 12, 36, 32);

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
                let filmstrip_x = cur_x + 162;
                if px >= filmstrip_x {
                    let thumb_stride = 76 + 8;
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

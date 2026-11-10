//! High-DPI Now Playing Widget with Full-Height Media Controls
//! Maximizes touch usability across the full 60px height of the Apple Touch Bar.

use crate::graphics::{FontEngine, Framebuffer};
use crate::services::mpris::TrackInfo;
use crate::types::{Color, Rect, TouchEvent, TouchPhase, WidgetAction};
use crate::widgets::Widget;

pub struct NowPlayingWidget {
    pub info: TrackInfo,
    scroll_offset: f32,
}

impl NowPlayingWidget {
    #[must_use]
    pub fn new() -> Self {
        Self {
            info: TrackInfo::default(),
            scroll_offset: 0.0,
        }
    }

    pub fn update_info(&mut self, info: TrackInfo) {
        if self.info.title != info.title {
            self.scroll_offset = 0.0;
        }
        self.info = info;
    }

    pub fn tick_scroll(&mut self) {
        self.scroll_offset += 2.0;
    }
}

impl Widget for NowPlayingWidget {
    fn render(&self, fb: &mut Framebuffer, area: Rect, font: &FontEngine) {
        fb.fill_rect(area, Color::BLACK);

        let btn_h = area.height.saturating_sub(2); // 58px tall on 60px bar
        let btn_y = area.y + 1;
        let mut cur_x = area.x + 2;

        // 1. Esc Key: Large prominent target (96x58)
        let esc_w = 96;
        let esc_rect = Rect::new(cur_x, btn_y, esc_w, btn_h);
        fb.draw_rounded_rect(esc_rect, 6, Color::rgb(45, 55, 72));
        let esc_text_w = font.measure_text_width("Esc", 22.0);
        let esc_text_x = cur_x + (esc_w as i32 - esc_text_w as i32) / 2;
        fb.draw_text(font, "Esc", esc_text_x, btn_y + 38, 22.0, Color::CYAN);
        cur_x += esc_w as i32 + 6;

        // 2. Playback buttons: [ ◀◀ ] [ ▶ / ❚❚ ] [ ▶▶ ] (Large 76px - 88px touch targets)
        let prev_w = 76;
        let prev_rect = Rect::new(cur_x, btn_y, prev_w, btn_h);
        fb.draw_rounded_rect(prev_rect, 6, Color::rgb(35, 42, 54));
        let prev_icon_w = font.measure_text_width("◀◀", 22.0);
        let prev_icon_x = cur_x + (prev_w as i32 - prev_icon_w as i32) / 2;
        fb.draw_text(font, "◀◀", prev_icon_x, btn_y + 38, 22.0, Color::WHITE);
        cur_x += prev_w as i32 + 6;

        let play_w = 88;
        let play_rect = Rect::new(cur_x, btn_y, play_w, btn_h);
        fb.draw_rounded_rect(play_rect, 6, Color::ACCENT);
        let play_icon = if self.info.is_playing { "❚❚" } else { "▶" };
        let play_icon_w = font.measure_text_width(play_icon, 26.0);
        let play_icon_x = cur_x + (play_w as i32 - play_icon_w as i32) / 2;
        fb.draw_text(font, play_icon, play_icon_x, btn_y + 39, 26.0, Color::BLACK);
        cur_x += play_w as i32 + 6;

        let next_w = 76;
        let next_rect = Rect::new(cur_x, btn_y, next_w, btn_h);
        fb.draw_rounded_rect(next_rect, 6, Color::rgb(35, 42, 54));
        let next_icon_w = font.measure_text_width("▶▶", 22.0);
        let next_icon_x = cur_x + (next_w as i32 - next_icon_w as i32) / 2;
        fb.draw_text(font, "▶▶", next_icon_x, btn_y + 38, 22.0, Color::WHITE);
        cur_x += next_w as i32 + 12;

        // 3. Track Title & Artist
        let max_x = area.x + area.width as i32 - 10;
        let remaining_w = max_x - cur_x;
        if remaining_w > 50 {
            if !self.info.title.is_empty() {
                let track_text = if self.info.artist.is_empty() {
                    self.info.title.clone()
                } else {
                    format!("{} — {}", self.info.title, self.info.artist)
                };

                let total_text_w = font.measure_text_width(&track_text, 22.0);
                let draw_x = if total_text_w > remaining_w as u32 {
                    let max_scroll = (total_text_w as f32 - remaining_w as f32) + 40.0;
                    let offset = (self.scroll_offset % (max_scroll + 60.0)) - 30.0;
                    cur_x - offset.clamp(0.0, max_scroll) as i32
                } else {
                    cur_x
                };

                fb.draw_text_clipped(font, &track_text, draw_x, btn_y + 34, 22.0, Color::WHITE, max_x);

                // Progress bar at bottom edge
                if self.info.length_secs > 0.0 {
                    let ratio = self.info.position_secs / self.info.length_secs;
                    let bar_rect = Rect::new(cur_x, btn_y + 48, remaining_w as u32, 5);
                    fb.draw_progress_bar(bar_rect, ratio, Color::DARK_GRAY, Color::CYAN);
                }
            } else {
                // Idle state: Clean, prominent prompt without missing emoji glyphs
                let idle_text = "Toca ▶ para reproducir";
                fb.draw_text_clipped(font, idle_text, cur_x + 6, btn_y + 38, 20.0, Color::GRAY, max_x);
            }
        }
    }

    fn handle_touch(&mut self, event: &TouchEvent, area: Rect) -> WidgetAction {
        if event.phase != TouchPhase::Down {
            return WidgetAction::None;
        }

        let btn_h = area.height;
        let btn_y = area.y;
        let mut cur_x = area.x + 2;

        // Esc key hit
        let esc_rect = Rect::new(cur_x, btn_y, 96, btn_h);
        if esc_rect.contains(event.x as i32, event.y as i32) {
            return WidgetAction::Escape;
        }
        cur_x += 96 + 6;

        // Prev key hit
        let prev_rect = Rect::new(cur_x, btn_y, 76, btn_h);
        if prev_rect.contains(event.x as i32, event.y as i32) {
            return WidgetAction::PreviousTrack;
        }
        cur_x += 76 + 6;

        // Play key hit
        let play_rect = Rect::new(cur_x, btn_y, 88, btn_h);
        if play_rect.contains(event.x as i32, event.y as i32) {
            return WidgetAction::PlayPause;
        }
        cur_x += 88 + 6;

        // Next key hit
        let next_rect = Rect::new(cur_x, btn_y, 76, btn_h);
        if next_rect.contains(event.x as i32, event.y as i32) {
            return WidgetAction::NextTrack;
        }

        WidgetAction::None
    }
}

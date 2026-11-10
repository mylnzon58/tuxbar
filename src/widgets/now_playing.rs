//! Now Playing media widget displaying active MPRIS track and interactive touch controls

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
        self.scroll_offset += 1.5;
    }
}

impl Widget for NowPlayingWidget {
    fn render(&self, fb: &mut Framebuffer, area: Rect, font: &FontEngine) {
        // Draw card background
        fb.draw_rounded_rect(area, 6, Color::CARD_BG);

        let padding = 12;
        let mut cur_x = area.x + padding;

        // 1. Draw Player badge (e.g. "SPOTIFY")
        if !self.info.player_name.is_empty() {
            let badge = self.info.player_name.to_uppercase();
            let badge_w = font.measure_text_width(&badge, 12.0) + 12;
            let badge_rect = Rect::new(cur_x, area.y + 10, badge_w, 20);
            fb.draw_rounded_rect(badge_rect, 4, Color::DARK_GRAY);
            fb.draw_text(font, &badge, cur_x + 6, area.y + 25, 12.0, Color::CYAN);
            cur_x += badge_w as i32 + 10;
        }

        // 2. Playback buttons: [ ◀◀ ] [ ▶ / ❚❚ ] [ ▶▶ ]
        let btn_w = 40;
        let btn_h = 36;
        let btn_y = area.y + 12;

        let prev_rect = Rect::new(cur_x, btn_y, btn_w, btn_h);
        fb.draw_rounded_rect(prev_rect, 6, Color::DARK_GRAY);
        fb.draw_text(font, "◀◀", cur_x + 8, btn_y + 24, 14.0, Color::WHITE);
        cur_x += btn_w as i32 + 8;

        let play_rect = Rect::new(cur_x, btn_y, btn_w, btn_h);
        fb.draw_rounded_rect(play_rect, 6, Color::ACCENT);
        let play_icon = if self.info.is_playing { "❚❚" } else { "▶" };
        let icon_x_offset = if self.info.is_playing { 12 } else { 14 };
        fb.draw_text(font, play_icon, cur_x + icon_x_offset, btn_y + 24, 14.0, Color::WHITE);
        cur_x += btn_w as i32 + 8;

        let next_rect = Rect::new(cur_x, btn_y, btn_w, btn_h);
        fb.draw_rounded_rect(next_rect, 6, Color::DARK_GRAY);
        fb.draw_text(font, "▶▶", cur_x + 8, btn_y + 24, 14.0, Color::WHITE);
        cur_x += btn_w as i32 + 16;

        // 3. Track Title & Artist
        let track_text = if self.info.artist.is_empty() {
            self.info.title.clone()
        } else {
            format!("{} — {}", self.info.title, self.info.artist)
        };

        let text_display_w = area.width.saturating_sub((cur_x - area.x) as u32 + padding as u32);
        if text_display_w > 50 {
            let total_text_w = font.measure_text_width(&track_text, 18.0);
            let draw_x = if total_text_w > text_display_w {
                // Marquee scrolling
                let max_scroll = (total_text_w - text_display_w) as f32 + 40.0;
                let offset = (self.scroll_offset % (max_scroll + 60.0)) - 30.0;
                cur_x - offset.clamp(0.0, max_scroll) as i32
            } else {
                cur_x
            };

            fb.draw_text(font, &track_text, draw_x, area.y + 28, 18.0, Color::WHITE);

            // 4. Progress bar at bottom
            if self.info.length_secs > 0.0 {
                let ratio = self.info.position_secs / self.info.length_secs;
                let bar_rect = Rect::new(cur_x, area.y + 42, text_display_w, 6);
                fb.draw_progress_bar(bar_rect, ratio, Color::DARK_GRAY, Color::CYAN);
            }
        }
    }

    fn handle_touch(&mut self, event: &TouchEvent, area: Rect) -> WidgetAction {
        if event.phase != TouchPhase::Down {
            return WidgetAction::None;
        }

        let padding = 12;
        let mut cur_x = area.x + padding;
        if !self.info.player_name.is_empty() {
            let badge_w = 80;
            cur_x += badge_w + 10;
        }

        let btn_w = 40;
        let prev_rect = Rect::new(cur_x, area.y, btn_w, area.height);
        if prev_rect.contains(event.x as i32, event.y as i32) {
            return WidgetAction::PreviousTrack;
        }
        cur_x += btn_w as i32 + 8;

        let play_rect = Rect::new(cur_x, area.y, btn_w, area.height);
        if play_rect.contains(event.x as i32, event.y as i32) {
            return WidgetAction::PlayPause;
        }
        cur_x += btn_w as i32 + 8;

        let next_rect = Rect::new(cur_x, area.y, btn_w, area.height);
        if next_rect.contains(event.x as i32, event.y as i32) {
            return WidgetAction::NextTrack;
        }

        WidgetAction::None
    }
}

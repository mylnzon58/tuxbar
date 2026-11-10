//! Clock widget displaying crisp time and date

use chrono::Local;
use crate::graphics::{FontEngine, Framebuffer};
use crate::types::{Color, Rect, TouchEvent, WidgetAction};
use crate::widgets::Widget;

#[derive(Default)]
pub struct ClockWidget;

impl Widget for ClockWidget {
    fn render(&self, fb: &mut Framebuffer, area: Rect, font: &FontEngine) {
        let now = Local::now();
        let time_str = now.format("%H:%M").to_string();
        let date_str = now.format("%a %d %b").to_string().to_uppercase();

        // Render card background
        fb.draw_rounded_rect(area, 6, Color::CARD_BG);

        // Draw time
        let time_w = font.measure_text_width(&time_str, 24.0);
        let time_x = area.x + (area.width as i32 - time_w as i32) / 2;
        let time_y = area.y + 36;
        fb.draw_text(font, &time_str, time_x, time_y, 24.0, Color::WHITE);

        // Draw date
        let date_w = font.measure_text_width(&date_str, 12.0);
        let date_x = area.x + (area.width as i32 - date_w as i32) / 2;
        let date_y = area.y + 52;
        fb.draw_text(font, &date_str, date_x, date_y, 12.0, Color::GRAY);
    }

    fn handle_touch(&mut self, _event: &TouchEvent, _area: Rect) -> WidgetAction {
        // Clock touch toggles display mode (e.g. Fn keys or extra widgets)
        WidgetAction::ToggleMode
    }
}


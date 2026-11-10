//! System telemetry widget displaying CPU usage, temperature, and RAM
//! Designed with large 22px bold metrics filling the full 56px height.

use crate::graphics::{FontEngine, Framebuffer};
use crate::services::telemetry::SystemStats;
use crate::types::{Color, Rect, TouchEvent, WidgetAction};
use crate::widgets::Widget;

pub struct SystemStatsWidget {
    pub stats: SystemStats,
}

impl SystemStatsWidget {
    #[must_use]
    pub fn new() -> Self {
        Self {
            stats: SystemStats::default(),
        }
    }

    pub fn update_stats(&mut self, stats: SystemStats) {
        self.stats = stats;
    }
}

impl Widget for SystemStatsWidget {
    fn render(&self, fb: &mut Framebuffer, area: Rect, font: &FontEngine) {
        fb.draw_rounded_rect(area, 6, Color::CARD_BG);

        let btn_h = area.height.saturating_sub(4);
        let btn_y = area.y + 2;
        let card_w = (area.width.saturating_sub(16)) / 3;

        // Card 1: CPU Load
        let cpu_x = area.x + 4;
        let cpu_rect = Rect::new(cpu_x, btn_y, card_w, btn_h);
        fb.draw_rounded_rect(cpu_rect, 4, Color::DARK_GRAY);
        let cpu_color = if self.stats.cpu_usage_pct > 80.0 {
            Color::RED
        } else if self.stats.cpu_usage_pct > 50.0 {
            Color::YELLOW
        } else {
            Color::GREEN
        };
        let cpu_str = format!("{:.0}%", self.stats.cpu_usage_pct);
        let cpu_w = font.measure_text_width(&cpu_str, 22.0);
        let cpu_txt_x = cpu_x + (card_w as i32 - cpu_w as i32) / 2;
        fb.draw_text(font, &cpu_str, cpu_txt_x, btn_y + 28, 22.0, cpu_color);
        let sub_w = font.measure_text_width("CPU", 11.0);
        fb.draw_text(font, "CPU", cpu_x + (card_w as i32 - sub_w as i32) / 2, btn_y + 46, 11.0, Color::GRAY);

        // Card 2: Temperature
        let temp_x = cpu_x + card_w as i32 + 4;
        let temp_rect = Rect::new(temp_x, btn_y, card_w, btn_h);
        fb.draw_rounded_rect(temp_rect, 4, Color::DARK_GRAY);
        let temp_color = if self.stats.temp_c > 85.0 {
            Color::RED
        } else if self.stats.temp_c > 70.0 {
            Color::YELLOW
        } else {
            Color::GREEN
        };
        let temp_str = format!("{:.0}°", self.stats.temp_c);
        let temp_w = font.measure_text_width(&temp_str, 22.0);
        let temp_txt_x = temp_x + (card_w as i32 - temp_w as i32) / 2;
        fb.draw_text(font, &temp_str, temp_txt_x, btn_y + 28, 22.0, temp_color);
        let sub_temp = font.measure_text_width("TEMP", 11.0);
        fb.draw_text(font, "TEMP", temp_x + (card_w as i32 - sub_temp as i32) / 2, btn_y + 46, 11.0, Color::GRAY);

        // Card 3: RAM
        let ram_x = temp_x + card_w as i32 + 4;
        let ram_rect = Rect::new(ram_x, btn_y, card_w, btn_h);
        fb.draw_rounded_rect(ram_rect, 4, Color::DARK_GRAY);
        let ram_str = format!("{:.1}G", self.stats.mem_used_mb as f32 / 1024.0);
        let ram_w = font.measure_text_width(&ram_str, 20.0);
        let ram_txt_x = ram_x + (card_w as i32 - ram_w as i32) / 2;
        fb.draw_text(font, &ram_str, ram_txt_x, btn_y + 28, 20.0, Color::CYAN);
        let sub_ram = font.measure_text_width("RAM", 11.0);
        fb.draw_text(font, "RAM", ram_x + (card_w as i32 - sub_ram as i32) / 2, btn_y + 46, 11.0, Color::GRAY);
    }

    fn handle_touch(&mut self, _event: &TouchEvent, _area: Rect) -> WidgetAction {
        WidgetAction::ToggleMode
    }
}

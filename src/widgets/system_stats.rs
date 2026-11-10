//! System telemetry widget displaying CPU usage, temperature, and RAM

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
        // Draw card background
        fb.draw_rounded_rect(area, 6, Color::CARD_BG);

        let col1_x = area.x + 12;
        let col2_x = area.x + (area.width as i32 / 2) + 6;

        // CPU Usage
        let cpu_str = format!("CPU {:>2.0}%", self.stats.cpu_usage_pct);
        let cpu_color = if self.stats.cpu_usage_pct > 80.0 {
            Color::RED
        } else if self.stats.cpu_usage_pct > 50.0 {
            Color::YELLOW
        } else {
            Color::GREEN
        };
        fb.draw_text(font, &cpu_str, col1_x, area.y + 25, 14.0, cpu_color);

        // Mini bar for CPU
        let cpu_bar_rect = Rect::new(col1_x, area.y + 36, (area.width / 2) - 20, 6);
        fb.draw_progress_bar(cpu_bar_rect, self.stats.cpu_usage_pct / 100.0, Color::DARK_GRAY, cpu_color);

        // Temperature
        let temp_str = format!("{:.0}°C", self.stats.temp_c);
        let temp_color = if self.stats.temp_c > 85.0 {
            Color::RED
        } else if self.stats.temp_c > 70.0 {
            Color::YELLOW
        } else {
            Color::GREEN
        };
        fb.draw_text(font, &temp_str, col2_x, area.y + 25, 14.0, temp_color);

        // Memory Usage
        let mem_str = format!("{:.1}G", self.stats.mem_used_mb as f32 / 1024.0);
        fb.draw_text(font, &mem_str, col2_x + 55, area.y + 25, 14.0, Color::CYAN);

        let mem_ratio = if self.stats.mem_total_mb > 0 {
            self.stats.mem_used_mb as f32 / self.stats.mem_total_mb as f32
        } else {
            0.0
        };
        let mem_bar_rect = Rect::new(col2_x, area.y + 36, (area.width / 2) - 20, 6);
        fb.draw_progress_bar(mem_bar_rect, mem_ratio, Color::DARK_GRAY, Color::CYAN);
    }

    fn handle_touch(&mut self, _event: &TouchEvent, _area: Rect) -> WidgetAction {
        WidgetAction::None
    }
}


//! Cyberdeck Hardware Telemetry Matrix Widget
//! Displays per-core CPU load for up to 16 threads, RAM, and thermal dynamics.

use crate::graphics::{FontEngine, Framebuffer};
use crate::services::telemetry::SystemStats;
use crate::types::{Color, Rect, TouchEvent, WidgetAction};
use crate::widgets::Widget;

pub struct CyberdeckWidget {
    stats: SystemStats,
    pulse: f32,
}

impl CyberdeckWidget {
    #[must_use]
    pub fn new() -> Self {
        Self {
            stats: SystemStats::default(),
            pulse: 0.0,
        }
    }

    pub fn update_stats(&mut self, stats: SystemStats) {
        self.stats = stats;
    }

    pub fn tick(&mut self) {
        self.pulse += 0.1;
    }
}

impl Widget for CyberdeckWidget {
    fn render(&self, fb: &mut Framebuffer, area: Rect, font: &FontEngine) {
        // Draw background card with subtle border glow
        fb.draw_rounded_rect(area, 6, Color::CARD_BG);

        let mut cur_x = area.x + 12;

        // 1. Cyberdeck Header Badge
        let badge_text = "CYBERDECK · T2";
        let badge_w = font.measure_text_width(badge_text, 11.0) + 12;
        let badge_rect = Rect::new(cur_x, area.y + 16, badge_w, 24);
        fb.draw_rounded_rect(badge_rect, 4, Color::DARK_GRAY);
        fb.draw_text(font, badge_text, cur_x + 6, area.y + 32, 11.0, Color::CYAN);
        cur_x += badge_w as i32 + 16;

        // 2. 12-Core CPU Activity Matrix
        let core_count = if self.stats.core_count > 0 {
            self.stats.core_count.min(16)
        } else {
            12
        };

        let matrix_w = 480;
        let col_w = (matrix_w / core_count as u32).saturating_sub(4);
        let col_h = 36;
        let col_y = area.y + 12;

        for i in 0..core_count {
            let load = self.stats.core_usage_pct[i].clamp(0.0, 100.0);
            let bar_x = cur_x + (i as u32 * (col_w + 4)) as i32;

            // Background slot
            let slot_rect = Rect::new(bar_x, col_y, col_w, col_h);
            fb.draw_rounded_rect(slot_rect, 2, Color::DARK_GRAY);

            // Active bar
            let fill_h = ((col_h as f32) * (load / 100.0)).round() as u32;
            if fill_h > 0 {
                let fill_y = col_y + col_h as i32 - fill_h as i32;
                let active_rect = Rect::new(bar_x, fill_y, col_w, fill_h);

                let color = if load > 85.0 {
                    Color::RED
                } else if load > 60.0 {
                    Color::ORANGE
                } else if load > 30.0 {
                    Color::YELLOW
                } else {
                    Color::NEON_CYAN
                };

                fb.draw_rounded_rect(active_rect, 2, color);
            }

            // Core label below
            let core_label = format!("{i}");
            let lbl_w = font.measure_text_width(&core_label, 9.0);
            let lbl_x = bar_x + ((col_w as i32 - lbl_w as i32) / 2);
            fb.draw_text(font, &core_label, lbl_x, area.y + 54, 9.0, Color::GRAY);
        }
        cur_x += matrix_w as i32 + 20;

        // 3. Overall CPU % & Temp Badge
        let cpu_str = format!("AVG CPU {:>2.0}%", self.stats.cpu_usage_pct);
        fb.draw_text(font, &cpu_str, cur_x, area.y + 24, 13.0, Color::WHITE);

        let temp_color = if self.stats.temp_c > 85.0 {
            Color::RED
        } else if self.stats.temp_c > 70.0 {
            Color::ORANGE
        } else {
            Color::GREEN
        };
        let temp_str = format!("{:.0}°C", self.stats.temp_c);
        fb.draw_text(font, &temp_str, cur_x, area.y + 44, 14.0, temp_color);
        cur_x += 120;

        // 4. Memory Usage Telemetry
        let mem_str = format!("RAM: {:.1}G / {:.1}G",
            self.stats.mem_used_mb as f32 / 1024.0,
            self.stats.mem_total_mb as f32 / 1024.0
        );
        fb.draw_text(font, &mem_str, cur_x, area.y + 24, 12.0, Color::WHITE);

        let mem_ratio = if self.stats.mem_total_mb > 0 {
            self.stats.mem_used_mb as f32 / self.stats.mem_total_mb as f32
        } else {
            0.0
        };
        let mem_bar_rect = Rect::new(cur_x, area.y + 32, 160, 8);
        fb.draw_progress_bar(mem_bar_rect, mem_ratio, Color::DARK_GRAY, Color::PURPLE);
        cur_x += 180;

        // 5. Heartbeat Activity Pulse
        let pulse_alpha = ((self.pulse.sin() * 0.5 + 0.5) * 200.0) as u8 + 55;
        let pulse_color = Color::rgba(0, 240, 255, pulse_alpha);
        let pulse_badge = "⚡ KERNEL LINK ACTIVE";
        fb.draw_text(font, pulse_badge, cur_x, area.y + 34, 11.0, pulse_color);
    }

    fn handle_touch(&mut self, _event: &TouchEvent, _area: Rect) -> WidgetAction {
        WidgetAction::ToggleMode
    }
}


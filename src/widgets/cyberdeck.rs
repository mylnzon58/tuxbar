//! Cyberdeck Hardware Telemetry Matrix Widget
//! Displays per-core CPU load for up to 16 threads, RAM, and thermal dynamics.
//! Fills the full 58px height with vivid, readable telemetry.

use crate::graphics::{FontEngine, Framebuffer};
use crate::services::telemetry::SystemStats;
use crate::types::{Color, Rect, TouchEvent, WidgetAction};
use crate::widgets::Widget;

pub struct CyberdeckWidget {
    stats: SystemStats,
    smooth_core_usage: [f32; 16],
    pulse: f32,
}

impl CyberdeckWidget {
    #[must_use]
    pub fn new() -> Self {
        Self {
            stats: SystemStats::default(),
            smooth_core_usage: [0.0; 16],
            pulse: 0.0,
        }
    }

    pub fn update_stats(&mut self, stats: SystemStats) {
        self.stats = stats;
    }

    pub fn tick(&mut self) {
        self.pulse += 0.1;
        // Smoothly interpolate displayed core load towards target kernel telemetry at 30 FPS
        for i in 0..16 {
            let target = self.stats.core_usage_pct[i];
            self.smooth_core_usage[i] += (target - self.smooth_core_usage[i]) * 0.25;
        }
    }
}

impl Widget for CyberdeckWidget {
    fn render(&self, fb: &mut Framebuffer, area: Rect, font: &FontEngine) {
        fb.fill_rect(area, Color::BLACK);

        let btn_h = area.height.saturating_sub(2);
        let btn_y = area.y + 1;
        let mut cur_x = area.x + 8;

        // 1. Esc Key
        let esc_w = 90;
        let esc_rect = Rect::new(cur_x, btn_y, esc_w, btn_h);
        fb.draw_rounded_rect(esc_rect, 6, Color::rgb(45, 55, 72));
        let esc_text_w = font.measure_text_width("Esc", 22.0);
        fb.draw_text(font, "Esc", cur_x + (esc_w as i32 - esc_text_w as i32) / 2, btn_y + 38, 22.0, Color::CYAN);
        cur_x += esc_w as i32 + 10;

        // 2. 12/16-Core CPU Activity Matrix (full 50px bars)
        let core_count = if self.stats.core_count > 0 {
            self.stats.core_count.min(16)
        } else {
            12
        };

        let matrix_w = 440;
        let col_w = (matrix_w / core_count as u32).saturating_sub(4);
        let col_h = 46;
        let col_y = area.y + 7;

        for i in 0..core_count {
            let load = self.smooth_core_usage[i].clamp(0.0, 100.0);
            let bar_x = cur_x + (i as u32 * (col_w + 4)) as i32;

            // Background slot
            let slot_rect = Rect::new(bar_x, col_y, col_w, col_h);
            fb.draw_rounded_rect(slot_rect, 3, Color::rgb(26, 32, 44));

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

                fb.draw_rounded_rect(active_rect, 3, color);
            }
        }
        cur_x += matrix_w as i32 + 20;

        // 3. Overall CPU % & Temp Badge (Bold 22px)
        let cpu_str = format!("⚡ CPU {:.0}%", self.stats.cpu_usage_pct);
        fb.draw_text(font, &cpu_str, cur_x, btn_y + 24, 18.0, Color::WHITE);

        let temp_color = if self.stats.temp_c > 85.0 {
            Color::RED
        } else if self.stats.temp_c > 70.0 {
            Color::ORANGE
        } else {
            Color::GREEN
        };
        let temp_str = format!("🔥 {:.0}°C", self.stats.temp_c);
        fb.draw_text(font, &temp_str, cur_x, btn_y + 48, 18.0, temp_color);
        cur_x += 160;

        // 4. Memory Telemetry (Bold 20px)
        let mem_str = format!("🧠 RAM {:.1}G / {:.1}G",
            self.stats.mem_used_mb as f32 / 1024.0,
            self.stats.mem_total_mb as f32 / 1024.0
        );
        fb.draw_text(font, &mem_str, cur_x, btn_y + 24, 18.0, Color::WHITE);

        let mem_ratio = if self.stats.mem_total_mb > 0 {
            self.stats.mem_used_mb as f32 / self.stats.mem_total_mb as f32
        } else {
            0.0
        };
        let mem_w = font.measure_text_width(&mem_str, 18.0);
        let mem_bar_w = (mem_w as u32).max(180);
        let mem_bar_rect = Rect::new(cur_x, btn_y + 36, mem_bar_w, 8);
        fb.draw_progress_bar(mem_bar_rect, mem_ratio, Color::DARK_GRAY, Color::PURPLE);
        cur_x += mem_bar_w as i32 + 42;

        // 5. Heartbeat Activity Pulse
        let pulse_alpha = ((self.pulse.sin() * 0.5 + 0.5) * 200.0) as u8 + 55;
        let pulse_color = Color::rgba(0, 240, 255, pulse_alpha);
        let pulse_badge = "● KERNEL LINK";
        fb.draw_text(font, pulse_badge, cur_x, btn_y + 36, 18.0, pulse_color);
    }

    fn handle_touch(&mut self, _event: &TouchEvent, _area: Rect) -> WidgetAction {
        WidgetAction::ToggleMode
    }
}

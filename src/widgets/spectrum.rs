//! Audiophile Dynamic Spectrum Analyzer Widget
//! Displays a multi-band dancing frequency equalizer with neon gradient and peak hold.

use crate::graphics::{FontEngine, Framebuffer};
use crate::types::{Color, Rect, TouchEvent, WidgetAction};
use crate::widgets::Widget;

const BANDS_COUNT: usize = 32;

pub struct SpectrumVisualizerWidget {
    levels: [f32; BANDS_COUNT],
    peaks: [f32; BANDS_COUNT],
    tick: f32,
    is_active: bool,
}

impl SpectrumVisualizerWidget {
    #[must_use]
    pub fn new() -> Self {
        Self {
            levels: [0.0; BANDS_COUNT],
            peaks: [0.0; BANDS_COUNT],
            tick: 0.0,
            is_active: true,
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    pub fn update_frame(&mut self) {
        self.tick += 0.15;

        for i in 0..BANDS_COUNT {
            if self.is_active {
                // Synthesize rich harmonic wave dynamics (simulating live audio frequency response)
                let freq = (i as f32 + 1.0) * 0.45;
                let wave1 = (self.tick * 2.2 + freq).sin();
                let wave2 = (self.tick * 3.7 - freq * 0.8).cos();
                let wave3 = (self.tick * 5.1 + freq * 1.5).sin();
                let combined = (wave1 * 0.45 + wave2 * 0.35 + wave3 * 0.20 + 1.0) / 2.0;

                // Bass bias for lower bands, mid-treble sparkle
                let bias = if i < 8 { 1.2 } else if i < 20 { 1.0 } else { 0.8 };
                let target = (combined * bias).clamp(0.08, 1.0);

                // Smooth dampening towards target
                self.levels[i] += (target - self.levels[i]) * 0.4;
            } else {
                // Ambient idle breathing wave
                let idle = ((self.tick * 0.5 + i as f32 * 0.2).sin() * 0.5 + 0.5) * 0.15 + 0.05;
                self.levels[i] += (idle - self.levels[i]) * 0.2;
            }

            // Update peak-hold indicators
            if self.levels[i] > self.peaks[i] {
                self.peaks[i] = self.levels[i];
            } else {
                self.peaks[i] = (self.peaks[i] - 0.02).max(0.0);
            }
        }
    }
}

impl Widget for SpectrumVisualizerWidget {
    fn render(&self, fb: &mut Framebuffer, area: Rect, _font: &FontEngine) {
        // Draw dark card background
        fb.draw_rounded_rect(area, 6, Color::CARD_BG);

        let padding_x = 8;
        let padding_y = 6;
        let inner_w = area.width.saturating_sub(padding_x * 2);
        let inner_h = area.height.saturating_sub(padding_y * 2);

        let band_gap = 3;
        let total_gaps = (BANDS_COUNT - 1) as u32 * band_gap;
        let bar_width = (inner_w.saturating_sub(total_gaps) / BANDS_COUNT as u32).max(3);

        let segments = 10;
        let seg_gap = 2;
        let seg_height = (inner_h.saturating_sub((segments - 1) * seg_gap) / segments).max(2);

        for (i, (&lvl, &peak)) in self.levels.iter().zip(self.peaks.iter()).enumerate() {
            let bar_x = area.x + padding_x as i32 + i as i32 * (bar_width + band_gap) as i32;
            let active_segs = (lvl * segments as f32).round() as u32;

            for seg in 0..segments {
                let seg_y = area.y + area.height as i32 - padding_y as i32 - ((seg + 1) * (seg_height + seg_gap)) as i32;
                let seg_rect = Rect::new(bar_x, seg_y, bar_width, seg_height);

                let color = if seg < active_segs {
                    // Segment gradient: Bottom Cyan -> Mid Magenta -> Top Pink
                    if seg < 4 {
                        Color::CYAN
                    } else if seg < 7 {
                        Color::MAGENTA
                    } else {
                        Color::PINK
                    }
                } else {
                    // Inactive segment slot
                    Color::rgba(255, 255, 255, 12)
                };

                fb.fill_rect(seg_rect, color);
            }

            // Draw floating peak indicator dot
            let peak_seg = (peak * (segments - 1) as f32).round() as u32;
            let peak_y = area.y + area.height as i32 - padding_y as i32 - ((peak_seg + 1) * (seg_height + seg_gap)) as i32;
            let peak_rect = Rect::new(bar_x, peak_y, bar_width, 2);
            fb.fill_rect(peak_rect, Color::WHITE);
        }
    }

    fn handle_touch(&mut self, _event: &TouchEvent, _area: Rect) -> WidgetAction {
        WidgetAction::PlayPause
    }
}

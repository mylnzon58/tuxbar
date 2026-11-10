//! Audiophile Real-Audio Frequency Analyzer Widget
//! Renders 32 frequency bands driven by ACTUAL sound waves from PipeWire / PulseAudio.
//! Displays real decibels (dB), pure hardware frequency reaction, and flatline when paused/silent.

use crate::graphics::{FontEngine, Framebuffer};
use crate::services::audio_capture::AudioTelemetry;
use crate::types::{Color, Rect, TouchEvent, WidgetAction};
use crate::widgets::Widget;

const BANDS_COUNT: usize = 32;

pub struct SpectrumVisualizerWidget {
    levels: [f32; BANDS_COUNT],
    peaks: [f32; BANDS_COUNT],
    current_db: f32,
    is_silent: bool,
}

impl SpectrumVisualizerWidget {
    #[must_use]
    pub fn new() -> Self {
        Self {
            levels: [0.0; BANDS_COUNT],
            peaks: [0.0; BANDS_COUNT],
            current_db: -100.0,
            is_silent: true,
        }
    }

    pub fn set_active(&mut self, _active: bool) {
        // Kept for backward compatibility
    }

    pub fn update_frame(&mut self) {
        // Kept for backward compatibility
    }

    pub fn update_with_real_audio(&mut self, audio: &AudioTelemetry) {
        self.is_silent = audio.is_silent;
        self.current_db = audio.db;

        if audio.is_silent {
            for i in 0..BANDS_COUNT {
                self.levels[i] = (self.levels[i] * 0.4).max(0.0);
                if self.levels[i] < 0.01 {
                    self.levels[i] = 0.0;
                }
                self.peaks[i] = (self.peaks[i] * 0.4).max(0.0);
                if self.peaks[i] < 0.01 {
                    self.peaks[i] = 0.0;
                }
            }
        } else {
            for i in 0..BANDS_COUNT {
                let target = audio.bands[i];
                if target > self.levels[i] {
                    self.levels[i] += (target - self.levels[i]) * 0.6;
                } else {
                    self.levels[i] += (target - self.levels[i]) * 0.3;
                }

                if self.levels[i] > self.peaks[i] {
                    self.peaks[i] = self.levels[i];
                } else {
                    self.peaks[i] = (self.peaks[i] - 0.02).max(0.0);
                }
            }
        }
    }
}

impl Widget for SpectrumVisualizerWidget {
    fn render(&self, fb: &mut Framebuffer, area: Rect, font: &FontEngine) {
        // Draw dark card background
        fb.draw_rounded_rect(area, 6, Color::CARD_BG);

        let padding_x = 8;
        let padding_y = 4;
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
            if peak > 0.05 {
                let peak_seg = (peak * (segments - 1) as f32).round() as u32;
                let peak_y = area.y + area.height as i32 - padding_y as i32 - ((peak_seg + 1) * (seg_height + seg_gap)) as i32;
                let peak_rect = Rect::new(bar_x, peak_y, bar_width, 2);
                fb.fill_rect(peak_rect, Color::WHITE);
            }
        }

        // Draw live decibels indicator badge if active
        if !self.is_silent && self.current_db > -70.0 {
            let db_text = format!("{:.0} dB", self.current_db);
            let tw = font.measure_text_width(&db_text, 14.0);
            let tx = area.x + area.width as i32 - tw as i32 - 12;
            fb.draw_text(font, &db_text, tx, area.y + 20, 14.0, Color::CYAN);
        }
    }

    fn handle_touch(&mut self, _event: &TouchEvent, _area: Rect) -> WidgetAction {
        WidgetAction::PlayPause
    }
}

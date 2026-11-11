//! Control Deck & System Shortcuts Widget
//! Provides dedicated hardware and desktop shortcuts:
//! Brightness, Mic Mute, Volume, Mute, Calculator, Browser, Files, Overview/Desktops, and Media.

use crate::graphics::{FontEngine, Framebuffer};
use crate::types::{Color, Rect, TouchEvent, TouchPhase, WidgetAction};
use crate::widgets::Widget;

pub struct ControlButton {
    pub label: &'static str,
    pub subtext: &'static str,
    pub color: Color,
    pub action: WidgetAction,
}

pub struct ControlDeckWidget {
    pub active_button: Option<usize>,
}

impl ControlDeckWidget {
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_button: None,
        }
    }

    #[allow(dead_code)]
    pub fn get_buttons() -> &'static [ControlButton] {
        &BUTTONS
    }
}

const BUTTONS: [ControlButton; 13] = [
    ControlButton {
        label: "ESC",
        subtext: "EXIT",
        color: Color::rgb(220, 38, 38), // Red
        action: WidgetAction::Escape,
    },
    ControlButton {
        label: "BRI -",
        subtext: "DIM",
        color: Color::rgb(217, 119, 6), // Amber
        action: WidgetAction::BrightnessDown,
    },
    ControlButton {
        label: "BRI +",
        subtext: "GLOW",
        color: Color::rgb(245, 158, 11), // Bright Amber
        action: WidgetAction::BrightnessUp,
    },
    ControlButton {
        label: "MIC",
        subtext: "MUTE",
        color: Color::rgb(225, 29, 72), // Rose/Crimson
        action: WidgetAction::ToggleMicMute,
    },
    ControlButton {
        label: "VOL -",
        subtext: "DOWN",
        color: Color::rgb(14, 165, 233), // Sky Cyan
        action: WidgetAction::VolumeDown,
    },
    ControlButton {
        label: "VOL +",
        subtext: "UP",
        color: Color::rgb(14, 165, 233), // Sky Cyan
        action: WidgetAction::VolumeUp,
    },
    ControlButton {
        label: "MUTE",
        subtext: "AUDIO",
        color: Color::rgb(168, 85, 247), // Violet
        action: WidgetAction::ToggleMute,
    },
    ControlButton {
        label: "CALC",
        subtext: "MATH",
        color: Color::rgb(16, 185, 129), // Emerald
        action: WidgetAction::LaunchCalc,
    },
    ControlButton {
        label: "WEB",
        subtext: "BROWSE",
        color: Color::rgb(59, 130, 246), // Electric Blue
        action: WidgetAction::LaunchBrowser,
    },
    ControlButton {
        label: "FILES",
        subtext: "FOLDERS",
        color: Color::rgb(234, 88, 12), // Deep Orange
        action: WidgetAction::LaunchFiles,
    },
    ControlButton {
        label: "DESKS",
        subtext: "SPACES",
        color: Color::rgb(147, 51, 234), // Neon Purple
        action: WidgetAction::ToggleOverview,
    },
    ControlButton {
        label: "PLAY",
        subtext: "PAUSE",
        color: Color::rgb(34, 197, 94), // Green
        action: WidgetAction::PlayPause,
    },
    ControlButton {
        label: "NEXT",
        subtext: "SKIP",
        color: Color::rgb(6, 182, 212), // Cyan
        action: WidgetAction::NextTrack,
    },
];

impl Widget for ControlDeckWidget {
    fn render(&self, fb: &mut Framebuffer, area: Rect, font: &FontEngine) {
        fb.fill_rect(area, Color::BLACK);

        let count = BUTTONS.len();
        let gap = 5;
        let total_gaps = (count - 1) as i32 * gap;
        let btn_w = (area.width as i32 - total_gaps) / count as i32;
        let btn_h = area.height.saturating_sub(2);
        let btn_y = area.y + 1;

        for (i, btn) in BUTTONS.iter().enumerate() {
            let btn_x = area.x + i as i32 * (btn_w + gap);
            let btn_rect = Rect::new(btn_x, btn_y, btn_w as u32, btn_h);

            let is_pressed = self.active_button == Some(i);

            let bg_color = if is_pressed {
                btn.color
            } else {
                Color::rgb(20, 26, 38)
            };

            fb.draw_rounded_rect(btn_rect, 6, bg_color);

            // Subtle border outline in theme color
            if !is_pressed {
                fb.fill_rect(
                    Rect::new(btn_x + 1, btn_y + 1, btn_w as u32 - 2, 2),
                    btn.color,
                );
            }

            // Primary Label (19px bold)
            let font_size = 19.0;
            let text_w = font.measure_text_width(btn.label, font_size);
            let text_x = btn_x + (btn_w - text_w as i32) / 2;
            let text_y = btn_y + 28;

            let text_color = if is_pressed {
                Color::BLACK
            } else {
                Color::WHITE
            };
            fb.draw_text(font, btn.label, text_x, text_y, font_size, text_color);

            // Secondary Subtext (11px micro badge)
            let sub_size = 11.0;
            let sub_w = font.measure_text_width(btn.subtext, sub_size);
            let sub_x = btn_x + (btn_w - sub_w as i32) / 2;
            let sub_y = btn_y + 46;

            let sub_color = if is_pressed {
                Color::rgb(30, 30, 30)
            } else {
                btn.color
            };
            fb.draw_text(font, btn.subtext, sub_x, sub_y, sub_size, sub_color);
        }
    }

    fn handle_touch(&mut self, event: &TouchEvent, area: Rect) -> WidgetAction {
        let count = BUTTONS.len();
        let gap = 5;
        let total_gaps = (count - 1) as i32 * gap;
        let btn_w = (area.width as i32 - total_gaps) / count as i32;

        let px = event.x as i32;
        let py = event.y as i32;

        if !area.contains(px, py) {
            self.active_button = None;
            return WidgetAction::None;
        }

        let offset_x = px - area.x;
        let stride = btn_w + gap;
        let idx = (offset_x / stride) as usize;

        if idx >= count {
            self.active_button = None;
            return WidgetAction::None;
        }

        match event.phase {
            TouchPhase::Down => {
                self.active_button = Some(idx);
                BUTTONS[idx].action.clone()
            }
            TouchPhase::Motion => {
                self.active_button = Some(idx);
                WidgetAction::None
            }
            TouchPhase::Up => {
                self.active_button = None;
                WidgetAction::None
            }
        }
    }
}

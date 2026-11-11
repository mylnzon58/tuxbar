//! Control Deck & System Shortcuts Widget
//! Provides 15 dedicated hardware, system, and desktop shortcuts:
//! Esc, Screen Brightness, Keyboard Backlight, Mic Mute, Volume, Mute,
//! Calculator, Web Browser, Files/Nautilus, Desktops/Spaces, and Media.

use crate::graphics::{FontEngine, Framebuffer};
use crate::types::{Color, Rect, TouchEvent, TouchPhase, WidgetAction};
use crate::widgets::Widget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconKind {
    Esc,
    ScreenBriDown,
    ScreenBriUp,
    KbdBriDown,
    KbdBriUp,
    MicMute,
    VolDown,
    VolUp,
    AudioMute,
    Calc,
    Web,
    Files,
    Desks,
    PlayPause,
    NextTrack,
}

pub struct ControlButton {
    pub label: &'static str,
    pub icon: IconKind,
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

const BUTTONS: [ControlButton; 15] = [
    ControlButton {
        label: "ESC",
        icon: IconKind::Esc,
        color: Color::rgb(220, 38, 38), // Crimson Red
        action: WidgetAction::Escape,
    },
    ControlButton {
        label: "BRI -",
        icon: IconKind::ScreenBriDown,
        color: Color::rgb(217, 119, 6), // Amber
        action: WidgetAction::BrightnessDown,
    },
    ControlButton {
        label: "BRI +",
        icon: IconKind::ScreenBriUp,
        color: Color::rgb(245, 158, 11), // Bright Amber
        action: WidgetAction::BrightnessUp,
    },
    ControlButton {
        label: "KBD -",
        icon: IconKind::KbdBriDown,
        color: Color::rgb(202, 138, 4), // Warm Gold
        action: WidgetAction::KbdBrightnessDown,
    },
    ControlButton {
        label: "KBD +",
        icon: IconKind::KbdBriUp,
        color: Color::rgb(234, 179, 8), // Radiant Gold
        action: WidgetAction::KbdBrightnessUp,
    },
    ControlButton {
        label: "MIC",
        icon: IconKind::MicMute,
        color: Color::rgb(225, 29, 72), // Vivid Rose/Red
        action: WidgetAction::ToggleMicMute,
    },
    ControlButton {
        label: "VOL -",
        icon: IconKind::VolDown,
        color: Color::rgb(14, 165, 233), // Sky Cyan
        action: WidgetAction::VolumeDown,
    },
    ControlButton {
        label: "VOL +",
        icon: IconKind::VolUp,
        color: Color::rgb(14, 165, 233), // Sky Cyan
        action: WidgetAction::VolumeUp,
    },
    ControlButton {
        label: "MUTE",
        icon: IconKind::AudioMute,
        color: Color::rgb(168, 85, 247), // Violet
        action: WidgetAction::ToggleMute,
    },
    ControlButton {
        label: "CALC",
        icon: IconKind::Calc,
        color: Color::rgb(16, 185, 129), // Emerald
        action: WidgetAction::LaunchCalc,
    },
    ControlButton {
        label: "WEB",
        icon: IconKind::Web,
        color: Color::rgb(59, 130, 246), // Electric Blue
        action: WidgetAction::LaunchBrowser,
    },
    ControlButton {
        label: "FILES",
        icon: IconKind::Files,
        color: Color::rgb(234, 88, 12), // Deep Orange
        action: WidgetAction::LaunchFiles,
    },
    ControlButton {
        label: "DESKS",
        icon: IconKind::Desks,
        color: Color::rgb(147, 51, 234), // Neon Purple
        action: WidgetAction::ToggleOverview,
    },
    ControlButton {
        label: "PLAY",
        icon: IconKind::PlayPause,
        color: Color::rgb(34, 197, 94), // Green
        action: WidgetAction::PlayPause,
    },
    ControlButton {
        label: "NEXT",
        icon: IconKind::NextTrack,
        color: Color::rgb(6, 182, 212), // Cyan
        action: WidgetAction::NextTrack,
    },
];

fn draw_control_icon(
    fb: &mut Framebuffer,
    icon: IconKind,
    cx: i32,
    cy: i32,
    color: Color,
    is_pressed: bool,
) {
    let primary_c = if is_pressed { Color::BLACK } else { color };
    let alt_c = if is_pressed { Color::rgb(40, 40, 40) } else { Color::WHITE };

    match icon {
        IconKind::Esc => {
            fb.draw_circle(cx, cy, 8, primary_c);
            fb.draw_line(cx - 3, cy - 3, cx + 3, cy + 3, primary_c);
            fb.draw_line(cx - 3, cy + 3, cx + 3, cy - 3, primary_c);
        }
        IconKind::ScreenBriDown => {
            fb.draw_circle(cx - 3, cy, 4, primary_c);
            fb.draw_line(cx - 3, cy - 7, cx - 3, cy - 5, primary_c);
            fb.draw_line(cx - 3, cy + 5, cx - 3, cy + 7, primary_c);
            fb.draw_line(cx - 8, cy, cx - 6, cy, primary_c);
            fb.draw_line(cx, cy, cx + 2, cy, primary_c);
            // Minus [-]
            fb.draw_line(cx + 5, cy, cx + 10, cy, primary_c);
            fb.draw_line(cx + 5, cy + 1, cx + 10, cy + 1, primary_c);
        }
        IconKind::ScreenBriUp => {
            fb.fill_circle(cx - 3, cy, 4, primary_c);
            fb.draw_line(cx - 3, cy - 8, cx - 3, cy - 5, primary_c);
            fb.draw_line(cx - 3, cy + 5, cx - 3, cy + 8, primary_c);
            fb.draw_line(cx - 9, cy, cx - 6, cy, primary_c);
            fb.draw_line(cx, cy, cx + 3, cy, primary_c);
            fb.draw_line(cx - 7, cy - 5, cx - 5, cy - 3, primary_c);
            fb.draw_line(cx - 1, cy + 3, cx + 1, cy + 5, primary_c);
            fb.draw_line(cx - 7, cy + 5, cx - 5, cy + 3, primary_c);
            fb.draw_line(cx - 1, cy - 3, cx + 1, cy - 5, primary_c);
            // Plus [+]
            fb.draw_line(cx + 5, cy, cx + 11, cy, primary_c);
            fb.draw_line(cx + 5, cy + 1, cx + 11, cy + 1, primary_c);
            fb.draw_line(cx + 8, cy - 3, cx + 8, cy + 3, primary_c);
        }
        IconKind::KbdBriDown => {
            // Keyboard frame
            fb.draw_rounded_rect(Rect::new(cx - 10, cy - 5, 20, 11), 2, primary_c);
            // Keycaps
            fb.fill_rect(Rect::new(cx - 7, cy - 3, 3, 2), alt_c);
            fb.fill_rect(Rect::new(cx - 2, cy - 3, 3, 2), alt_c);
            fb.fill_rect(Rect::new(cx + 3, cy - 3, 3, 2), alt_c);
            fb.fill_rect(Rect::new(cx - 5, cy + 1, 9, 2), alt_c);
            // Dim minus
            fb.draw_line(cx - 4, cy - 8, cx + 4, cy - 8, primary_c);
        }
        IconKind::KbdBriUp => {
            // Keyboard frame
            fb.draw_rounded_rect(Rect::new(cx - 10, cy - 5, 20, 11), 2, primary_c);
            // Keycaps
            fb.fill_rect(Rect::new(cx - 7, cy - 3, 3, 2), alt_c);
            fb.fill_rect(Rect::new(cx - 2, cy - 3, 3, 2), alt_c);
            fb.fill_rect(Rect::new(cx + 3, cy - 3, 3, 2), alt_c);
            fb.fill_rect(Rect::new(cx - 5, cy + 1, 9, 2), alt_c);
            // 3 Glow rays
            fb.draw_line(cx - 6, cy - 8, cx - 8, cy - 11, primary_c);
            fb.draw_line(cx, cy - 8, cx, cy - 12, primary_c);
            fb.draw_line(cx + 6, cy - 8, cx + 8, cy - 11, primary_c);
        }
        IconKind::MicMute => {
            fb.draw_rounded_rect(Rect::new(cx - 3, cy - 9, 7, 10), 3, primary_c);
            fb.fill_rect(
                Rect::new(cx - 2, cy - 7, 5, 6),
                if is_pressed { Color::BLACK } else { Color::rgb(254, 205, 211) },
            );
            fb.draw_line(cx - 6, cy - 4, cx - 6, cy + 2, primary_c);
            fb.draw_line(cx - 6, cy + 2, cx + 6, cy + 2, primary_c);
            fb.draw_line(cx + 6, cy - 4, cx + 6, cy + 2, primary_c);
            fb.draw_line(cx, cy + 2, cx, cy + 6, primary_c);
            fb.draw_line(cx - 4, cy + 6, cx + 4, cy + 6, primary_c);
            // Distinct diagonal mute slash
            let slash_c = if is_pressed { Color::BLACK } else { Color::rgb(255, 60, 60) };
            fb.draw_line(cx - 7, cy + 5, cx + 7, cy - 8, slash_c);
            fb.draw_line(cx - 7, cy + 6, cx + 7, cy - 7, slash_c);
        }
        IconKind::VolDown => {
            fb.fill_rect(Rect::new(cx - 9, cy - 3, 3, 6), primary_c);
            fb.fill_rect(Rect::new(cx - 6, cy - 4, 2, 8), primary_c);
            fb.fill_rect(Rect::new(cx - 4, cy - 6, 2, 12), primary_c);
            fb.fill_rect(Rect::new(cx - 2, cy - 7, 2, 14), primary_c);
            // 1 sound wave
            fb.draw_line(cx + 3, cy - 3, cx + 5, cy, primary_c);
            fb.draw_line(cx + 5, cy, cx + 3, cy + 3, primary_c);
        }
        IconKind::VolUp => {
            fb.fill_rect(Rect::new(cx - 9, cy - 3, 3, 6), primary_c);
            fb.fill_rect(Rect::new(cx - 6, cy - 4, 2, 8), primary_c);
            fb.fill_rect(Rect::new(cx - 4, cy - 6, 2, 12), primary_c);
            fb.fill_rect(Rect::new(cx - 2, cy - 7, 2, 14), primary_c);
            // Wave 1
            fb.draw_line(cx + 2, cy - 3, cx + 4, cy, primary_c);
            fb.draw_line(cx + 4, cy, cx + 2, cy + 3, primary_c);
            // Wave 2
            fb.draw_line(cx + 6, cy - 6, cx + 9, cy, primary_c);
            fb.draw_line(cx + 9, cy, cx + 6, cy + 6, primary_c);
        }
        IconKind::AudioMute => {
            fb.fill_rect(Rect::new(cx - 9, cy - 3, 3, 6), primary_c);
            fb.fill_rect(Rect::new(cx - 6, cy - 4, 2, 8), primary_c);
            fb.fill_rect(Rect::new(cx - 4, cy - 6, 2, 12), primary_c);
            fb.fill_rect(Rect::new(cx - 2, cy - 7, 2, 14), primary_c);
            // Prominent cross
            let cross_c = if is_pressed { Color::BLACK } else { Color::rgb(233, 213, 255) };
            fb.draw_line(cx + 3, cy - 4, cx + 9, cy + 4, cross_c);
            fb.draw_line(cx + 3, cy - 3, cx + 9, cy + 5, cross_c);
            fb.draw_line(cx + 3, cy + 4, cx + 9, cy - 4, cross_c);
            fb.draw_line(cx + 3, cy + 5, cx + 9, cy - 3, cross_c);
        }
        IconKind::Calc => {
            fb.draw_rounded_rect(Rect::new(cx - 8, cy - 10, 16, 20), 3, primary_c);
            fb.fill_rect(
                Rect::new(cx - 6, cy - 8, 12, 4),
                if is_pressed { Color::rgb(20, 20, 20) } else { Color::rgb(6, 78, 59) },
            );
            fb.draw_line(cx - 4, cy - 6, cx + 3, cy - 6, Color::rgb(110, 231, 183));
            let btn_c = if is_pressed { Color::BLACK } else { Color::rgb(209, 250, 229) };
            fb.fill_rect(Rect::new(cx - 5, cy - 1, 4, 3), btn_c);
            fb.fill_rect(Rect::new(cx + 1, cy - 1, 4, 3), btn_c);
            fb.fill_rect(Rect::new(cx - 5, cy + 4, 4, 3), btn_c);
            fb.fill_rect(Rect::new(cx + 1, cy + 4, 4, 3), btn_c);
        }
        IconKind::Web => {
            fb.draw_circle(cx, cy, 9, primary_c);
            fb.draw_line(cx - 9, cy, cx + 9, cy, primary_c);
            fb.draw_line(cx, cy - 9, cx, cy + 9, primary_c);
            let arc_c = if is_pressed { Color::BLACK } else { Color::rgb(147, 197, 253) };
            fb.draw_line(cx - 4, cy - 7, cx - 4, cy + 7, arc_c);
            fb.draw_line(cx + 4, cy - 7, cx + 4, cy + 7, arc_c);
        }
        IconKind::Files => {
            fb.fill_rect(Rect::new(cx - 9, cy - 9, 7, 3), primary_c);
            fb.draw_rounded_rect(Rect::new(cx - 9, cy - 7, 18, 15), 2, primary_c);
            let flap_c = if is_pressed { Color::BLACK } else { Color::rgb(251, 146, 60) };
            fb.fill_rect(Rect::new(cx - 8, cy - 3, 16, 10), flap_c);
            fb.draw_line(
                cx - 8,
                cy - 3,
                cx + 7,
                cy - 3,
                if is_pressed { Color::WHITE } else { Color::rgb(254, 215, 170) },
            );
        }
        IconKind::Desks => {
            let d_other = if is_pressed { Color::BLACK } else { Color::rgb(147, 51, 234) };
            fb.draw_rounded_rect(Rect::new(cx - 9, cy - 9, 8, 7), 2, primary_c);
            fb.draw_rounded_rect(Rect::new(cx + 1, cy - 9, 8, 7), 2, d_other);
            fb.draw_rounded_rect(Rect::new(cx - 9, cy + 1, 8, 7), 2, d_other);
            fb.draw_rounded_rect(Rect::new(cx + 1, cy + 1, 8, 7), 2, primary_c);
            fb.draw_line(cx - 7, cy - 6, cx - 3, cy - 6, Color::WHITE);
        }
        IconKind::PlayPause => {
            for i in 0..7 {
                fb.draw_line(cx - 8 + i, cy - 5 + i, cx - 8 + i, cy + 5 - i, primary_c);
            }
            fb.fill_rect(Rect::new(cx + 2, cy - 5, 2, 11), primary_c);
            fb.fill_rect(Rect::new(cx + 6, cy - 5, 2, 11), primary_c);
        }
        IconKind::NextTrack => {
            for i in 0..5 {
                fb.draw_line(cx - 7 + i, cy - 4 + i, cx - 7 + i, cy + 4 - i, primary_c);
            }
            for i in 0..5 {
                fb.draw_line(cx - 2 + i, cy - 4 + i, cx - 2 + i, cy + 4 - i, primary_c);
            }
            fb.fill_rect(Rect::new(cx + 5, cy - 5, 2, 11), primary_c);
        }
    }
}

impl Widget for ControlDeckWidget {
    fn render(&self, fb: &mut Framebuffer, area: Rect, font: &FontEngine) {
        fb.fill_rect(area, Color::BLACK);

        let count = BUTTONS.len();
        let gap = 4;
        let total_gaps = (count - 1) as i32 * gap;
        let btn_w = (area.width as i32 - total_gaps) / count as i32;
        let btn_h = area.height.saturating_sub(2);
        let btn_y = area.y + 1;

        for (i, btn) in BUTTONS.iter().enumerate() {
            let btn_x = area.x + i as i32 * (btn_w + gap);
            let btn_rect = Rect::new(btn_x, btn_y, btn_w as u32, btn_h);

            let is_pressed = self.active_button == Some(i);

            let border_color = if is_pressed {
                Color::WHITE
            } else {
                Color::rgba(btn.color.r, btn.color.g, btn.color.b, 150)
            };

            let bg_color = if is_pressed {
                btn.color
            } else {
                Color::rgb(18, 24, 34)
            };

            // Draw rounded bordered card
            fb.draw_rounded_rect_bordered(btn_rect, 6, bg_color, border_color, 1);

            // Subtle vibrant top-edge accent line
            if !is_pressed {
                fb.fill_rect(
                    Rect::new(btn_x + 3, btn_y + 1, btn_w as u32 - 6, 2),
                    btn.color,
                );
            }

            // Draw Colorful Vector Icon
            let cx = btn_x + btn_w / 2;
            let cy = btn_y + 18;
            draw_control_icon(fb, btn.icon, cx, cy, btn.color, is_pressed);

            // Button label in 11.5px bold
            let font_size = 11.5;
            let text_w = font.measure_text_width(btn.label, font_size);
            let text_x = btn_x + (btn_w - text_w as i32) / 2;
            let text_y = btn_y + 47;

            let text_color = if is_pressed {
                Color::BLACK
            } else {
                Color::rgb(240, 243, 246)
            };
            fb.draw_text(font, btn.label, text_x, text_y, font_size, text_color);
        }
    }

    fn handle_touch(&mut self, event: &TouchEvent, area: Rect) -> WidgetAction {
        let count = BUTTONS.len();
        let gap = 4;
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

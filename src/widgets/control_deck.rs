//! Control Deck & System Shortcuts Widget
//! Provides 15 dedicated hardware, system, and desktop shortcuts with realistic, high-fidelity iconography:
//! Esc, Screen Brightness, Keyboard Backlight, Mic Mute, Volume, Mute,
//! Calculator, Web Browser, Files/Nautilus, Desktops/Spaces, and Media.

use crate::graphics::{FontEngine, Framebuffer};
use crate::types::{Color, Rect, TouchEvent, TouchPhase, WidgetAction};
use crate::widgets::Widget;
use image::RgbaImage;

const ICON_ESC: &[u8] = include_bytes!("../assets/icons/esc.png");
const ICON_BRI_DOWN: &[u8] = include_bytes!("../assets/icons/bri_down.png");
const ICON_BRI_UP: &[u8] = include_bytes!("../assets/icons/bri_up.png");
const ICON_KBD_DOWN: &[u8] = include_bytes!("../assets/icons/kbd_down.png");
const ICON_KBD_UP: &[u8] = include_bytes!("../assets/icons/kbd_up.png");
const ICON_MIC: &[u8] = include_bytes!("../assets/icons/mic.png");
const ICON_VOL_DOWN: &[u8] = include_bytes!("../assets/icons/vol_down.png");
const ICON_VOL_UP: &[u8] = include_bytes!("../assets/icons/vol_up.png");
const ICON_MUTE: &[u8] = include_bytes!("../assets/icons/mute.png");
const ICON_CALC: &[u8] = include_bytes!("../assets/icons/calc.png");
const ICON_WEB: &[u8] = include_bytes!("../assets/icons/web.png");
const ICON_FILES: &[u8] = include_bytes!("../assets/icons/files.png");
const ICON_DESKS: &[u8] = include_bytes!("../assets/icons/desks.png");
const ICON_PLAY: &[u8] = include_bytes!("../assets/icons/play.png");
const ICON_NEXT: &[u8] = include_bytes!("../assets/icons/next.png");

pub struct ControlButton {
    pub label: &'static str,
    pub action: WidgetAction,
}

pub struct ControlDeckWidget {
    pub active_button: Option<usize>,
    icons: Vec<RgbaImage>,
}

impl ControlDeckWidget {
    #[must_use]
    pub fn new() -> Self {
        let raw_icons = [
            ICON_ESC,
            ICON_BRI_DOWN,
            ICON_BRI_UP,
            ICON_KBD_DOWN,
            ICON_KBD_UP,
            ICON_MIC,
            ICON_VOL_DOWN,
            ICON_VOL_UP,
            ICON_MUTE,
            ICON_CALC,
            ICON_WEB,
            ICON_FILES,
            ICON_DESKS,
            ICON_PLAY,
            ICON_NEXT,
        ];

        let icons = raw_icons
            .iter()
            .map(|bytes| {
                image::load_from_memory(bytes)
                    .map(|img| img.to_rgba8())
                    .unwrap_or_else(|_| RgbaImage::new(36, 36))
            })
            .collect();

        Self {
            active_button: None,
            icons,
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
        action: WidgetAction::Escape,
    },
    ControlButton {
        label: "BRI -",
        action: WidgetAction::BrightnessDown,
    },
    ControlButton {
        label: "BRI +",
        action: WidgetAction::BrightnessUp,
    },
    ControlButton {
        label: "KBD -",
        action: WidgetAction::KbdBrightnessDown,
    },
    ControlButton {
        label: "KBD +",
        action: WidgetAction::KbdBrightnessUp,
    },
    ControlButton {
        label: "MIC",
        action: WidgetAction::ToggleMicMute,
    },
    ControlButton {
        label: "VOL -",
        action: WidgetAction::VolumeDown,
    },
    ControlButton {
        label: "VOL +",
        action: WidgetAction::VolumeUp,
    },
    ControlButton {
        label: "MUTE",
        action: WidgetAction::ToggleMute,
    },
    ControlButton {
        label: "CALC",
        action: WidgetAction::LaunchCalc,
    },
    ControlButton {
        label: "WEB",
        action: WidgetAction::LaunchBrowser,
    },
    ControlButton {
        label: "FILES",
        action: WidgetAction::LaunchFiles,
    },
    ControlButton {
        label: "DESKS",
        action: WidgetAction::ToggleOverview,
    },
    ControlButton {
        label: "PLAY",
        action: WidgetAction::PlayPause,
    },
    ControlButton {
        label: "NEXT",
        action: WidgetAction::NextTrack,
    },
];

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
                Color::ACCENT
            } else if i == 0 {
                Color::rgb(220, 38, 38)
            } else {
                Color::rgb(85, 105, 135)
            };

            let bg_color = if is_pressed {
                Color::ACCENT
            } else if i == 0 {
                Color::rgb(45, 20, 25)
            } else {
                Color::rgb(22, 28, 40)
            };

            // Draw rounded bordered card with 2px prominent borders matching Fn keys
            fb.draw_rounded_rect_bordered(btn_rect, 6, bg_color, border_color, 2);

            if i == 0 {
                // Esc key: Prominent 22px bold keycap typography vertically centered like in Fn keys
                let font_size = 22.0;
                let text_w = font.measure_text_width("Esc", font_size);
                let text_x = btn_x + (btn_w - text_w as i32) / 2;
                let text_y = btn_y + 38;

                let text_color = if is_pressed {
                    Color::BLACK
                } else {
                    Color::rgb(252, 165, 165)
                };
                fb.draw_text(font, "Esc", text_x, text_y, font_size, text_color);
            } else {
                // Draw Authentic High-Definition Icon (36x36)
                if let Some(icon_img) = self.icons.get(i) {
                    let ix = btn_x + (btn_w - 36) / 2;
                    let iy = btn_y + 2;
                    fb.draw_image(icon_img, ix, iy, 36, 36, 0);
                }

                // Button label in 13.5px bold
                let font_size = 13.5;
                let text_w = font.measure_text_width(btn.label, font_size);
                let text_x = btn_x + (btn_w - text_w as i32) / 2;
                let text_y = btn_y + 51;

                let text_color = if is_pressed {
                    Color::BLACK
                } else {
                    Color::rgb(240, 243, 246)
                };
                fb.draw_text(font, btn.label, text_x, text_y, font_size, text_color);
            }
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

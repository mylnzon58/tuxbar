//! Full-height Function Keys Row (F1 - F12 + Esc)
//! Provides 100% of standard MacBook function keys with large, prominent touch targets (58px tall).

use crate::graphics::{FontEngine, Framebuffer};
use crate::types::{Color, Rect, TouchEvent, TouchPhase, WidgetAction};
use crate::widgets::Widget;
use evdev::Key;

pub struct FnKeysWidget {
    pub active_key: Option<usize>,
}

impl FnKeysWidget {
    #[must_use]
    pub fn new() -> Self {
        Self { active_key: None }
    }

    pub fn get_key_for_index(idx: usize) -> Option<Key> {
        match idx {
            0 => Some(Key::KEY_ESC),
            1 => Some(Key::KEY_F1),
            2 => Some(Key::KEY_F2),
            3 => Some(Key::KEY_F3),
            4 => Some(Key::KEY_F4),
            5 => Some(Key::KEY_F5),
            6 => Some(Key::KEY_F6),
            7 => Some(Key::KEY_F7),
            8 => Some(Key::KEY_F8),
            9 => Some(Key::KEY_F9),
            10 => Some(Key::KEY_F10),
            11 => Some(Key::KEY_F11),
            12 => Some(Key::KEY_F12),
            _ => None,
        }
    }
}

const KEYS: [&str; 13] = [
    "Esc", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12"
];

impl Widget for FnKeysWidget {
    fn render(&self, fb: &mut Framebuffer, area: Rect, font: &FontEngine) {
        fb.fill_rect(area, Color::BLACK);

        let count = KEYS.len();
        let gap = 5;
        let total_gaps = (count - 1) as i32 * gap;
        let btn_w = (area.width as i32 - total_gaps) / count as i32;
        let btn_h = area.height.saturating_sub(2); // 58px tall on 60px bar
        let btn_y = area.y + 1;

        for (i, &key_str) in KEYS.iter().enumerate() {
            let btn_x = area.x + i as i32 * (btn_w + gap);
            let btn_rect = Rect::new(btn_x, btn_y, btn_w as u32, btn_h);

            let is_pressed = self.active_key == Some(i);
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

            fb.draw_rounded_rect_bordered(btn_rect, 6, bg_color, border_color, 2);

            // Large, prominent 22-24px bold typography vertically centered
            let font_size = if i == 0 { 22.0 } else { 24.0 };
            let text_w = font.measure_text_width(key_str, font_size);
            let text_x = btn_x + (btn_w - text_w as i32) / 2;
            let text_y = btn_y + 38;

            let text_color = if is_pressed { 
                Color::BLACK 
            } else if i == 0 { 
                Color::rgb(252, 165, 165) 
            } else { 
                Color::WHITE 
            };
            fb.draw_text(font, key_str, text_x, text_y, font_size, text_color);
        }
    }

    fn handle_touch(&mut self, event: &TouchEvent, area: Rect) -> WidgetAction {
        let count = KEYS.len();
        let gap = 5;
        let total_gaps = (count - 1) as i32 * gap;
        let btn_w = (area.width as i32 - total_gaps) / count as i32;

        let px = event.x as i32;
        let py = event.y as i32;

        if !area.contains(px, py) {
            self.active_key = None;
            return WidgetAction::None;
        }

        match event.phase {
            TouchPhase::Down | TouchPhase::Motion => {
                let offset_x = px - area.x;
                let stride = btn_w + gap;
                let idx = (offset_x / stride) as usize;
                if idx < count {
                    self.active_key = Some(idx);
                }
            }
            TouchPhase::Up => {
                self.active_key = None;
            }
        }

        WidgetAction::None
    }
}

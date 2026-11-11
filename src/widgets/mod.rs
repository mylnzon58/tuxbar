pub mod clock;
pub mod control_deck;
pub mod cyberdeck;
pub mod fn_keys;
pub mod gallery;
pub mod now_playing;
pub mod spectrum;
pub mod system_stats;

use crate::graphics::{FontEngine, Framebuffer};
use crate::types::{Rect, TouchEvent, WidgetAction};

pub trait Widget {
    fn render(&self, fb: &mut Framebuffer, area: Rect, font: &FontEngine);
    fn handle_touch(&mut self, event: &TouchEvent, area: Rect) -> WidgetAction;
}


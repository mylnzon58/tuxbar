pub mod clock;
pub mod now_playing;
pub mod system_stats;

use crate::graphics::{FontEngine, Framebuffer};
use crate::types::{Rect, TouchEvent, WidgetAction};

pub trait Widget {
    fn render(&self, fb: &mut Framebuffer, area: Rect, font: &FontEngine);
    fn handle_touch(&mut self, event: &TouchEvent, area: Rect) -> WidgetAction;
}

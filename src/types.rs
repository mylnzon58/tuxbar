//! Domain types for TuxBar
//! Defines geometry, colors, touch events, and actions.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    #[must_use]
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    #[must_use]
    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x
            && px < self.x + self.width as i32
            && py >= self.y
            && py < self.y + self.height as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[allow(dead_code)]
impl Color {
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    #[must_use]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[must_use]
    pub const fn to_argb_u32(self) -> u32 {
        ((self.a as u32) << 24)
            | ((self.r as u32) << 16)
            | ((self.g as u32) << 8)
            | (self.b as u32)
    }

    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const CYAN: Self = Self::rgb(0, 210, 255);
    pub const GREEN: Self = Self::rgb(34, 197, 94);
    pub const YELLOW: Self = Self::rgb(234, 179, 8);
    pub const RED: Self = Self::rgb(239, 68, 68);
    pub const GRAY: Self = Self::rgb(148, 163, 184);
    pub const DARK_GRAY: Self = Self::rgb(30, 41, 59);
    pub const CARD_BG: Self = Self::rgb(15, 23, 42);
    pub const ACCENT: Self = Self::rgb(59, 130, 246);
    pub const MAGENTA: Self = Self::rgb(217, 70, 239);
    pub const PINK: Self = Self::rgb(244, 63, 94);
    pub const ORANGE: Self = Self::rgb(249, 115, 22);
    pub const PURPLE: Self = Self::rgb(147, 51, 234);
    pub const NEON_CYAN: Self = Self::rgb(0, 240, 255);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarMode {
    #[default]
    ProSplit,
    ControlDeck,
    FnKeys,
    Audiophile,
    Cyberdeck,
    Gallery,
}

impl BarMode {
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::ProSplit => Self::ControlDeck,
            Self::ControlDeck => Self::FnKeys,
            Self::FnKeys => Self::Audiophile,
            Self::Audiophile => Self::Cyberdeck,
            Self::Cyberdeck => Self::Gallery,
            Self::Gallery => Self::ProSplit,
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::ProSplit => "PRO WIDGETS",
            Self::ControlDeck => "CONTROL DECK",
            Self::FnKeys => "F1-F12 KEYS",
            Self::Audiophile => "AUDIOPHILE",
            Self::Cyberdeck => "CYBERDECK",
            Self::Gallery => "GALLERY",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ActiveTouch {
    pub x: i32,
    pub y: i32,
    pub intensity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchPhase {
    Down,
    #[allow(dead_code)]
    Motion,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchEvent {
    pub slot: usize,
    pub x: u32,
    pub y: u32,
    pub phase: TouchPhase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WidgetAction {
    None,
    Escape,
    PlayPause,
    NextTrack,
    PreviousTrack,
    #[allow(dead_code)]
    SeekRatio(u32),
    BrightnessUp,
    BrightnessDown,
    KbdBrightnessUp,
    KbdBrightnessDown,
    VolumeUp,
    VolumeDown,
    ToggleMute,
    ToggleMicMute,
    LaunchCalc,
    LaunchBrowser,
    LaunchFiles,
    ToggleOverview,
    ToggleMode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10, 20, 100, 40);
        assert!(rect.contains(10, 20)); // Top-left corner
        assert!(rect.contains(50, 30)); // Center
        assert!(rect.contains(109, 59)); // Bottom-right inside
        assert!(!rect.contains(9, 20)); // Left outside
        assert!(!rect.contains(10, 19)); // Top outside
        assert!(!rect.contains(110, 40)); // Right boundary outside
        assert!(!rect.contains(50, 60)); // Bottom boundary outside
    }

    #[test]
    fn test_color_argb() {
        let black = Color::BLACK;
        assert_eq!(black.to_argb_u32(), 0xFF000000);

        let white = Color::WHITE;
        assert_eq!(white.to_argb_u32(), 0xFFFFFFFF);

        let red = Color::rgb(255, 0, 0);
        assert_eq!(red.to_argb_u32(), 0xFFFF0000);

        let green_alpha = Color::rgba(0, 255, 0, 128);
        assert_eq!(green_alpha.to_argb_u32(), 0x8000FF00);
    }
}


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
    PlayPause,
    NextTrack,
    PreviousTrack,
    #[allow(dead_code)]
    SeekRatio(u32),
    #[allow(dead_code)]
    BrightnessUp,
    #[allow(dead_code)]
    BrightnessDown,
    #[allow(dead_code)]
    VolumeUp,
    #[allow(dead_code)]
    VolumeDown,
    #[allow(dead_code)]
    ToggleMute,
    ToggleMode,
}

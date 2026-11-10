//! Typography rendering engine powered by pure-Rust fontdue

use fontdue::{Font, FontSettings};

pub struct FontEngine {
    font: Font,
}

impl FontEngine {
    pub fn new() -> Result<Self, String> {
        let font_bytes = include_bytes!("../assets/font.ttf");
        let settings = FontSettings {
            scale: 40.0,
            ..FontSettings::default()
        };
        let font = Font::from_bytes(font_bytes as &[u8], settings)
            .map_err(|e| format!("Failed to parse embedded TrueType font: {e}"))?;
        Ok(Self { font })
    }

    #[must_use]
    pub fn rasterize(&self, c: char, size_px: f32) -> (fontdue::Metrics, Vec<u8>) {
        self.font.rasterize(c, size_px)
    }

    #[must_use]
    pub fn measure_text_width(&self, text: &str, size_px: f32) -> u32 {
        let mut total_width = 0.0f32;
        for c in text.chars() {
            let metrics = self.font.metrics(c, size_px);
            total_width += metrics.advance_width;
        }
        total_width.ceil() as u32
    }
}

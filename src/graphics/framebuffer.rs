//! Pure-Rust memory-safe software framebuffer for the Touch Bar
//! Resolution: 2008x60 (ARGB8888 32-bit per pixel)

use crate::graphics::font::FontEngine;
use crate::types::{Color, Rect};

pub const TOUCHBAR_WIDTH: u32 = 2008;
pub const TOUCHBAR_HEIGHT: u32 = 60;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

impl Framebuffer {
    #[must_use]
    pub fn new() -> Self {
        let size = (TOUCHBAR_WIDTH * TOUCHBAR_HEIGHT) as usize;
        Self {
            width: TOUCHBAR_WIDTH,
            height: TOUCHBAR_HEIGHT,
            pixels: vec![0; size],
        }
    }

    pub fn clear(&mut self, color: Color) {
        let val = color.to_argb_u32();
        self.pixels.fill(val);
    }

    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let idx = (y as u32 * self.width + x as u32) as usize;
            if color.a == 255 {
                self.pixels[idx] = color.to_argb_u32();
            } else if color.a > 0 {
                // Alpha blend over background
                let bg = self.pixels[idx];
                let bg_r = (bg >> 16) & 0xFF;
                let bg_g = (bg >> 8) & 0xFF;
                let bg_b = bg & 0xFF;

                let a = color.a as u32;
                let inv_a = 255 - a;

                let r = ((color.r as u32 * a + bg_r * inv_a) / 255) as u8;
                let g = ((color.g as u32 * a + bg_g * inv_a) / 255) as u8;
                let b = ((color.b as u32 * a + bg_b * inv_a) / 255) as u8;

                self.pixels[idx] = Color::rgb(r, g, b).to_argb_u32();
            }
        }
    }

    #[allow(dead_code)]
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let x_start = rect.x.max(0);
        let x_end = (rect.x + rect.width as i32).min(self.width as i32);
        let y_start = rect.y.max(0);
        let y_end = (rect.y + rect.height as i32).min(self.height as i32);

        for y in y_start..y_end {
            for x in x_start..x_end {
                self.set_pixel(x, y, color);
            }
        }
    }

    pub fn draw_rounded_rect(&mut self, rect: Rect, radius: i32, color: Color) {
        let x_start = rect.x.max(0);
        let x_end = (rect.x + rect.width as i32).min(self.width as i32);
        let y_start = rect.y.max(0);
        let y_end = (rect.y + rect.height as i32).min(self.height as i32);

        let r2 = radius * radius;

        for y in y_start..y_end {
            for x in x_start..x_end {
                let dx_left = x - (rect.x + radius);
                let dx_right = x - (rect.x + rect.width as i32 - 1 - radius);
                let dy_top = y - (rect.y + radius);
                let dy_bottom = y - (rect.y + rect.height as i32 - 1 - radius);

                let is_corner = (dx_left < 0 && dy_top < 0 && (dx_left * dx_left + dy_top * dy_top > r2))
                    || (dx_right > 0 && dy_top < 0 && (dx_right * dx_right + dy_top * dy_top > r2))
                    || (dx_left < 0 && dy_bottom > 0 && (dx_left * dx_left + dy_bottom * dy_bottom > r2))
                    || (dx_right > 0 && dy_bottom > 0 && (dx_right * dx_right + dy_bottom * dy_bottom > r2));

                if !is_corner {
                    self.set_pixel(x, y, color);
                }
            }
        }
    }

    pub fn draw_progress_bar(&mut self, rect: Rect, ratio: f32, bg: Color, fg: Color) {
        self.draw_rounded_rect(rect, 3, bg);
        let fill_w = ((rect.width as f32) * ratio.clamp(0.0, 1.0)).round() as u32;
        if fill_w > 0 {
            let active_rect = Rect::new(rect.x, rect.y, fill_w, rect.height);
            self.draw_rounded_rect(active_rect, 3, fg);
        }
    }

    pub fn draw_text(
        &mut self,
        font: &FontEngine,
        text: &str,
        mut x: i32,
        base_y: i32,
        size_px: f32,
        color: Color,
    ) {
        for c in text.chars() {
            let (metrics, bitmap) = font.rasterize(c, size_px);
            let glyph_x = x + metrics.xmin;
            let glyph_y = base_y - metrics.height as i32 - metrics.ymin;

            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    let alpha = bitmap[row * metrics.width + col];
                    if alpha > 0 {
                        let blended_alpha = ((color.a as u32 * alpha as u32) / 255) as u8;
                        let text_color = Color::rgba(color.r, color.g, color.b, blended_alpha);
                        self.set_pixel(glyph_x + col as i32, glyph_y + row as i32, text_color);
                    }
                }
            }
            x += metrics.advance_width.round() as i32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framebuffer_init() {
        let fb = Framebuffer::new();
        assert_eq!(fb.width, TOUCHBAR_WIDTH);
        assert_eq!(fb.height, TOUCHBAR_HEIGHT);
        assert_eq!(fb.pixels.len(), (TOUCHBAR_WIDTH * TOUCHBAR_HEIGHT) as usize);
        assert!(fb.pixels.iter().all(|&p| p == 0));
    }

    #[test]
    fn test_framebuffer_clear() {
        let mut fb = Framebuffer::new();
        fb.clear(Color::rgb(255, 0, 0));
        assert!(fb.pixels.iter().all(|&p| p == 0xFFFF0000));
    }

    #[test]
    fn test_framebuffer_set_pixel_bounds() {
        let mut fb = Framebuffer::new();
        // Out of bounds should not panic or corrupt memory
        fb.set_pixel(-1, 0, Color::WHITE);
        fb.set_pixel(0, -1, Color::WHITE);
        fb.set_pixel(TOUCHBAR_WIDTH as i32, 0, Color::WHITE);
        fb.set_pixel(0, TOUCHBAR_HEIGHT as i32, Color::WHITE);
        assert!(fb.pixels.iter().all(|&p| p == 0));

        // In bounds pixel
        fb.set_pixel(10, 10, Color::CYAN);
        let idx = (10 * TOUCHBAR_WIDTH + 10) as usize;
        assert_eq!(fb.pixels[idx], Color::CYAN.to_argb_u32());
    }

    #[test]
    fn test_framebuffer_fill_rect() {
        let mut fb = Framebuffer::new();
        let rect = Rect::new(5, 5, 10, 10);
        fb.fill_rect(rect, Color::GREEN);

        // Check corner pixel
        let idx = (5 * TOUCHBAR_WIDTH + 5) as usize;
        assert_eq!(fb.pixels[idx], Color::GREEN.to_argb_u32());

        // Check outside rect
        let outside_idx = (4 * TOUCHBAR_WIDTH + 5) as usize;
        assert_eq!(fb.pixels[outside_idx], 0);
    }

    #[test]
    fn test_framebuffer_progress_bar() {
        let mut fb = Framebuffer::new();
        let rect = Rect::new(0, 0, 100, 10);
        fb.draw_progress_bar(rect, 0.5, Color::DARK_GRAY, Color::CYAN);

        // Midpoint should be cyan
        let mid_idx = (5 * TOUCHBAR_WIDTH + 25) as usize;
        assert_ne!(fb.pixels[mid_idx], 0);
    }
}


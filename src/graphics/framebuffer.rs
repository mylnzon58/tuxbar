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

    pub fn draw_rounded_rect_bordered(
        &mut self,
        rect: Rect,
        radius: i32,
        bg: Color,
        border: Color,
        border_width: u32,
    ) {
        self.draw_rounded_rect(rect, radius, border);
        let bw = border_width as i32;
        if rect.width > border_width * 2 && rect.height > border_width * 2 {
            let inner_rect = Rect::new(
                rect.x + bw,
                rect.y + bw,
                rect.width - border_width * 2,
                rect.height - border_width * 2,
            );
            let inner_radius = (radius - bw).max(1);
            self.draw_rounded_rect(inner_rect, inner_radius, bg);
        }
    }

    #[allow(dead_code)]
    pub fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.set_pixel(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    #[allow(dead_code)]
    pub fn draw_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        let mut x = radius;
        let mut y = 0;
        let mut err = 0;

        while x >= y {
            self.set_pixel(cx + x, cy + y, color);
            self.set_pixel(cx + y, cy + x, color);
            self.set_pixel(cx - y, cy + x, color);
            self.set_pixel(cx - x, cy + y, color);
            self.set_pixel(cx - x, cy - y, color);
            self.set_pixel(cx - y, cy - x, color);
            self.set_pixel(cx + y, cy - x, color);
            self.set_pixel(cx + x, cy - y, color);

            y += 1;
            err += 1 + 2 * y;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }

    #[allow(dead_code)]
    pub fn fill_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        let r2 = radius * radius;
        let x_start = (cx - radius).max(0);
        let x_end = (cx + radius).min(self.width as i32 - 1);
        let y_start = (cy - radius).max(0);
        let y_end = (cy + radius).min(self.height as i32 - 1);

        for y in y_start..=y_end {
            let dy = y - cy;
            for x in x_start..=x_end {
                let dx = x - cx;
                if dx * dx + dy * dy <= r2 {
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

    pub fn draw_text_clipped(
        &mut self,
        font: &FontEngine,
        text: &str,
        mut x: i32,
        base_y: i32,
        size_px: f32,
        color: Color,
        max_x: i32,
    ) {
        for c in text.chars() {
            if x >= max_x {
                break;
            }
            let (metrics, bitmap) = font.rasterize(c, size_px);
            let glyph_x = x + metrics.xmin;
            let glyph_y = base_y - metrics.height as i32 - metrics.ymin;

            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    let px = glyph_x + col as i32;
                    let py = glyph_y + row as i32;
                    if px >= max_x || px < 0 {
                        continue;
                    }
                    let alpha = bitmap[row * metrics.width + col];
                    if alpha > 0 {
                        let blended_alpha = ((color.a as u32 * alpha as u32) / 255) as u8;
                        let text_color = Color::rgba(color.r, color.g, color.b, blended_alpha);
                        self.set_pixel(px, py, text_color);
                    }
                }
            }
            x += metrics.advance_width.round() as i32;
        }
    }

    pub fn draw_glow_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        let r2 = radius * radius;
        let x_start = (cx - radius).max(0);
        let x_end = (cx + radius).min(self.width as i32);
        let y_start = (cy - radius).max(0);
        let y_end = (cy + radius).min(self.height as i32);

        for y in y_start..y_end {
            let dy = y - cy;
            for x in x_start..x_end {
                let dx = x - cx;
                let dist2 = dx * dx + dy * dy;
                if dist2 <= r2 {
                    let factor = 1.0 - (dist2 as f32 / r2 as f32).sqrt();
                    let alpha = ((color.a as f32) * factor).round() as u8;
                    if alpha > 0 {
                        self.set_pixel(x, y, Color::rgba(color.r, color.g, color.b, alpha));
                    }
                }
            }
        }
    }

    pub fn draw_image(
        &mut self,
        img: &image::RgbaImage,
        dest_x: i32,
        dest_y: i32,
        target_w: u32,
        target_h: u32,
        radius: i32,
    ) {
        let src_w = img.width();
        let src_h = img.height();
        if src_w == 0 || src_h == 0 || target_w == 0 || target_h == 0 {
            return;
        }

        let r2 = radius * radius;

        for ty in 0..target_h {
            let dy = dest_y + ty as i32;
            if dy < 0 || dy >= self.height as i32 {
                continue;
            }
            let sy = (ty * src_h) / target_h;

            for tx in 0..target_w {
                let dx = dest_x + tx as i32;
                if dx < 0 || dx >= self.width as i32 {
                    continue;
                }

                if radius > 0 {
                    let dx_left = tx as i32 - radius;
                    let dx_right = tx as i32 - (target_w as i32 - 1 - radius);
                    let dy_top = ty as i32 - radius;
                    let dy_bottom = ty as i32 - (target_h as i32 - 1 - radius);

                    let is_corner = (dx_left < 0 && dy_top < 0 && (dx_left * dx_left + dy_top * dy_top > r2))
                        || (dx_right > 0 && dy_top < 0 && (dx_right * dx_right + dy_top * dy_top > r2))
                        || (dx_left < 0 && dy_bottom > 0 && (dx_left * dx_left + dy_bottom * dy_bottom > r2))
                        || (dx_right > 0 && dy_bottom > 0 && (dx_right * dx_right + dy_bottom * dy_bottom > r2));

                    if is_corner {
                        continue;
                    }
                }

                let sx = (tx * src_w) / target_w;
                let pixel = img.get_pixel(sx, sy);
                let color = Color::rgba(pixel[0], pixel[1], pixel[2], pixel[3]);
                self.set_pixel(dx, dy, color);
            }
        }
    }

    pub fn save_ppm<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        writeln!(file, "P6\n{} {}\n255", self.width, self.height)?;
        let mut buffer = Vec::with_capacity((self.width * self.height * 3) as usize);
        for &pixel in &self.pixels {
            let r = ((pixel >> 16) & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let b = (pixel & 0xFF) as u8;
            buffer.extend_from_slice(&[r, g, b]);
        }
        file.write_all(&buffer)?;
        Ok(())
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


//! Software painter for the display list.

use crate::text::Fonts;
use dlbronz_web::css::Color;
use dlbronz_web::layout::Item;
use dlbronz_web::render::Page;

pub struct Surface {
    pub w: usize,
    pub h: usize,
    pub buf: Vec<u32>, // 0x00RRGGBB
}

impl Surface {
    pub fn new(w: usize, h: usize, bg: u32) -> Surface {
        Surface { w, h, buf: vec![bg; w * h] }
    }

    #[inline]
    fn blend(&mut self, x: i32, y: i32, c: Color, coverage: f32) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        let a = (c.a as f32 / 255.0) * coverage;
        if a <= 0.0 {
            return;
        }
        let i = y as usize * self.w + x as usize;
        let d = self.buf[i];
        let (dr, dg, db) = ((d >> 16) & 0xff, (d >> 8) & 0xff, d & 0xff);
        let mix = |s: u8, d: u32| -> u32 { (s as f32 * a + d as f32 * (1.0 - a)).round().clamp(0.0, 255.0) as u32 };
        self.buf[i] = (mix(c.r, dr) << 16) | (mix(c.g, dg) << 8) | mix(c.b, db);
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Color) {
        let x0 = x.round() as i32;
        let y0 = y.round() as i32;
        let x1 = (x + w).round() as i32;
        let y1 = (y + h).round() as i32;
        for yy in y0.max(0)..y1.min(self.h as i32) {
            for xx in x0.max(0)..x1.min(self.w as i32) {
                self.blend(xx, yy, c, 1.0);
            }
        }
    }
}

pub fn pack(c: Color) -> u32 {
    ((c.r as u32) << 16) | ((c.g as u32) << 8) | c.b as u32
}

/// Paints `page` shifted by (`dy`). Only items intersecting the surface are drawn.
pub fn paint(surface: &mut Surface, page: &Page, fonts: &mut Fonts, dy: f32) {
    let sh = surface.h as f32;
    for item in &page.items {
        match item {
            Item::Rect { x, y, w, h, color } => {
                let ry = y + dy;
                if ry + h < 0.0 || ry > sh {
                    continue;
                }
                surface.fill_rect(*x, ry, *w, *h, *color);
            }
            Item::Text { x, y, w, size, bold, underline, mono, color, text, .. } => {
                let top = y + dy;
                if top + size * 1.2 < 0.0 || top > sh {
                    continue;
                }
                let (asc, desc) = fonts.line(*size, *bold, *mono);
                let baseline = top + (size * 1.2 - (asc - desc)) / 2.0 + asc;
                let mut pen = *x;
                for ch in text.chars() {
                    let (m, bitmap) = {
                        let g = fonts.glyph(ch, *size, *bold, *mono);
                        (g.0, g.1.clone())
                    };
                    let gx = (pen + m.xmin as f32).round() as i32;
                    let gy = (baseline - m.height as f32 - m.ymin as f32).round() as i32;
                    for row in 0..m.height {
                        for col in 0..m.width {
                            let cov = bitmap[row * m.width + col] as f32 / 255.0;
                            if cov > 0.0 {
                                surface.blend(gx + col as i32, gy + row as i32, *color, cov);
                            }
                        }
                    }
                    pen += m.advance_width;
                }
                if *underline {
                    surface.fill_rect(*x, baseline + 1.5, *w, (size / 16.0).max(1.0), *color);
                }
            }
        }
    }
}

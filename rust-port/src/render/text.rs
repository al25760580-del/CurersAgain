//! Original font atlas rendering. No host font substitution or full Scribble implementation.
use super::sprites::{num, texture};
use sdl2::{
    pixels::Color,
    rect::{FRect, Rect},
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
};
use serde_json::Value;
use std::{collections::HashMap, path::Path};
pub struct Font<'a> {
    pub tex: Texture<'a>,
    glyphs: HashMap<char, (Rect, i32, i32)>,
    fallback: Option<Box<Font<'a>>>,
    pub init_baseline: f32,
}
impl<'a> Font<'a> {
    /// Upload an exported font atlas and cache glyph rectangles, shifts and x offsets.
    /// Metrics are original font pixels, not host-platform font rasterization.
    pub fn load(
        tc: &'a TextureCreator<WindowContext>,
        root: &Path,
        data: &Value,
    ) -> Result<Self, String> {
        let name = data["name"].as_str().unwrap();
        let tex = texture(tc, &root.join("fonts").join(format!("{name}.png")), None)?;
        let mut glyphs = HashMap::new();
        for item in data["glyphs"].as_array().unwrap() {
            let g = &item["properties"];
            if let Some(ch) = char::from_u32(num(g, "Character") as u32) {
                glyphs.insert(
                    ch,
                    (
                        Rect::new(
                            num(g, "SourceX") as i32,
                            num(g, "SourceY") as i32,
                            num(g, "SourceWidth") as u32,
                            num(g, "SourceHeight") as u32,
                        ),
                        num(g, "Shift") as i32,
                        num(g, "Offset") as i32,
                    ),
                );
            }
        }
        Ok(Self {
            tex,
            glyphs,
            fallback: None,
            init_baseline: -1.,
        })
    }
    pub fn load_language(
        tc: &'a TextureCreator<WindowContext>,
        source: &crate::language::FontSource,
    ) -> Result<Self, String> {
        let data: crate::language::GlyphFile = crate::language::parse(&source.metrics)?;
        if data.schema != 1
            || data.format != "bitmap-glyphs"
            || !data.init_baseline.is_finite()
            || data.init_baseline.abs() > 64.
            || data.glyphs.len() > 65536
        {
            return Err("Invalid glyph header".into());
        }
        let dims = image::image_dimensions(&source.atlas).map_err(|e| e.to_string())?;
        if dims != (data.atlas_width, data.atlas_height) || dims.0 > 4096 || dims.1 > 4096 {
            return Err("Invalid or oversized glyph atlas".into());
        }
        let mut glyphs = HashMap::new();
        for (ch, [x, y, w, h, advance, offset]) in data.glyphs {
            let mut chars = ch.chars();
            let code = chars.next().ok_or("Empty glyph key")?;
            if chars.next().is_some()
                || [x, y, w, h].iter().any(|n| *n < 0)
                || x as i64 + w as i64 > dims.0 as i64
                || y as i64 + h as i64 > dims.1 as i64
                || !(0..=512).contains(&advance)
                || offset.unsigned_abs() > 512
            {
                return Err(format!("Invalid glyph metrics: {ch}"));
            }
            glyphs.insert(code, (Rect::new(x, y, w as u32, h as u32), advance, offset));
        }
        let tex = texture(tc, &source.atlas, None)?;
        let fallback = source
            .fallback
            .as_ref()
            .map(|f| Self::load_language(tc, f).map(Box::new))
            .transpose()?;
        Ok(Self {
            tex,
            glyphs,
            fallback,
            init_baseline: data.init_baseline,
        })
    }
    fn advance(&self, ch: char) -> i32 {
        self.glyphs
            .get(&ch)
            .map(|g| g.1)
            .unwrap_or_else(|| self.fallback.as_ref().map(|f| f.advance(ch)).unwrap_or(0))
    }
    pub fn width(&self, text: &str) -> i32 {
        text.chars().map(|ch| self.advance(ch)).sum()
    }
    fn glyph_extent(&self, ch: char) -> (i32, i32, i32) {
        if let Some((r, advance, offset)) = self.glyphs.get(&ch) {
            (*advance, *offset, r.width() as i32)
        } else {
            self.fallback
                .as_ref()
                .map(|f| f.glyph_extent(ch))
                .unwrap_or((0, 0, 0))
        }
    }
    /// Includes advance and bitmap overhang, in logical pixels; same fallback as drawing.
    pub fn visual_width(&self, text: &str) -> u32 {
        let (mut pen, mut left, mut right) = (0i64, 0i64, 0i64);
        for ch in text.chars() {
            let (advance, offset, width) = self.glyph_extent(ch);
            if width > 0 {
                left = left.min(pen + offset as i64);
                right = right.max(pen + offset as i64 + width as i64);
            }
            pen += advance as i64;
            right = right.max(pen);
        }
        (right - left).clamp(0, u32::MAX as i64) as u32
    }
    /// Draw one atlas-text line with left/center/right alignment (0/1/2).
    /// Preserves texture alpha for intro fades. No word wrap, shaping or Scribble effects.
    pub fn text(
        &mut self,
        c: &mut Canvas<Window>,
        text: &str,
        x: f32,
        y: f32,
        align: u8,
        col: Color,
    ) -> Result<(), String> {
        let width = self.width(text);
        let mut x = x - if align == 1 {
            width as f32 / 2.0
        } else if align == 2 {
            width as f32
        } else {
            0.0
        };
        self.tex.set_color_mod(col.r, col.g, col.b);
        for ch in text.chars() {
            if let Some((r, shift, offset)) = self.glyphs.get(&ch) {
                if r.width() > 0 && r.height() > 0 {
                    c.copy_f(
                        &self.tex,
                        *r,
                        FRect::new(x + *offset as f32, y, r.width() as f32, r.height() as f32),
                    )?;
                }
                x += *shift as f32;
            } else if let Some(fallback) = self.fallback.as_mut() {
                fallback.tex.set_alpha_mod(self.tex.alpha_mod());
                fallback.text(c, &ch.to_string(), x, y, 0, col)?;
                x += fallback.advance(ch) as f32;
            }
        }
        Ok(())
    }
}

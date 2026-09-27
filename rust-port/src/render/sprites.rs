//! SDL sprite resources and unrotated drawing, extracted without changing formulas.
use sdl2::{
    pixels::PixelFormatEnum,
    rect::FRect,
    render::{BlendMode, Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
};
use serde_json::Value;
use std::path::Path;
pub struct Sprite<'a> {
    pub tex: Texture<'a>,
    pub w: u32,
    pub h: u32,
    pub ox: f32,
    pub oy: f32,
}
/// Read an extracted numeric JSON field; missing/non-numeric values currently default to zero.
/// Loader utility, not original game logic. Validate new metadata before relying on this fallback.
pub fn num(v: &Value, k: &str) -> f64 {
    v[k].as_f64().unwrap_or(0.0)
}
/// Upload extracted RGBA pixels to SDL; optional tint preserves only source alpha.
/// The tint approximates the locked-character shader, not a recovered GPU program.
/// No game files are modified; errors name the missing/unreadable asset.
pub fn texture<'a>(
    tc: &'a TextureCreator<WindowContext>,
    p: &Path,
    fog: Option<u32>,
) -> Result<Texture<'a>, String> {
    let mut image = image::open(p)
        .map_err(|e| format!("{}: {e}", p.display()))?
        .to_rgba8();
    if let Some(c) = fog {
        for p in image.pixels_mut() {
            p[0] = (c & 255) as u8;
            p[1] = ((c >> 8) & 255) as u8;
            p[2] = ((c >> 16) & 255) as u8;
        }
    }
    let (w, h) = image.dimensions();
    let mut t = tc
        .create_texture_static(PixelFormatEnum::RGBA32, w, h)
        .map_err(|e| e.to_string())?;
    t.update(None, &image, w as usize * 4)
        .map_err(|e| e.to_string())?;
    t.set_blend_mode(BlendMode::Blend);
    Ok(t)
}
/// Load one exported frame and its GameMaker origin (logical sprite pixels).
/// Metadata discovery is separate from YYC code recovery; this function does not parse data.win.
pub fn load_sprite<'a>(
    tc: &'a TextureCreator<WindowContext>,
    root: &Path,
    name: &str,
    frame: usize,
    meta: &Value,
    fog: Option<u32>,
) -> Result<Sprite<'a>, String> {
    let tex = texture(
        tc,
        &root
            .join("sprites")
            .join(name)
            .join(format!("{name}_{frame}.png")),
        fog,
    )?;
    let q = tex.query();
    let p = &meta["properties"];
    Ok(Sprite {
        tex,
        w: q.width,
        h: q.height,
        ox: num(p, "OriginXWrapper") as f32,
        oy: num(p, "OriginYWrapper") as f32,
    })
}
/// Place a sprite using its resource origin and explicit x/y scale in current canvas units.
/// Scope: unrotated sprites; particles have a separate rotation path. SDL errors propagate.
pub fn draw(
    c: &mut Canvas<Window>,
    s: &Sprite,
    x: f32,
    y: f32,
    sx: f32,
    sy: f32,
) -> Result<(), String> {
    c.copy_f(
        &s.tex,
        None,
        FRect::new(
            x - s.ox * sx,
            y - s.oy * sy,
            s.w as f32 * sx,
            s.h as f32 * sy,
        ),
    )
}

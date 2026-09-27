//! OS color cursor backed by the extracted Steam sprite (not a recreated arrow).
//! Presentation only: its hotspot does not change title hit testing.
//! InputManager Step 0x1425198f0 writes cursor_sprite=asset 1698 (tagged 0x10006a2).
//! Current port policy: resource size/origin, static OS cursor. Native idle hiding
//! (mouseMovedTime=180), hideMouse and gameplay cursor 1699 are not ported yet.
//! Native uses GameMaker cursor_sprite; OS rendering is a responsiveness choice,
//! not a claim of identical scaling, screenshot inclusion or compositor latency.
use sdl2::{mouse::Cursor, pixels::PixelFormatEnum, surface::Surface};
use std::path::Path;

/// Keep this Cursor alive while installed. SDL copies surface pixels at creation;
/// the OS cursor remains responsive when the renderer is busy and stays out of screenshots.
pub fn load(root: &Path) -> Result<Cursor, String> {
    let path = root.join("sprites/spr_GameCursor/spr_GameCursor_0.png");
    let mut rgba = image::open(&path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .to_rgba8();
    let (w, h) = rgba.dimensions();
    // Resource 1698: 48x50, origin (0,0), exactly one frame.
    if (w, h) != (48, 50) {
        return Err(format!("Unexpected cursor dimensions: {w}x{h}"));
    }
    let hotspot = validate_hotspot((w, h), (0, 0))?;
    let surface = Surface::from_data(rgba.as_mut(), w, h, w * 4, PixelFormatEnum::RGBA32)?;
    Cursor::from_surface(&surface, hotspot.0, hotspot.1)
}
fn validate_hotspot(size: (u32, u32), hotspot: (i32, i32)) -> Result<(i32, i32), String> {
    if hotspot.0 < 0 || hotspot.1 < 0 || hotspot.0 as u32 >= size.0 || hotspot.1 as u32 >= size.1 {
        return Err("Cursor hotspot outside image".into());
    }
    Ok(hotspot)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_origin_is_valid() {
        assert_eq!(validate_hotspot((48, 50), (0, 0)).unwrap(), (0, 0));
    }
    #[test]
    fn invalid_origins_are_rejected() {
        for h in [(-1, 0), (0, -1), (48, 0), (0, 50)] {
            assert!(validate_hotspot((48, 50), h).is_err());
        }
        assert!(validate_hotspot((0, 0), (0, 0)).is_err());
    }
}

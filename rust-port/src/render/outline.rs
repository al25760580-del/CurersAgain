//! Original single-line title prompt outline, not a post-process shader.
//!
//! Evidence: `gml_Script_draw_text_outline` at 0x140547f30, called by
//! `commandPromps` at 0x14053dcf0; TitleLab session 20260927T052024Z,
//! frame 2880: 16 black offset draws followed by one white draw.
//! Coordinates/radius are logical room pixels (640×360), not window pixels.
//! This module does NOT implement the generic helper's multiline wrapping.

/// Return every original outline pass, including duplicate rounded offsets.
/// The YYC loop runs angle=45 while angle<405, advancing 360/samples degrees.
/// GameMaker lengthdir_y has the opposite sign to mathematical sin (Y down).
/// Radius=1, samples=16 is verified for the keyboard confirm/cancel prompt.
pub fn outline_offsets(radius: f64, samples: u32) -> Vec<(f32, f32)> {
    assert!(
        samples > 0 && samples <= 4096,
        "outline sample count must be bounded"
    );
    (0..samples)
        .map(|i| {
            let angle = (45.0 + i as f64 * 360.0 / samples as f64).to_radians();
            (
                (radius * angle.cos()).round() as f32,
                (-radius * angle.sin()).round() as f32,
            )
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    /// Exact sequence captured at draw_text_ext_color, relative to (630,345).
    #[test]
    fn matches_native_sixteen_outline_passes() {
        assert_eq!(
            outline_offsets(1.0, 16),
            vec![
                (1., -1.),
                (0., -1.),
                (0., -1.),
                (0., -1.),
                (-1., -1.),
                (-1., 0.),
                (-1., 0.),
                (-1., 0.),
                (-1., 1.),
                (0., 1.),
                (0., 1.),
                (0., 1.),
                (1., 1.),
                (1., 0.),
                (1., 0.),
                (1., 0.)
            ]
        );
    }
    #[test]
    fn zero_radius_keeps_pass_count() {
        assert_eq!(outline_offsets(0., 16), vec![(0., 0.); 16]);
    }
}

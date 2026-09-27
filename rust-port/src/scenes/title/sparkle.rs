//! Verified stationary portrait-sparkle specialization of obj_vfx, NOT all generic VFX.
//! Caller: obj_TitleCharacter.Draw 0x1435b94f0; VFX Create 0x143626310,
//! Step 0x143626fc0, Draw 0x1436276b0, Animation End 0x1436274f0.
//! Resource 2574: three frames, playback 25 FPS, origin (25,25), 50×50 pixels.
//! Runtime confirms x/y fixed, scale 1, alpha 1, image_speed 1, duration 0.
//! Animation End destroys when sprite_index > 0 and duration == 0.
//! Other branches (follow, duration, additive, damping, growth) are not implemented.
#[derive(Debug, Clone)]
pub struct PortraitSpark {
    pub x: f32,
    pub y: f32,
    pub image_index: f32,
}
impl PortraitSpark {
    /// Called after the owning portrait's draw-time emission; x/y are logical pixels.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            image_index: 0.0,
        }
    }
    /// Advance builtin image_index in float32 at 60 Hz; false means Animation End destroys it.
    pub fn step(&mut self) -> bool {
        self.image_index += 25.0f32 / 60.0;
        self.image_index < 3.0
    }
    /// The atlas frame is discrete; interpolation must not invent intermediate images.
    pub fn frame(&self) -> usize {
        self.image_index.floor() as usize
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn matches_all_five_observed_sparkle_animations() {
        let rows: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/portrait-sparkles.json"
        )))
        .unwrap();
        let mut models = HashMap::new();
        let mut counts = HashMap::new();
        for r in rows.as_array().unwrap() {
            let id = r["id"].as_u64().unwrap();
            let s = models
                .entry(id)
                .or_insert_with(|| PortraitSpark::new(0.0, 0.0));
            assert_eq!(s.image_index as f64, r["image_index"].as_f64().unwrap());
            s.step();
            *counts.entry(id).or_insert(0) += 1;
        }
        assert_eq!(models.len(), 5);
        for (id, s) in models {
            assert_eq!(counts[&id], 8);
            assert!(s.image_index >= 3.0);
        }
    }
}

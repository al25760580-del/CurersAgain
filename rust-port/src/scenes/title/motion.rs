//! Formulas recovered from obj_TitleCharacter_Step_0 and obj_TitleScreen_Step_0.
//! Scope: title bobbing and character entrance. No legend sparkle, fog or particles.
//! Character Create 0x1435b80e0; Step 0x1435b8ea0; Draw 0x1435b94f0.
//! TitleScreen Create 0x1435bd370; Step 0x1435c07d0.
//! Builtin y is float32 in the captured runner; custom titleY/lifetime remain float64.
#[derive(Clone, Copy, Debug)]
pub struct Bob {
    pub phase: f64,
    pub y: f64,
    pub increment: f64,
}
impl Bob {
    /// Advance a custom double-precision oscillator; phase is radians, y is logical pixels.
    /// Character callers must use step_character to additionally round builtin y to float32.
    pub fn step(&mut self) {
        self.phase += self.increment;
        if self.phase > std::f64::consts::TAU {
            self.phase = 0.0;
        }
        self.y += self.phase.sin() / 4.0;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recovered_character_motion_matches_47_runtime_samples() {
        let data: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/title-manifest.json"
        )))
        .unwrap();
        let chars = data["characters"].as_array().unwrap();
        assert_eq!(chars.len(), 47);
        for c in chars {
            let mut b = Bob {
                phase: c["phase"].as_f64().unwrap(),
                y: c["y"].as_f64().unwrap(),
                increment: std::f64::consts::PI / 60.0,
            };
            for _ in 0..180 {
                b.step();
            }
            assert!(
                (b.y - c["expected_y_180"].as_f64().unwrap()).abs() < 0.0001,
                "{}",
                c["name"]
            );
            assert!((b.phase - c["expected_phase_180"].as_f64().unwrap()).abs() < 1e-10);
        }
    }
    #[test]
    fn recovered_logo_motion_matches_runtime_sample() {
        let d: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/title-manifest.json"
        )))
        .unwrap();
        let mut b = Bob {
            phase: d["title"]["lifetime"].as_f64().unwrap(),
            y: d["title"]["titleY"].as_f64().unwrap(),
            increment: std::f64::consts::PI / 120.0,
        };
        for _ in 0..180 {
            b.step();
        }
        assert!((b.y - d["title_expected_180"]["titleY"].as_f64().unwrap()).abs() < 1e-9);
    }
}

/// obj_TitleCharacter.Step: advance phase and accumulate into GameMaker's builtin y.
/// Units: phase radians, y logical pixels; one invocation per 60-Hz simulation step.
/// Unlike custom titleY, builtin y is stored as float32: omitting this rounding drifts.
/// Scope excludes sprite animation, entrance offset and legend spark cooldown.
pub fn step_character(bob: &mut Bob) {
    bob.step();
    bob.y = bob.y as f32 as f64;
}
/// obj_TitleCharacter.Create/Step entrance: startY=-300; delay=floor(y/20).
/// Step checks startY before decrementing delay, multiplying it by 0.8 only at zero.
/// YYC's equality test treats abs(offset)<=1e-5 as zero; preserve that residual.
/// Offset is a draw displacement in logical pixels, not an update to builtin y.
pub fn step_entrance(offset: &mut f64, delay: &mut i32) {
    if offset.abs() > 1e-5 && *delay == 0 {
        *offset *= 0.8;
    }
    if *delay > 0 {
        *delay -= 1;
    }
}
#[cfg(test)]
mod entrance_tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn all_47_characters_match_first_180_frames() {
        let init: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/title-creation.json"
        )))
        .unwrap();
        let rows: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/character-entry.json"
        )))
        .unwrap();
        let mut models = HashMap::new();
        for (id, v) in init["characters"].as_object().unwrap() {
            let y = v["ystart"].as_f64().unwrap();
            let delay = (y / 20.0).floor() as i32;
            assert_eq!(delay, v["delay"].as_i64().unwrap() as i32 + 1);
            models.insert(
                id.parse::<u64>().unwrap(),
                (
                    Bob {
                        phase: v["lifetime"].as_f64().unwrap() - std::f64::consts::PI / 60.0,
                        y,
                        increment: std::f64::consts::PI / 60.0,
                    },
                    -300.0,
                    delay,
                ),
            );
        }
        assert_eq!(models.len(), 47);
        assert_eq!(rows.as_array().unwrap().len(), 8460);
        for row in rows.as_array().unwrap() {
            let (b, o, d) = models.get_mut(&row["id"].as_u64().unwrap()).unwrap();
            step_character(b);
            step_entrance(o, d);
            assert_eq!(*d, row["delay"].as_i64().unwrap() as i32);
            assert!(
                (*o - row["startY"].as_f64().unwrap()).abs() < 1e-10,
                "offset {row}"
            );
            assert!(
                (b.y - row["y"].as_f64().unwrap()).abs() < 1e-10,
                "y {} {row}",
                b.y
            );
            assert!((b.phase - row["lifetime"].as_f64().unwrap()).abs() < 1e-10);
        }
    }
}

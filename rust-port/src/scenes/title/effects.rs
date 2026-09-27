//! Recovered effect motion; births replay captured parameters, not original RNG.
use serde_json::Value;
#[derive(Clone, Debug)]
pub struct Effect {
    pub triangle: bool,
    pub x: f32,
    pub y: f32,
    pub previous_x: f32,
    pub previous_y: f32,
    pub angle: f32,
    pub previous_angle: f32,
    pub alpha: f32,
    pub previous_alpha: f32,
    pub sx: f32,
    pub sy: f32,
    pub dx: f64,
    pub angle_change: f64,
    pub speed: f64,
    pub length: f32,
    pub width: f32,
    pub depth: f64,
    pub gui: bool,
}
fn n(v: &Value, k: &str) -> f64 {
    v[k].as_f64().unwrap_or(0.0)
}
impl Effect {
    pub fn from_record(record: &Value) -> Self {
        let v = &record["vars"];
        let x = n(v, "x") as f32;
        let y = n(v, "y") as f32;
        let angle = n(v, "image_angle") as f32;
        let alpha = n(v, "image_alpha") as f32;
        Self {
            triangle: record["object"] == "obj_charSelect_triangle",
            x,
            y,
            previous_x: x,
            previous_y: y,
            angle,
            previous_angle: angle,
            alpha,
            previous_alpha: alpha,
            sx: n(v, "image_xscale") as f32,
            sy: n(v, "image_yscale") as f32,
            dx: n(v, "hspeedRandom"),
            angle_change: n(v, "angleChange"),
            speed: n(v, "randomSpeed"),
            length: n(v, "drawLength") as f32,
            width: n(v, "linewidth") as f32,
            depth: n(v, "depth"),
            gui: v["gui"].as_bool().unwrap_or(false),
        }
    }
    pub fn step(&mut self) {
        self.previous_x = self.x;
        self.previous_y = self.y;
        self.previous_angle = self.angle;
        self.previous_alpha = self.alpha;
        if self.triangle {
            self.x = (self.x as f64 + self.dx) as f32;
            self.y += 1.0;
            self.angle = (self.angle as f64 + self.angle_change) as f32;
            self.alpha = (self.alpha as f64 - 0.001) as f32;
        } else {
            self.x = (self.x as f64 + self.speed) as f32;
        }
    }
    pub fn alive(&self) -> bool {
        if self.triangle {
            self.y <= 500.0
        } else {
            self.x <= 1000.0
        }
    }
}
pub struct EffectReplay {
    pub active: Vec<Effect>,
    births: Vec<(u64, Effect)>,
    period: u64,
    tick: u64,
}
impl EffectReplay {
    pub fn new(data: &Value) -> Self {
        Self {
            active: data["initial"]
                .as_array()
                .unwrap()
                .iter()
                .map(Effect::from_record)
                .collect(),
            births: data["births"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| (r["at"].as_u64().unwrap(), Effect::from_record(r)))
                .collect(),
            period: (data["end_frame"].as_u64().unwrap() - data["start_frame"].as_u64().unwrap())
                .max(1),
            tick: 0,
        }
    }
    pub fn step(&mut self) {
        self.tick += 1;
        for e in &mut self.active {
            e.step();
        }
        self.active.retain(Effect::alive);
        let at = (self.tick - 1) % self.period + 1;
        for (f, e) in &self.births {
            if *f == at {
                self.active.push(e.clone());
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn particle_x_motion_and_lifetime() {
        let v = serde_json::json!({"object":"obj_speedingParticle","vars":{"x":996,"y":301,"randomSpeed":8}});
        let mut e = Effect::from_record(&v);
        e.step();
        assert_eq!(e.x, 1004.0);
        assert_eq!(e.y, 301.0);
        assert!(!e.alive());
    }
    #[test]
    fn triangle_preserves_unbounded_angle_and_fades() {
        let v = serde_json::json!({"object":"obj_charSelect_triangle","vars":{"x":1,"y":499,"hspeedRandom":0.25,"angleChange":3,"image_angle":720,"image_alpha":0.4}});
        let mut e = Effect::from_record(&v);
        e.step();
        assert_eq!((e.x, e.y, e.angle), (1.25, 500.0, 723.0));
        assert!((e.alpha - 0.399).abs() < 1e-6);
        assert!(e.alive());
        e.step();
        assert!(!e.alive());
    }
    #[test]
    fn original_particle_fixture_pairs() {
        let a: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/particle-motion.json"
        )))
        .unwrap();
        for pair in a.as_array().unwrap() {
            let before = serde_json::json!({"object":pair["object"],"vars":pair["before"]});
            let mut e = Effect::from_record(&before);
            e.step();
            let after = &pair["after"];
            assert!((e.x - n(after, "x") as f32).abs() < 0.0001);
            assert!((e.y - n(after, "y") as f32).abs() < 0.0001);
            if e.triangle {
                assert!((e.angle - n(after, "image_angle") as f32).abs() < 0.0001);
                assert!((e.alpha - n(after, "image_alpha") as f32).abs() < 1e-6);
            }
        }
    }
}

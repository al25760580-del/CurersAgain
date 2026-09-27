//! MenuBGEffect Alarm[1] and charSelect_triangle Create/Step reconstruction.
//! Native cadence/ranges/motion; random stream is port-specific, NOT GameMaker RNG parity.
use crate::scenes::title::effects::Effect;
pub struct MenuTriangles {
    pub active: Vec<Effect>,
    tick: u64,
    rng: u64,
}
impl Default for MenuTriangles {
    fn default() -> Self {
        Self::new(0x484f4c4f43555245)
    }
}
impl MenuTriangles {
    pub fn new(seed: u64) -> Self {
        Self {
            active: vec![],
            tick: 0,
            rng: seed,
        }
    }
    fn sample(&mut self) -> f64 {
        self.rng = self.rng.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^= z >> 31;
        (z >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn birth(samples: [f64; 4]) -> Effect {
        let [x, angle, size, drift] = samples;
        Effect {
            triangle: true,
            x: (x * 640.).floor() as f32,
            y: -30.,
            previous_x: (x * 640.).floor() as f32,
            previous_y: -30.,
            angle: 0.,
            previous_angle: 0.,
            alpha: 0.4,
            previous_alpha: 0.4,
            sx: (size * 2.) as f32,
            sy: (size * 2.) as f32,
            dx: drift - 0.5,
            angle_change: (angle * 5.).floor(),
            speed: 0.,
            length: 0.,
            width: 0.,
            depth: 250.,
            gui: false,
        }
    }
    pub fn step(&mut self) {
        self.tick += 1;
        // Native alarm creates the instance before its Step; captured newborn y=-29.
        if self.tick % 30 == 0 {
            let samples = [self.sample(), self.sample(), self.sample(), self.sample()];
            self.active.push(Self::birth(samples));
        }
        for e in &mut self.active {
            e.step();
        }
        self.active.retain(Effect::alive);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_cadence_and_newborn_step() {
        let mut b = MenuTriangles::default();
        for _ in 0..29 {
            b.step()
        }
        assert!(b.active.is_empty());
        b.step();
        assert_eq!(b.active.len(), 1);
        assert_eq!(b.active[0].y, -29.);
        assert!((b.active[0].alpha - 0.399).abs() < 0.00001);
        for _ in 0..30 {
            b.step()
        }
        assert_eq!(b.active.len(), 2);
    }
    #[test]
    fn injectable_native_ranges() {
        let e = MenuTriangles::birth([0.5, 0.9, 0.75, 0.25]);
        assert_eq!(
            (e.x, e.y, e.sx, e.angle_change, e.dx),
            (320., -30., 1.5, 4., -0.25)
        );
    }
    #[test]
    fn bounded_population_and_no_replay() {
        let mut b = MenuTriangles::default();
        for _ in 0..36000 {
            b.step();
            assert!(b.active.len() <= 18)
        }
        assert!(!b.active.is_empty());
    }
}

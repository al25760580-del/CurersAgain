//! Experimental semantic port. Not a GameMaker ABI implementation or runnable game.
//! Evidence: AttackController Step_1 at VA 0x140b1c070 in the analyzed binary.
//! Global variable-id slot 0x14576aff8 resolves to damageTextThisFrame.

#[derive(Debug, Default, Clone, PartialEq)]
pub struct AttackControllerState {
    // f64 models the numeric value, NOT GameMaker's tagged RValue layout.
    pub damage_text_this_frame: f64,
}

impl AttackControllerState {
    /// Port of the observed reset only. No allocation/free or runner internals.
    pub fn begin_step(&mut self) {
        self.damage_text_this_frame = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn begins_frame_with_zero_damage_text() {
        let mut s = AttackControllerState {
            damage_text_this_frame: 17.0,
        };
        s.begin_step();
        assert_eq!(s.damage_text_this_frame, 0.0);
    }
    #[test]
    fn repeated_reset_is_idempotent() {
        let mut s = AttackControllerState::default();
        s.begin_step();
        s.begin_step();
        assert_eq!(s, AttackControllerState::default());
    }
}

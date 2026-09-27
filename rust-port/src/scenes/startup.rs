//! Visual startup state machine recovered from YYC (not video/state replay).
//! PreIntro: Create 0x142d9c6b0, Step 0x142d9cea0, Alarms 0/1 0x142d9c990/0x142d9cbc0.
//! Intro: Create 0x142528f50, Step 0x14252a050, Alarms 0..3 0x142529660..0x142529d90.
//! All timers are game steps at 60 Hz. Alphas are black-cover opacity and text opacity.
//! Scope: ordinary boot without unlock notifications; audio and unlock overlay are separate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootRoom {
    PreIntro,
    Intro,
    Title,
}
#[derive(Clone, Debug)]
pub struct Startup {
    pub room: BootRoom,
    pub lifetime: u32,
    pub scene: i32,
    pub text: usize,
    pub screen_alpha: f64,
    pub text_alpha: f64,
    alarms: [i32; 4],
}
impl Default for Startup {
    fn default() -> Self {
        Self {
            room: BootRoom::PreIntro,
            lifetime: 0,
            scene: 0,
            text: 0,
            screen_alpha: 1.,
            text_alpha: 1.,
            alarms: [10, -1, -1, -1],
        }
    }
}
impl Startup {
    /// Construct the second room. Original sprite 1905 has six 320×180 frames.
    fn intro() -> Self {
        Self {
            room: BootRoom::Intro,
            lifetime: 0,
            scene: -1,
            text: 0,
            screen_alpha: 1.,
            text_alpha: 0.,
            alarms: [31, -1, 31, -1],
        }
    }
    /// EnterKey: PreIntro 0x142d9c650 -> next room (Intro), Intro 0x142528bc0
    /// -> Title in the ordinary no-unlock path. Caller supplies a pressed edge,
    /// not a held/released state. Never route this same edge into the title menu.
    pub fn enter_pressed(&mut self) -> bool {
        match self.room {
            BootRoom::PreIntro => *self = Self::intro(),
            BootRoom::Intro => self.finish_intro(),
            BootRoom::Title => return false,
        }
        true
    }
    /// Native Mouse_53 (global left pressed) calls EnterKey without a hitbox:
    /// PreIntro 0x142d9d170, Intro 0x14252ad60, method slot 0x1457477b8.
    /// Coalesce edges from the current input batch before replacing the room.
    /// No hold/release or title-menu confirmation is performed here.
    pub fn advance_from_input(&mut self, enter: bool, left_press: bool) -> bool {
        (enter || left_press) && self.enter_pressed()
    }
    /// Natural end of Intro. Not a generic skip for all startup rooms.
    fn finish_intro(&mut self) {
        self.room = BootRoom::Title;
        self.lifetime = 0;
    }
    /// Execute one alarm. Returning true means room replacement occurred during the alarm.
    fn alarm(&mut self, i: usize) -> bool {
        if self.room == BootRoom::PreIntro {
            if i == 0 && self.screen_alpha > 1e-12 {
                self.screen_alpha -= 0.02;
                if self.screen_alpha < -1e-12 {
                    self.screen_alpha = 0.;
                } else {
                    self.alarms[0] = 1;
                }
            }
            if i == 1 && self.screen_alpha < 1. - 1e-12 {
                self.screen_alpha += 0.02;
                if self.screen_alpha >= 1. - 1e-12 {
                    self.screen_alpha = 1.;
                    if self.scene >= 1 {
                        *self = Self::intro();
                        return true;
                    }
                    self.scene += 1;
                } else {
                    self.alarms[1] = 1;
                }
            }
        } else if self.room == BootRoom::Intro {
            match i {
                0 => {
                    if self.screen_alpha > 0. {
                        self.screen_alpha = (self.screen_alpha - 0.26).max(0.);
                        if self.screen_alpha > 0. {
                            self.alarms[0] = 12;
                        }
                    }
                }
                1 => {
                    if self.screen_alpha < 1. {
                        self.screen_alpha = (self.screen_alpha + 0.26).min(1.);
                        if self.screen_alpha < 1. {
                            self.alarms[1] = 12;
                        } else {
                            self.scene = if self.scene >= 5 { -1 } else { self.scene + 1 };
                        }
                    }
                }
                2 => {
                    if self.text_alpha < 1. {
                        self.text_alpha = (self.text_alpha + 0.26).min(1.);
                        if self.text_alpha < 1. {
                            self.alarms[2] = 12;
                        }
                    }
                }
                3 => {
                    if self.text_alpha > 0. {
                        self.text_alpha = (self.text_alpha - 0.26).max(0.);
                        if self.text_alpha > 0. {
                            self.alarms[3] = 12;
                        } else if self.text < 8 {
                            self.text += 1;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }
    /// Alarm phase precedes Step. New-room creation during an alarm starts that room's tick.
    /// Step cases below are the literal lifetime switch table, not guessed durations.
    pub fn step(&mut self) {
        if self.room == BootRoom::Title {
            return;
        }
        for i in 0..4 {
            if self.alarms[i] > 0 {
                self.alarms[i] -= 1;
                if self.alarms[i] == 0 {
                    self.alarms[i] = -1;
                    if self.alarm(i) {
                        self.step();
                        return;
                    }
                }
            }
        }
        match self.room {
            BootRoom::PreIntro => match self.lifetime {
                240 | 600 => self.alarms[1] = 1,
                320 => self.alarms[0] = 1,
                _ => {}
            },
            BootRoom::Intro => match self.lifetime {
                30 => self.scene = 0,
                214 | 486 | 960 | 1266 | 1604 | 2058 => {
                    self.alarms[1] = 1;
                    self.alarms[3] = 1;
                }
                261 | 532 | 1006 | 1312 | 1650 => {
                    self.alarms[0] = 1;
                    self.alarms[2] = 1;
                }
                746 | 1854 => self.alarms[3] = 1,
                792 | 1900 => self.alarms[2] = 1,
                2140 => {
                    self.finish_intro();
                    return;
                }
                _ => {}
            },
            BootRoom::Title => {}
        }
        self.lifetime += 1;
    }
    /// Select the text index: PreIntro uses currentScene, Intro uses currentText.
    pub fn text_index(&self) -> usize {
        if self.room == BootRoom::PreIntro {
            self.scene.max(0) as usize
        } else {
            self.text
        }
    }
    /// Return the live ordinary-boot room label for the HUD and external monitor.
    pub fn scene_name(&self) -> &'static str {
        match self.room {
            BootRoom::PreIntro => "rm_Pre_Intro (1)",
            BootRoom::Intro => "rm_Intro (2)",
            BootRoom::Title => "rm_Title (3)",
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_startup_runtime_samples() {
        let a: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/startup-motion.json"
        )))
        .unwrap();
        for kind in ["obj_Pre_Intro", "obj_Intro"] {
            let mut s = if kind == "obj_Pre_Intro" {
                Startup::default()
            } else {
                Startup::intro()
            };
            for v in a.as_array().unwrap().iter().filter(|v| v["object"] == kind) {
                let t = v["lifetime"].as_u64().unwrap() as u32;
                while s.lifetime < t && s.room != BootRoom::Title {
                    s.step();
                }
                assert_eq!(
                    s.scene,
                    v["currentScene"].as_i64().unwrap() as i32,
                    "{kind} tick {t}"
                );
                assert_eq!(
                    s.text,
                    v["currentText"].as_u64().unwrap() as usize,
                    "{kind} tick {t}"
                );
                assert!(
                    (s.screen_alpha - v["screenAlpha"].as_f64().unwrap()).abs() < 1e-10,
                    "screen {kind} tick {t}: {}",
                    s.screen_alpha
                );
                if kind == "obj_Intro" {
                    assert!(
                        (s.text_alpha - v["textAlpha"].as_f64().unwrap()).abs() < 1e-10,
                        "text tick {t}: {}",
                        s.text_alpha
                    );
                }
            }
        }
    }
    #[test]
    fn enter_in_preintro_enters_intro_not_title() {
        let mut s = Startup::default();
        for _ in 0..400 {
            s.step();
        }
        assert!(s.enter_pressed());
        assert_eq!(s.room, BootRoom::Intro);
        assert_eq!(s.lifetime, 0);
        assert_eq!(s.scene, -1);
        assert_eq!(s.text, 0);
        assert_eq!(s.screen_alpha, 1.0);
        assert_eq!(s.text_alpha, 0.0);
        assert_eq!(s.alarms, [31, -1, 31, -1]);
    }
    #[test]
    fn second_press_enters_title_and_title_is_not_confirmed() {
        let mut s = Startup::default();
        s.enter_pressed();
        assert!(s.enter_pressed());
        assert_eq!(s.room, BootRoom::Title);
        assert!(!s.enter_pressed());
    }
    #[test]
    fn advancing_time_without_another_edge_does_not_skip_intro() {
        let mut s = Startup::default();
        s.enter_pressed();
        for _ in 0..120 {
            s.step();
        }
        assert_eq!(s.room, BootRoom::Intro);
        assert_eq!(s.lifetime, 120);
    }
}

#[cfg(test)]
mod click_tests {
    use super::*;
    #[test]
    fn left_press_advances_one_room_at_a_time() {
        let mut s = Startup::default();
        assert!(s.advance_from_input(false, true));
        assert_eq!(s.room, BootRoom::Intro);
        assert_eq!(s.lifetime, 0);
        assert!(s.advance_from_input(false, true));
        assert_eq!(s.room, BootRoom::Title);
        assert!(!s.advance_from_input(false, true));
    }
    #[test]
    fn simultaneous_enter_and_click_do_not_skip_two_rooms() {
        let mut s = Startup::default();
        assert!(s.advance_from_input(true, true));
        assert_eq!(s.room, BootRoom::Intro);
    }
    #[test]
    fn no_new_edge_does_not_advance() {
        let mut s = Startup::default();
        s.advance_from_input(false, true);
        for _ in 0..120 {
            assert!(!s.advance_from_input(false, false));
            s.step();
        }
        assert_eq!(s.room, BootRoom::Intro);
    }
}

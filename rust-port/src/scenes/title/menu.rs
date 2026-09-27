//! Semantic title-menu reconstruction for HoloCure 0.7.1746645739.
//! Evidence: SelectLeft=0x1435bc9d0, SelectRight=0x1435bcea0,
//! SelectUp=0x1435bc890, SelectDown=0x1435bc930, Confirmed=0x1435bb3f0.
//! No runner ABI, audio side effects, saves or original rendering reproduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Title,
    Scores,
    Achievements,
    Shop,
    Characters,
    House,
    Options,
    Credits,
    Exit,
}
#[derive(Debug, Clone, Copy)]
pub enum Input {
    Left,
    Right,
    Up,
    Down,
    Confirm,
    Back,
    Hover(usize),
}
pub const LABELS: [&str; 8] = [
    "LEADERBOARDS",
    "ACHIEVEMENTS",
    "SHOP",
    "PLAY",
    "HOLO HOUSE",
    "SETTINGS",
    "CREDITS",
    "QUIT",
];
#[derive(Debug)]
pub struct TitleMenu {
    pub current_option: usize,
    pub can_control: bool,
    pub idle_time: f64,
    pub screen: Screen,
    pub house_unlocked: bool,
    pub leaderboard_enabled: bool,
}
impl Default for TitleMenu {
    fn default() -> Self {
        Self {
            current_option: 3,
            can_control: false,
            idle_time: 0.0,
            screen: Screen::Title,
            house_unlocked: true,
            leaderboard_enabled: true,
        }
    }
}
impl TitleMenu {
    pub fn ready_for_reference() -> Self {
        Self {
            can_control: true,
            ..Self::default()
        }
    }
    pub fn input(&mut self, input: Input) {
        if self.screen != Screen::Title {
            if matches!(input, Input::Back) {
                self.screen = Screen::Title;
                self.can_control = true;
                self.idle_time = 0.0;
            }
            return;
        }
        // Decompiled Up/Down reset idletime, but do not move currentOption.
        if matches!(
            input,
            Input::Up | Input::Down | Input::Left | Input::Right | Input::Confirm
        ) {
            self.idle_time = 0.0;
        }
        if !self.can_control {
            return;
        }
        match input {
            Input::Left => self.current_option = (self.current_option + 7) % 8,
            Input::Right => self.current_option = (self.current_option + 1) % 8,
            Input::Hover(i) if i < 8 => {
                self.current_option = i;
                self.idle_time = 0.0;
            }
            Input::Confirm => {
                self.screen = match self.current_option {
                    0 if self.leaderboard_enabled => Screen::Scores,
                    1 => Screen::Achievements,
                    2 => Screen::Shop,
                    3 => Screen::Characters,
                    4 if self.house_unlocked => Screen::House,
                    5 => Screen::Options,
                    6 => Screen::Credits,
                    7 => Screen::Exit,
                    _ => Screen::Title,
                }
            }
            _ => {}
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn ready() -> TitleMenu {
        TitleMenu {
            can_control: true,
            ..Default::default()
        }
    }
    #[test]
    fn reference_launch_starts_at_play() {
        let m = TitleMenu::ready_for_reference();
        assert_eq!(m.current_option, 3);
        assert!(m.can_control);
        assert_eq!(m.screen, Screen::Title);
    }
    #[test]
    fn observed_initial_selection() {
        assert_eq!(TitleMenu::default().current_option, 3);
    }
    #[test]
    fn wrap_left() {
        let mut m = ready();
        m.current_option = 0;
        m.input(Input::Left);
        assert_eq!(m.current_option, 7);
    }
    #[test]
    fn wrap_right() {
        let mut m = ready();
        m.current_option = 7;
        m.input(Input::Right);
        assert_eq!(m.current_option, 0);
    }
    #[test]
    fn vertical_only_resets_idle() {
        let mut m = ready();
        m.idle_time = 10.0;
        m.input(Input::Down);
        assert_eq!(m.current_option, 3);
        assert_eq!(m.idle_time, 0.0);
        m.input(Input::Up);
        assert_eq!(m.current_option, 3);
    }
    #[test]
    fn control_gate() {
        let mut m = TitleMenu::default();
        m.input(Input::Right);
        m.input(Input::Confirm);
        assert_eq!(m.current_option, 3);
        assert_eq!(m.screen, Screen::Title);
    }
    #[test]
    fn confirmation_routes() {
        let expected = [
            Screen::Scores,
            Screen::Achievements,
            Screen::Shop,
            Screen::Characters,
            Screen::House,
            Screen::Options,
            Screen::Credits,
            Screen::Exit,
        ];
        for (i, s) in expected.into_iter().enumerate() {
            let mut m = ready();
            m.current_option = i;
            m.input(Input::Confirm);
            assert_eq!(m.screen, s);
        }
    }
    #[test]
    fn locked_routes_do_not_leave_title() {
        let mut m = ready();
        m.house_unlocked = false;
        m.current_option = 4;
        m.input(Input::Confirm);
        assert_eq!(m.screen, Screen::Title);
        m.leaderboard_enabled = false;
        m.current_option = 0;
        m.input(Input::Confirm);
        assert_eq!(m.screen, Screen::Title);
    }
    #[test]
    fn returning_preserves_selection() {
        let mut m = ready();
        m.current_option = 5;
        m.input(Input::Confirm);
        m.input(Input::Back);
        assert_eq!(m.screen, Screen::Title);
        assert_eq!(m.current_option, 5);
    }
}

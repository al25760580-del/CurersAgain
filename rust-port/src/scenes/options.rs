//! `rm_Options` - the settings room behind the title's Settings icon.
//!
//! Everything here is derived from the original's `obj_Options` events: `Create`,
//! `Step`, `Alarm 0`, the mouse events and the `Select*` scripts. Drawing lives in
//! `render::options` and is deliberately not part of this module, so the whole
//! room can be tested without a renderer.
//!
//! Two pages: 7 options on page 0 and 12 on page 1, shown through a 7-row window
//! with a `show_option_range` scroll offset, exactly as the original computes it.

use crate::first_run::Settings;

/// The 10 controller buttons the original offers, in its own order. The stored
/// `controllerButtons` resolve to positions in this list through
/// `GetCurrentController`.
pub const CONTROLLER_BUTTON_LIST: [i32; 10] =
    [32769, 32770, 32771, 32772, 32773, 32775, 32774, 32776, 32778, 32777];

/// Resolutions in the order the original lists them; the index is what is stored.
pub const RESOLUTIONS: [(u32, u32); 4] = [(640, 360), (1280, 720), (1920, 1080), (2560, 1440)];

/// Keys the original refuses while rebinding, named by its own `key_to_string`.
/// `ValidKeysOnly` itself is not present in the export, so this list is the only
/// verified part of the rejection rule.
pub const ILLEGAL_REBIND_KEYS: [&str; 9] = ["[", "]", "/", "\\", ";", "WIN KEY", "+", "=", "*"];

/// Number of action keys the original rebinds. The saved file carries one extra
/// entry that the room never reads; the vector is kept whole to preserve it.
pub const KEY_SLOTS: usize = 6;

/// Rows visible at once: `current_option` runs 0..=6 and the window then scrolls.
pub const VISIBLE_ROWS: usize = 7;

/// Options per page, matching the original's `maxOptions`.
pub const MAX_OPTIONS: [usize; 2] = [7, 12];

/// Menu container origin, from the original's `Create`.
pub const CONTAINER: (i32, i32) = (320, 48);

/// Row pitch in logical pixels, from the original's mouse hit test.
pub const ROW_PITCH: i32 = 34;

/// Left offset of a row from the container, from the original's mouse hit test.
pub const ROW_X_OFFSET: i32 = 12;

/// Vertical offset of the first row from the container.
pub const ROW_Y_OFFSET: i32 = 43;

/// Row width the original tests with its `long` button hitbox.
pub const ROW_WIDTH: i32 = 180;

/// Row height the original tests with its `long` button hitbox.
pub const ROW_HEIGHT: i32 = 29;

/// What a confirmed option asks the caller to do. The room never touches the
/// window or the audio device itself; it reports and the caller applies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    None,
    /// Store the settings, as the original's `SaveSettings` does.
    Save,
    /// Apply the pending resolution and store it.
    ApplyResolution(u32, u32),
    /// Switch the language pack.
    SetLanguage(String),
    /// Leave the room and give control back to the title.
    Exit,
}

/// `rm_Options` state. All fields mirror the original's own variables so the
/// behaviour can be compared statement by statement.
pub struct Options {
    pub current_option: usize,
    pub show_option_range: usize,
    pub option_page: usize,
    pub keybind_menu: bool,
    pub controller_menu: bool,
    pub remapping: bool,
    /// Countdown while rebinding all six keys at once; 0 when idle.
    pub set_all: usize,
    pub prev_menu: [usize; 2],
    /// False for the first frames: the original sets `alarm[0] = 10` on create.
    pub can_control: bool,
    pub alarm_0: i32,
    /// True once a controller assignment has been accepted; cleared on up.
    pub controller_set: bool,
    /// Pending resolution index, applied only on confirm.
    pub selected_resolution: usize,
    /// Pending language index into `languages`.
    pub selected_language_option: usize,
    /// Pending controller slot positions into `CONTROLLER_BUTTON_LIST`.
    pub current_positions: [usize; 6],
    /// True while a slider was dragged; the original saves on mouse release.
    pub changed_settings: bool,
    /// Language ids the caller offers, in the order the menu shows them.
    pub languages: Vec<String>,
    pub settings: Settings,
}

impl Options {
    /// Build the room. The original's `Create` reads `global.Resolution` into the
    /// pending value and then forces `global.Resolution = 1`; the pending read is
    /// reproduced here so a cancelled visit leaves the stored value untouched.
    pub fn new(settings: Settings, languages: Vec<String>) -> Self {
        let mut s = Self {
            current_option: 0,
            show_option_range: 0,
            option_page: 0,
            keybind_menu: false,
            controller_menu: false,
            remapping: false,
            set_all: 0,
            prev_menu: [0, 0],
            can_control: false,
            alarm_0: 10,
            controller_set: false,
            selected_resolution: (settings.resolution as usize).min(RESOLUTIONS.len() - 1),
            selected_language_option: 0,
            current_positions: [0; 6],
            changed_settings: false,
            languages,
            settings,
        };
        s.sync_language_index();
        s.current_controller();
        s
    }

    /// Advance the create alarm; control opens after ten frames, as `Alarm 0`.
    pub fn update(&mut self) {
        if self.alarm_0 > 0 {
            self.alarm_0 -= 1;
            if self.alarm_0 == 0 {
                self.can_control = true;
            }
        }
    }

    /// Absolute option index the cursor is on: `current_option + show_option_range`.
    pub fn absolute_option(&self) -> usize {
        self.current_option + self.show_option_range
    }

    fn in_submenu(&self) -> bool {
        self.keybind_menu || self.controller_menu
    }

    pub fn select_left(&mut self) {
        if !self.can_control {
            return;
        }
        if self.controller_menu && self.current_option < KEY_SLOTS {
            let len = CONTROLLER_BUTTON_LIST.len();
            let p = &mut self.current_positions[self.current_option];
            *p = if *p > 0 { *p - 1 } else { len - 1 };
            return;
        }
        if self.in_submenu() {
            return;
        }
        match (self.option_page, self.absolute_option()) {
            (0, 1) => self.settings.music_volume = (self.settings.music_volume - 0.1).max(0.0),
            (0, 2) => self.settings.sound_volume = (self.settings.sound_volume - 0.1).max(0.0),
            (0, 5) => {
                let n = self.languages.len().max(1);
                self.selected_language_option = if self.selected_language_option > 0 {
                    self.selected_language_option - 1
                } else {
                    n - 1
                };
            }
            (1, 0) => {
                if self.selected_resolution > 0 {
                    self.selected_resolution -= 1;
                }
            }
            (1, 2) => self.settings.attack_alpha = (self.settings.attack_alpha - 0.1).max(0.3),
            (1, 6) => self.settings.port_display = 1.0 - self.settings.port_display,
            _ => {}
        }
    }

    pub fn select_right(&mut self) {
        if !self.can_control {
            return;
        }
        if self.controller_menu && self.current_option < KEY_SLOTS {
            let len = CONTROLLER_BUTTON_LIST.len();
            let p = &mut self.current_positions[self.current_option];
            *p = if *p + 1 < len { *p + 1 } else { 0 };
            return;
        }
        if self.in_submenu() {
            return;
        }
        match (self.option_page, self.absolute_option()) {
            (0, 1) => self.settings.music_volume = (self.settings.music_volume + 0.1).min(1.0),
            (0, 2) => self.settings.sound_volume = (self.settings.sound_volume + 0.1).min(1.0),
            (0, 5) => {
                let n = self.languages.len().max(1);
                self.selected_language_option = if self.selected_language_option + 1 < n {
                    self.selected_language_option + 1
                } else {
                    0
                };
            }
            (1, 0) => {
                if self.selected_resolution + 1 < RESOLUTIONS.len() {
                    self.selected_resolution += 1;
                }
            }
            (1, 2) => self.settings.attack_alpha = (self.settings.attack_alpha + 0.1).min(1.0),
            (1, 6) => self.settings.port_display = 1.0 - self.settings.port_display,
            _ => {}
        }
    }

    pub fn select_up(&mut self) {
        if !self.can_control {
            return;
        }
        if self.current_option == 0 && self.show_option_range > 0 {
            self.show_option_range -= 1;
        } else if self.current_option > 0 && !self.remapping {
            self.current_option -= 1;
        }
        self.controller_set = false;
    }

    pub fn select_down(&mut self) {
        if !self.can_control {
            return;
        }
        if !self.in_submenu() {
            let max = MAX_OPTIONS[self.option_page];
            if self.current_option < VISIBLE_ROWS && self.absolute_option() < max - 1 {
                if self.current_option == VISIBLE_ROWS - 1 {
                    self.show_option_range += 1;
                } else {
                    self.current_option += 1;
                }
            }
        } else if self.current_option < KEY_SLOTS && !self.remapping {
            self.current_option += 1;
        }
    }

    /// Mouse wheel: the original scrolls the window without moving the cursor.
    pub fn scroll(&mut self, down: bool) {
        if !self.can_control {
            return;
        }
        if down {
            if !self.in_submenu()
                && self.show_option_range < MAX_OPTIONS[self.option_page].saturating_sub(VISIBLE_ROWS)
            {
                self.show_option_range += 1;
            }
        } else if self.show_option_range > 0 {
            self.show_option_range -= 1;
        }
    }

    /// Row the pointer is over, from the original's `container[1] + 43 + i*34`.
    pub fn row_at(&self, x: i32, y: i32) -> Option<usize> {
        if self.remapping {
            return None;
        }
        let left = CONTAINER.0 + ROW_X_OFFSET;
        for i in 0..VISIBLE_ROWS {
            let top = CONTAINER.1 + ROW_Y_OFFSET + (i as i32) * ROW_PITCH;
            if x >= left && x < left + ROW_WIDTH && y >= top && y < top + ROW_HEIGHT {
                return Some(i);
            }
        }
        None
    }

    /// Click a slider segment. Returns true when a value changed, which the
    /// original turns into a `changedSettings` flag and saves on release.
    pub fn slider_click(&mut self, x: i32) -> bool {
        if !self.can_control || self.in_submenu() || self.remapping {
            return false;
        }
        let left = CONTAINER.0 + 20;
        let width = 77;
        if x < left || x >= left + width {
            return false;
        }
        match (self.option_page, self.absolute_option()) {
            (0, 1) => {
                for j in 0..11 {
                    if x > left + j * 7 && x < left + (j + 1) * 7 {
                        self.settings.music_volume = j as f64 * 0.1;
                        return true;
                    }
                }
            }
            (0, 2) => {
                for j in 0..11 {
                    if x > left + j * 7 && x < left + (j + 1) * 7 {
                        self.settings.sound_volume = j as f64 * 0.1;
                        return true;
                    }
                }
            }
            (1, 2) => {
                for j in 0..8 {
                    let a = left + (j as f64 * 8.75) as i32;
                    let b = left + ((j + 1) as f64 * 8.75) as i32;
                    if x > a && x < b {
                        self.settings.attack_alpha = 0.3 + j as f64 * 0.1;
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    /// Confirm. Mirrors the original's `Confirmed`, including the page switch and
    /// the two rebinding submenus.
    pub fn confirmed(&mut self) -> Action {
        if !self.can_control {
            return Action::None;
        }
        if self.controller_menu {
            return self.confirm_controller();
        }
        if self.keybind_menu {
            if self.remapping {
                return Action::None;
            }
            self.remapping = true;
            if self.current_option == KEY_SLOTS {
                self.set_all = KEY_SLOTS;
                self.current_option = 0;
                for slot in self.settings.the_buttons.iter_mut().take(KEY_SLOTS) {
                    *slot = String::new();
                }
            }
            return Action::None;
        }
        let abs = self.absolute_option();
        if self.option_page == 0 {
            match abs {
                0 => {
                    self.option_page = 1;
                    self.current_option = 0;
                    self.show_option_range = 0;
                }
                3 => {
                    self.keybind_menu = true;
                    self.prev_menu = [self.current_option, self.show_option_range];
                    self.current_option = 0;
                    self.show_option_range = 0;
                }
                4 => {
                    self.controller_menu = true;
                    self.prev_menu = [self.current_option, self.show_option_range];
                    self.current_option = 0;
                    self.show_option_range = 0;
                    self.current_controller();
                }
                5 => {
                    if let Some(id) = self.languages.get(self.selected_language_option).cloned() {
                        self.settings.CurrentLanguage = id.clone();
                        return Action::SetLanguage(id);
                    }
                }
                6 => self.settings.hiscorenames = !self.settings.hiscorenames,
                _ => {}
            }
        } else {
            match abs {
                0 => {
                    let (w, h) = RESOLUTIONS[self.selected_resolution];
                    self.settings.resolution = self.selected_resolution as f64;
                    return Action::ApplyResolution(w, h);
                }
                1 => self.settings.fullscreen = !self.settings.fullscreen,
                3 => self.settings.show_damage_text = !self.settings.show_damage_text,
                4 => self.settings.light_fx = !self.settings.light_fx,
                5 => self.settings.screen_shake = !self.settings.screen_shake,
                6 => self.settings.port_display = 1.0 - self.settings.port_display,
                7 => self.settings.show_hud_hp = 1.0 - self.settings.show_hud_hp,
                8 => self.settings.show_hp_val = 1.0 - self.settings.show_hp_val,
                9 => self.settings.above_hp = 1.0 - self.settings.above_hp,
                10 => self.settings.hide_full_hp = 1.0 - self.settings.hide_full_hp,
                11 => self.settings.show_skill_radius = 1.0 - self.settings.show_skill_radius,
                _ => {}
            }
        }
        Action::Save
    }

    fn confirm_controller(&mut self) -> Action {
        // The original refuses the assignment when two slots resolve to the same
        // button and silently reloads the stored positions instead.
        let mut seen = [false; CONTROLLER_BUTTON_LIST.len()];
        for p in self.current_positions {
            if seen[p] {
                self.current_controller();
                return Action::None;
            }
            seen[p] = true;
        }
        for (i, p) in self.current_positions.iter().enumerate() {
            if let Some(slot) = self.settings.controller_buttons.get_mut(i) {
                *slot = controller_name(CONTROLLER_BUTTON_LIST[*p]).to_string();
            }
        }
        self.controller_set = true;
        Action::Save
    }

    /// Rebind one key, from the original's `Step` event. Returns true when the
    /// key was accepted.
    ///
    /// The original then drops control for ten frames (`canControl = false;
    /// alarm[0] = 10`) and reapplies the controls on every accepted key.
    pub fn rebind(&mut self, key: &str) -> bool {
        if !self.remapping || !self.can_control || key.is_empty() {
            return false;
        }
        if ILLEGAL_REBIND_KEYS.contains(&key) {
            return false;
        }
        // The original rejects a key already bound to another action.
        for (i, slot) in self.settings.the_buttons.iter().enumerate() {
            if i != self.current_option && slot == key {
                return false;
            }
        }
        if let Some(slot) = self.settings.the_buttons.get_mut(self.current_option) {
            *slot = key.to_string();
        }
        self.remapping = false;
        self.can_control = false;
        self.alarm_0 = 10;
        if self.set_all > 0 {
            self.set_all -= 1;
            self.current_option += 1;
            if self.set_all != 0 {
                self.remapping = true;
            }
        }
        true
    }

    /// Back out, mirroring the original's `ReturnMenu`.
    pub fn return_menu(&mut self) -> Action {
        if !self.can_control {
            return Action::None;
        }
        if self.option_page == 1 {
            self.option_page = 0;
            self.current_option = 0;
            self.show_option_range = 0;
            return Action::None;
        }
        if !self.in_submenu() {
            return Action::Exit;
        }
        if self.controller_menu {
            self.controller_set = false;
            self.controller_menu = false;
        } else if !self.remapping {
            self.keybind_menu = false;
        } else {
            return Action::None;
        }
        self.current_option = self.prev_menu[0];
        self.show_option_range = self.prev_menu[1];
        Action::None
    }

    /// Rebuild the pending controller positions from the stored buttons, as the
    /// original's `GetCurrentController` does.
    pub fn current_controller(&mut self) {
        self.current_positions = [0; 6];
        for i in 0..KEY_SLOTS {
            let stored = self.settings.controller_buttons.get(i).cloned().unwrap_or_default();
            for (j, code) in CONTROLLER_BUTTON_LIST.iter().enumerate() {
                if controller_name(*code) == stored {
                    self.current_positions[i] = j;
                }
            }
        }
    }

    /// Point the pending language index at the stored id.
    pub fn sync_language_index(&mut self) {
        if let Some(p) = self.languages.iter().position(|x| *x == self.settings.CurrentLanguage) {
            self.selected_language_option = p;
        }
    }
}

/// The `gp_*` name the original stores for a controller button code.
pub fn controller_name(code: i32) -> &'static str {
    match code {
        32769 => "gp_face1",
        32770 => "gp_shoulderlb",
        32771 => "gp_face2",
        32772 => "gp_shoulderrb",
        32773 => "gp_start",
        32774 => "gp_face3",
        32775 => "gp_select",
        32776 => "gp_padu",
        32777 => "gp_padd",
        32778 => "gp_padl",
        32779 => "gp_padr",
        _ => "gp_face1",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn room() -> Options {
        let mut s = Settings::default();
        s.the_buttons = vec![
            "SPACE".into(),
            "SHIFT".into(),
            "A".into(),
            "D".into(),
            "W".into(),
            "S".into(),
            "CTRL".into(),
        ];
        s.controller_buttons = vec![
            "gp_face1".into(),
            "gp_shoulderlb".into(),
            "gp_face2".into(),
            "gp_shoulderrb".into(),
            "gp_start".into(),
            "gp_select".into(),
            "gp_face3".into(),
        ];
        Options::new(s, vec!["eng".into(), "jp".into()])
    }

    fn open(r: &mut Options) {
        for _ in 0..10 {
            r.update();
        }
    }

    /// Walk the cursor down until it sits on an absolute option of the page.
    fn goto(r: &mut Options, abs: usize) {
        while r.absolute_option() < abs {
            r.select_down();
        }
        assert_eq!(r.absolute_option(), abs);
    }

    #[test]
    fn control_opens_after_ten_frames() {
        let mut r = room();
        assert!(!r.can_control);
        for _ in 0..9 {
            r.update();
        }
        assert!(!r.can_control);
        r.update();
        assert!(r.can_control);
    }

    #[test]
    fn nothing_responds_before_control_opens() {
        let mut r = room();
        r.select_down();
        r.select_left();
        r.confirmed();
        assert_eq!(r.current_option, 0);
        assert_eq!(r.settings.music_volume, Settings::default().music_volume);
        assert_eq!(r.return_menu(), Action::None);
    }

    #[test]
    fn page_zero_has_seven_rows_and_never_scrolls() {
        let mut r = room();
        open(&mut r);
        for _ in 0..6 {
            r.select_down();
        }
        assert_eq!(r.current_option, 6);
        assert_eq!(r.show_option_range, 0);
        r.select_down();
        assert_eq!(r.current_option, 6, "the last option is the end of the page");
        assert_eq!(r.show_option_range, 0, "seven options exactly fill the window");
        r.select_up();
        assert_eq!(r.current_option, 5);
    }

    #[test]
    fn page_one_scrolls_once_the_window_is_full() {
        let mut r = room();
        open(&mut r);
        r.confirmed();
        assert_eq!(r.option_page, 1);
        for _ in 0..6 {
            r.select_down();
        }
        assert_eq!(r.current_option, 6);
        assert_eq!(r.show_option_range, 0);
        r.select_down();
        assert_eq!(r.current_option, 6, "the window scrolls instead of the cursor");
        assert_eq!(r.show_option_range, 1);
        assert_eq!(r.absolute_option(), 7);
        r.select_up();
        assert_eq!(r.current_option, 5, "up moves the cursor while it is off the top");
        assert_eq!(r.show_option_range, 1, "the window only unwinds once the cursor is home");
        while r.current_option > 0 {
            r.select_up();
        }
        assert_eq!(r.current_option, 0);
        r.select_up();
        assert_eq!(r.show_option_range, 0);
    }

    #[test]
    fn page_one_has_twelve_options() {
        let mut r = room();
        open(&mut r);
        r.confirmed();
        assert_eq!(r.option_page, 1);
        for _ in 0..11 {
            r.select_down();
        }
        assert_eq!(r.absolute_option(), 11);
        r.select_down();
        assert_eq!(r.absolute_option(), 11, "the last option does not wrap");
    }

    #[test]
    fn volumes_step_by_a_tenth_and_clamp() {
        let mut r = room();
        open(&mut r);
        goto(&mut r, 1); // music volume is the second option on page zero
        r.settings.music_volume = 0.5;
        r.settings.sound_volume = 0.5;
        r.select_left();
        assert_eq!(r.settings.music_volume, 0.4);
        r.select_right();
        assert_eq!(r.settings.music_volume, 0.5);
        r.select_right();
        assert_eq!(r.settings.music_volume, 0.6);
        r.settings.music_volume = 0.0;
        r.select_left();
        assert_eq!(r.settings.music_volume, 0.0, "volume does not go negative");
        r.settings.music_volume = 1.0;
        r.select_right();
        assert_eq!(r.settings.music_volume, 1.0, "volume does not exceed one");
        goto(&mut r, 2); // sound volume is the third option on page zero
        r.settings.sound_volume = 0.5;
        r.select_left();
        assert_eq!(r.settings.sound_volume, 0.4);
    }

    #[test]
    fn attack_alpha_stays_in_the_original_range() {
        let mut r = room();
        open(&mut r);
        r.confirmed();
        r.settings.attack_alpha = 0.3;
        r.select_left();
        assert_eq!(r.settings.attack_alpha, 0.3, "the floor is 0.3, not 0.0");
        r.settings.attack_alpha = 1.0;
        r.select_right();
        assert_eq!(r.settings.attack_alpha, 1.0);
    }

    #[test]
    fn language_cycles_both_ways() {
        let mut r = room();
        open(&mut r);
        goto(&mut r, 5); // language is the sixth option on page zero
        assert_eq!(r.selected_language_option, 0);
        r.select_left();
        assert_eq!(r.selected_language_option, 1, "left from the first wraps to the last");
        r.select_right();
        assert_eq!(r.selected_language_option, 0);
        assert_eq!(r.confirmed(), Action::SetLanguage("eng".into()));
        assert_eq!(r.settings.CurrentLanguage, "eng");
    }

    #[test]
    fn resolution_applies_only_on_confirm() {
        let mut r = room();
        open(&mut r);
        r.confirmed();
        // The stored value is 1 (1280x720), so one step right lands on 1920x1080.
        r.select_right();
        assert_eq!(r.selected_resolution, 2);
        assert_eq!(r.settings.resolution, 1.0, "the stored value is untouched until confirm");
        assert_eq!(r.confirmed(), Action::ApplyResolution(1920, 1080));
        assert_eq!(r.settings.resolution, 2.0);
        r.select_left();
        r.select_left();
        assert_eq!(r.selected_resolution, 0);
        r.select_left();
        assert_eq!(r.selected_resolution, 0, "the first entry does not wrap");
    }

    #[test]
    fn page_one_toggles_flip_the_stored_value() {
        let mut r = room();
        open(&mut r);
        r.confirmed();
        let before = r.settings.screen_shake;
        for _ in 0..5 {
            r.select_down();
        }
        r.confirmed();
        assert_eq!(r.settings.screen_shake, !before);
    }

    #[test]
    fn numeric_toggles_stay_numeric() {
        let mut r = room();
        open(&mut r);
        r.confirmed();
        let before = r.settings.hide_full_hp;
        for _ in 0..10 {
            r.select_down();
        }
        r.confirmed();
        assert_eq!(r.settings.hide_full_hp, 1.0 - before);
        assert!(r.settings.hide_full_hp == 0.0 || r.settings.hide_full_hp == 1.0);
    }

    #[test]
    fn back_from_page_one_returns_to_page_zero() {
        let mut r = room();
        open(&mut r);
        r.confirmed();
        assert_eq!(r.option_page, 1);
        assert_eq!(r.return_menu(), Action::None);
        assert_eq!(r.option_page, 0);
        assert_eq!(r.current_option, 0);
        assert_eq!(r.show_option_range, 0);
    }

    #[test]
    fn back_from_the_root_exits() {
        let mut r = room();
        open(&mut r);
        assert_eq!(r.return_menu(), Action::Exit);
    }

    #[test]
    fn keybind_menu_remembers_where_it_came_from() {
        let mut r = room();
        open(&mut r);
        for _ in 0..3 {
            r.select_down();
        }
        assert_eq!(r.absolute_option(), 3);
        r.confirmed();
        assert!(r.keybind_menu);
        assert_eq!(r.prev_menu, [3, 0]);
        assert_eq!(r.current_option, 0);
        assert_eq!(r.return_menu(), Action::None);
        assert!(!r.keybind_menu);
        assert_eq!(r.current_option, 3);
    }

    #[test]
    fn rebinding_accepts_a_free_key_and_rejects_duplicates() {
        let mut r = room();
        open(&mut r);
        for _ in 0..3 {
            r.select_down();
        }
        r.confirmed();
        assert!(r.keybind_menu);
        r.confirmed();
        assert!(r.remapping);
        assert!(!r.rebind("D"), "D is already bound to another action");
        assert!(!r.rebind("/"), "the original refuses this key");
        assert!(r.rebind("J"));
        assert_eq!(r.settings.the_buttons[0], "J");
        assert!(!r.remapping);
        // The original locks input for ten frames after every accepted key.
        assert!(!r.can_control);
        assert_eq!(r.alarm_0, 10);
        r.select_down();
        assert_eq!(r.current_option, 0, "nothing moves while control is locked");
        for _ in 0..10 {
            r.update();
        }
        assert!(r.can_control);
    }

    #[test]
    fn set_all_clears_six_slots_and_waits_for_each() {
        let mut r = room();
        open(&mut r);
        for _ in 0..3 {
            r.select_down();
        }
        r.confirmed();
        for _ in 0..6 {
            r.select_down();
        }
        assert_eq!(r.current_option, 6);
        r.confirmed();
        assert_eq!(r.set_all, 6);
        assert_eq!(r.current_option, 0);
        assert!(r.settings.the_buttons[..6].iter().all(|k| k.is_empty()));
        for k in ["Z", "X", "C", "V", "B", "N"] {
            assert!(r.rebind(k), "{k} should be accepted");
            // Each accepted key locks input for ten frames before the next.
            assert!(!r.can_control);
            for _ in 0..10 {
                r.update();
            }
            assert!(r.can_control);
        }
        assert_eq!(r.set_all, 0);
        assert!(!r.remapping);
        assert_eq!(&r.settings.the_buttons[..6], &["Z", "X", "C", "V", "B", "N"]);
        // The seventh entry is written by the original but never read.
        assert_eq!(r.settings.the_buttons[6], "CTRL");
    }

    #[test]
    fn controller_menu_rejects_duplicate_slots() {
        let mut r = room();
        open(&mut r);
        goto(&mut r, 4); // controller menu is the fifth option on page zero
        r.confirmed();
        assert!(r.controller_menu);
        r.current_positions[1] = r.current_positions[0];
        assert_eq!(r.confirmed(), Action::None, "a duplicate assignment is refused");
        assert!(!r.controller_set);
        // The stored buttons occupy positions 0..5, so only 6..9 are free.
        r.current_positions[1] = 9;
        assert_eq!(r.confirmed(), Action::Save);
        assert!(r.controller_set);
        assert_eq!(r.settings.controller_buttons[1], "gp_padd");
    }

    #[test]
    fn controller_positions_come_from_the_stored_buttons() {
        let r = room();
        assert_eq!(r.current_positions, [0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn slider_click_sets_the_value_directly() {
        let mut r = room();
        open(&mut r);
        goto(&mut r, 1);
        let left = CONTAINER.0 + 20;
        assert!(r.slider_click(left + 3 * 7 + 3));
        assert!((r.settings.music_volume - 0.3).abs() < 1e-9);
        goto(&mut r, 2);
        assert!(r.slider_click(left + 7 * 7 + 3));
        assert!((r.settings.sound_volume - 0.7).abs() < 1e-9);
        // Clicking outside the slider bar changes nothing.
        assert!(!r.slider_click(left + 200));
        assert!(!r.slider_click(left - 40));
    }

    #[test]
    fn attack_alpha_slider_uses_its_own_segments() {
        let mut r = room();
        open(&mut r);
        r.confirmed(); // the first option on page zero opens page one
        assert_eq!(r.option_page, 1);
        goto(&mut r, 2);
        let left = CONTAINER.0 + 20;
        assert!(r.slider_click(left + 4 * 8 + 5));
        assert!((r.settings.attack_alpha - 0.7).abs() < 1e-9);
        // Eight segments, and the lowest one is 0.3 rather than 0.0.
        assert!(r.slider_click(left + 1));
        assert!((r.settings.attack_alpha - 0.3).abs() < 1e-9);
    }

    #[test]
    fn wheel_scrolls_the_window() {
        let mut r = room();
        open(&mut r);
        r.scroll(true);
        assert_eq!(r.show_option_range, 0, "page zero has exactly seven rows");
        r.confirmed();
        r.scroll(true);
        assert_eq!(r.show_option_range, 1);
        r.scroll(false);
        assert_eq!(r.show_option_range, 0);
    }

    #[test]
    fn row_hit_test_matches_the_original_geometry() {
        let mut r = room();
        let left = CONTAINER.0 + ROW_X_OFFSET;
        let first = CONTAINER.1 + ROW_Y_OFFSET;
        assert_eq!(r.row_at(left + 10, first + 10), Some(0));
        assert_eq!(r.row_at(left + 10, first + ROW_PITCH + 10), Some(1));
        assert_eq!(r.row_at(left - 5, first + 10), None, "outside the row");
        r.remapping = true;
        assert_eq!(r.row_at(left + 10, first + 10), None, "no clicks while remapping");
    }

    #[test]
    fn controller_names_match_the_stored_file() {
        assert_eq!(controller_name(32769), "gp_face1");
        assert_eq!(controller_name(32773), "gp_start");
        assert_eq!(controller_name(32775), "gp_select");
        assert_eq!(controller_name(32774), "gp_face3");
    }
}

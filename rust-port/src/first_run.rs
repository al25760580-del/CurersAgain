//! rm_InitRoom: source-derived state names, English reference UI.
//! Evidence: GameManager.Confirmed 0x14232cb20, LoadSettings 0x14091ac00,
//! and the captured initStep 0..4 sequence. This is not a full save/gameplay port.
#![allow(non_snake_case)]
use serde_json::{json, Value};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct Settings {
    pub readyToStart: bool,
    pub CurrentLanguage: String,
    pub username: String,
    /// Display OTHER players' names. Not consent to publishing this user's name.
    pub hiscorenames: bool,
    /// Original field is BOOLEAN, despite its name.
    pub hiscoreName: bool,
    extra: Value,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            readyToStart: false,
            CurrentLanguage: "eng".into(),
            username: "Player".into(),
            hiscorenames: true,
            hiscoreName: false,
            extra: json!({}),
        }
    }
}
impl Settings {
    pub fn LoadSettings(root: &Path) -> Result<Self, String> {
        let path = root.join("settings.json");
        let raw = match fs::read(&path) {
            Ok(x) => x,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(format!("Cannot read {}: {e}", path.display())),
        };
        let v: Value = serde_json::from_slice(&raw).map_err(|e| {
            format!(
                "Invalid settings JSON in {}: {e}. File was not changed.",
                path.display()
            )
        })?;
        if !v.is_object() {
            return Err("Settings must be a JSON object. File was not changed.".into());
        }
        for k in ["readyToStart", "hiscorenames", "hiscoreName"] {
            if v.get(k).is_some_and(|x| !x.is_boolean()) {
                return Err(format!("Settings field {k} must be boolean"));
            }
        }
        for k in ["CurrentLanguage", "username"] {
            if v.get(k).is_some_and(|x| !x.is_string()) {
                return Err(format!("Settings field {k} must be text"));
            }
        }
        let mut s = Self::default();
        s.readyToStart = v["readyToStart"].as_bool().unwrap_or(false);
        s.CurrentLanguage = v["CurrentLanguage"].as_str().unwrap_or("eng").into();
        s.username = v["username"].as_str().unwrap_or("Player").into();
        s.hiscorenames = v["hiscorenames"].as_bool().unwrap_or(true);
        s.hiscoreName = v["hiscoreName"].as_bool().unwrap_or(false);
        if !CheckName(&s.username) {
            s.readyToStart = false;
        }
        s.extra = v;
        Ok(s)
    }
    /// Same-directory atomic replacement on the Linux host; keep the previous file.
    /// No Steam paths, native save_n.dat, or Firebase credential files are used here.
    pub fn SaveSettings(&self, root: &Path) -> Result<(), String> {
        fs::create_dir_all(root).map_err(|e| e.to_string())?;
        let mut v = self.extra.clone();
        for (k, x) in [
            ("readyToStart", json!(self.readyToStart)),
            ("CurrentLanguage", json!(self.CurrentLanguage)),
            ("username", json!(self.username)),
            ("hiscorenames", json!(self.hiscorenames)),
            ("hiscoreName", json!(self.hiscoreName)),
            ("portSchema", json!(1)),
        ] {
            v[k] = x;
        }
        let dest = root.join("settings.json");
        let temp = root.join("settings.json.tmp");
        if dest.exists() {
            fs::copy(&dest, root.join("settings.json.bak")).map_err(|e| e.to_string())?;
        }
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| format!("Cannot create settings transaction {}: {e}", temp.display()))?;
        let result = (|| {
            f.write_all(&serde_json::to_vec_pretty(&v).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            f.sync_all().map_err(|e| e.to_string())?;
            fs::rename(&temp, &dest).map_err(|e| e.to_string())?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result
    }
}
pub fn ProfileDirectory() -> Result<PathBuf, String> {
    let root = if let Some(p) = std::env::var_os("HOLOCURE_PORT_DATA_DIR") {
        PathBuf::from(p)
    } else if cfg!(target_os = "windows") {
        PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is unavailable")?)
            .join("holocure-rust-reference")
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")))
            .ok_or("No user data directory")?
            .join("holocure-rust-reference")
    };
    // Check lexical path AND the existing ancestor before creating anything.
    let blocked = |p: &Path| {
        p.components().any(|v| {
            ["steam", "steamapps", "compatdata"]
                .contains(&v.as_os_str().to_string_lossy().to_lowercase().as_str())
        })
    };
    if blocked(&root) {
        return Err("Refusing to use a Steam directory for the Rust profile".into());
    }
    if let Some(parent) = root.ancestors().find(|p| p.exists()) {
        if blocked(&parent.canonicalize().map_err(|e| e.to_string())?) {
            return Err("Refusing a profile linked into Steam".into());
        }
    }
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let resolved = root.canonicalize().map_err(|e| e.to_string())?;
    if resolved.components().any(|p| {
        ["steam", "steamapps", "compatdata"]
            .contains(&p.as_os_str().to_string_lossy().to_lowercase().as_str())
    }) {
        return Err("Refusing to use a Steam directory for the Rust profile".into());
    }
    Ok(resolved)
}
/// PORT POLICY: nonempty name, no control characters, <=24 Unicode scalars.
/// Original profanity/length rules and IME composition are not fully recovered.
pub fn CheckName(s: &str) -> bool {
    !s.trim().is_empty() && s.chars().count() <= 24 && !s.chars().any(char::is_control)
}

#[derive(Clone, Debug)]
pub struct GameManager {
    pub initStep: u8,
    pub currentOption: usize,
    pub langOption: usize,
    pub initLangOptions: Vec<String>,
    pub inputText: String,
    pub nameOkay: bool,
    pub canType: bool,
    pub hnShow: usize,
    pub useName: usize,
    pub image_index: f32,
    pub image_speed: f32,
    pub rectTime: u32,
    pub rectVis: bool,
    pub error: String,
    pub settings: Settings,
    delay: u8,
}
impl GameManager {
    pub fn new(settings: Settings) -> Self {
        Self {
            initStep: 0,
            currentOption: 0,
            langOption: 0,
            initLangOptions: vec!["eng".into(), "jp".into(), "id".into()],
            inputText: settings.username.clone(),
            nameOkay: false,
            canType: false,
            hnShow: usize::from(!settings.hiscorenames),
            useName: usize::from(!settings.hiscoreName),
            image_index: 0.,
            image_speed: 0.05,
            rectTime: 0,
            rectVis: true,
            error: String::new(),
            settings,
            delay: 0,
        }
    }
    pub fn active(&self) -> bool {
        !self.settings.readyToStart
    }
    pub fn step(&mut self) {
        self.image_index += self.image_speed;
        self.rectTime += 1;
        if self.rectTime >= 20 {
            self.rectTime = 0;
            self.rectVis = !self.rectVis;
        }
        if self.delay > 0 {
            self.delay -= 1;
            if self.delay == 0 {
                self.canType = true;
            }
        }
    }
    pub fn options(&self) -> Vec<&str> {
        match self.initStep {
            0 => self.initLangOptions.iter().map(String::as_str).collect(),
            1 => vec![],
            2 => vec!["first_run.show", "first_run.hide"],
            3 => vec!["first_run.allow", "first_run.decline"],
            _ => vec!["first_run.start", "first_run.restart"],
        }
    }
    pub fn SelectUp(&mut self) {
        self.currentOption = self.currentOption.saturating_sub(1);
    }
    pub fn SelectDown(&mut self) {
        let n = self.options().len();
        if n > 0 {
            self.currentOption = (self.currentOption + 1).min(n - 1);
        }
    }
    pub fn ReceiveInput(&mut self, s: &str) {
        if self.initStep != 1 || !self.canType {
            return;
        }
        for ch in s.chars() {
            if !ch.is_control() && self.inputText.chars().count() < 24 {
                self.inputText.push(ch);
            }
        }
        self.nameOkay = false;
        self.error.clear();
    }
    pub fn Backspace(&mut self) {
        if self.initStep == 1 && self.canType {
            self.inputText.pop();
            self.nameOkay = false;
            self.error.clear();
        }
    }
    /// Return true only after the final Start choice. Saving is the caller's transaction.
    pub fn Confirmed(&mut self) -> bool {
        self.error.clear();
        match self.initStep {
            0 => {
                self.langOption = self.currentOption;
                self.settings.CurrentLanguage = self.initLangOptions[self.langOption].clone();
                self.initStep = 1;
                self.canType = false;
                self.delay = 10;
            }
            1 => {
                if !self.canType {
                    return false;
                }
                self.nameOkay = CheckName(&self.inputText);
                if !self.nameOkay {
                    self.error = "Enter a name with 1 to 24 characters.".into();
                    return false;
                }
                self.settings.username = self.inputText.trim().into();
                self.initStep = 2;
            }
            2 => {
                self.hnShow = self.currentOption;
                self.settings.hiscorenames = self.hnShow == 0;
                self.initStep = 3;
            }
            3 => {
                self.useName = self.currentOption;
                self.settings.hiscoreName = self.useName == 0;
                self.initStep = 4;
            }
            4 => {
                if self.currentOption == 0 {
                    return true;
                }
                self.initStep = 0;
                self.nameOkay = false;
                self.canType = false;
            }
            _ => {}
        }
        self.currentOption = 0;
        false
    }
    pub fn EnterKey(&mut self) -> bool {
        self.Confirmed()
    }
    pub fn Complete(&mut self, root: &Path) -> bool {
        let mut next = self.settings.clone();
        next.readyToStart = true;
        match next.SaveSettings(root) {
            Ok(()) => {
                self.settings = next;
                true
            }
            Err(e) => {
                self.error = format!("Save failed: {e}");
                false
            }
        }
    }
    /// Shared port layout for drawing and pointer input; not yet native hitbox conformance.
    pub fn button_rect(&self, i: usize) -> (i32, i32, u32, u32) {
        // Steam Draw_64: origin (320,y), long hitbox +-90, 29 high, stride 34.
        // Selected/unselected sprite alpha differs; the hitbox does not shrink.
        let y = match self.initStep {
            0 | 2 => 156,
            3 => 171,
            _ => 226,
        };
        (230, y + i as i32 * 34, 180, 29)
    }

    pub fn MouseOverButton(&self, p: (f64, f64)) -> Option<usize> {
        (0..self.options().len()).find(|&i| {
            let (x, y, w, h) = self.button_rect(i);
            p.0 >= x as f64
                && p.0 < (x + w as i32) as f64
                && p.1 >= y as f64
                && p.1 < (y + h as i32) as f64
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn dir() -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "holocure-settings-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&p).unwrap();
        p
    }
    fn wizard() -> GameManager {
        let mut g = GameManager::new(Settings::default());
        g.EnterKey();
        for _ in 0..10 {
            g.step();
        }
        g
    }
    #[test]
    fn arrow_animation_matches_native_float32_samples() {
        let mut g = GameManager::new(Settings::default());
        for _ in 0..6 {
            g.step();
        }
        assert_eq!(g.image_index, 0.30000001192092896_f32);
        for _ in 6..252 {
            g.step();
        }
        assert_eq!(g.image_index, 12.600029945373535_f32);
    }
    #[test]
    fn five_steps_and_two_independent_permissions() {
        let mut g = wizard();
        assert_eq!(g.initStep, 1);
        g.inputText = "Test".into();
        g.EnterKey();
        assert_eq!(g.initStep, 2);
        g.currentOption = 1;
        g.Confirmed();
        assert!(!g.settings.hiscorenames);
        g.currentOption = 0;
        g.Confirmed();
        assert!(g.settings.hiscoreName);
        assert_eq!(g.initStep, 4);
        assert!(g.Confirmed());
        assert!(!g.settings.readyToStart);
    }
    #[test]
    fn rejection_does_not_erase_local_name() {
        let mut g = wizard();
        g.inputText = "Test".into();
        g.EnterKey();
        g.Confirmed();
        g.currentOption = 1;
        g.Confirmed();
        assert!(!g.settings.hiscoreName);
        assert_eq!(g.settings.username, "Test");
    }
    #[test]
    fn empty_name_and_input_delay() {
        let mut g = GameManager::new(Settings::default());
        g.EnterKey();
        g.ReceiveInput("x");
        assert_eq!(g.inputText, "Player");
        for _ in 0..10 {
            g.step();
        }
        g.inputText = "  ".into();
        assert!(!g.EnterKey());
        assert_eq!(g.initStep, 1);
    }
    #[test]
    fn unicode_backspace() {
        let mut g = wizard();
        g.inputText.clear();
        g.ReceiveInput("猫é");
        g.Backspace();
        assert_eq!(g.inputText, "猫");
    }
    #[test]
    fn restart_returns_to_language() {
        let mut g = wizard();
        g.EnterKey();
        g.Confirmed();
        g.Confirmed();
        g.currentOption = 1;
        assert!(!g.Confirmed());
        assert_eq!(g.initStep, 0);
    }
    #[test]
    fn roundtrip_preserves_names_flags_and_unknown_fields() {
        let d = dir();
        fs::write(
            d.join("settings.json"),
            br#"{"unknown":17,"username":"Test","hiscoreName":false}"#,
        )
        .unwrap();
        let mut s = Settings::LoadSettings(&d).unwrap();
        assert!(!s.readyToStart);
        s.readyToStart = true;
        s.SaveSettings(&d).unwrap();
        let t = Settings::LoadSettings(&d).unwrap();
        assert!(t.readyToStart);
        assert!(!t.hiscoreName);
        assert_eq!(t.extra["unknown"], 17);
        assert!(d.join("settings.json.bak").exists());
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn corrupt_settings_are_not_overwritten() {
        let d = dir();
        fs::write(d.join("settings.json"), "broken").unwrap();
        assert!(Settings::LoadSettings(&d).is_err());
        assert_eq!(
            fs::read_to_string(d.join("settings.json")).unwrap(),
            "broken"
        );
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn save_failure_does_not_complete_wizard() {
        let d = dir();
        fs::write(d.join("settings.json.tmp"), "existing transaction").unwrap();
        let mut g = wizard();
        assert!(!g.Complete(&d));
        assert!(!g.settings.readyToStart);
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn pointer_uses_native_long_hitbox() {
        let mut g = GameManager::new(Settings::default());
        for (step, y, count) in [(0, 156, 3), (2, 156, 2), (3, 171, 2), (4, 226, 2)] {
            g.initStep = step;
            for i in 0..count {
                let top = y + i as i32 * 34;
                assert_eq!(g.button_rect(i), (230, top, 180, 29));
                assert_eq!(g.MouseOverButton((230., top as f64)), Some(i));
                assert_eq!(g.MouseOverButton((409.999, top as f64 + 28.999)), Some(i));
                assert_eq!(g.MouseOverButton((410., top as f64)), None);
                assert_eq!(g.MouseOverButton((229.999, top as f64)), None);
                assert_eq!(g.MouseOverButton((320., top as f64 + 29.)), None);
                assert_eq!(g.MouseOverButton((320., top as f64 - 0.001)), None);
            }
        }
        g.initStep = 1;
        assert_eq!(g.MouseOverButton((320., 170.)), None);
    }
}

//! rm_HiScores state with explicit offline/official read-only providers. Native names retained; provider schema is PORT-specific.
//! Network authentication is independent; no Steam credential files are accessed.
#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub id: String,
    pub text_key: String,
    #[serde(default)]
    pub sprite: Option<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema: u32,
    pub stages: Vec<Choice>,
    pub characters: Vec<Choice>,
}
impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        let c: Self = crate::language::parse(path)?;
        if c.schema != 1
            || c.stages.is_empty()
            || c.characters.is_empty()
            || c.stages.len() > 32
            || c.characters.len() > 100
        {
            return Err("Invalid scores catalog".into());
        }
        Ok(c)
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Score {
    pub username: String,
    pub stage: String,
    pub character: String,
    pub score: u64,
    pub duration_seconds: u32,
    pub level: u32,
    #[serde(default)]
    pub daily: bool,
    #[serde(default)]
    pub is_player: bool,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ScoreFile {
    schema: u32,
    records: Vec<Score>,
}
/// Escribe solo el fichero local propio del port (perfil aislado).
pub fn write_local(path: &Path, rows: &[Score]) -> Result<(), String> {
    let file = ScoreFile {
        schema: 1,
        records: rows.to_vec(),
    };
    let json = serde_json::to_vec_pretty(&file).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, json).map_err(|e| e.to_string())
}

/// Read-only provider: never writes results, authenticates, posts, or reads native saves.
pub fn read_local(path: &Path) -> Result<Vec<Score>, String> {
    let size = match fs::metadata(path) {
        Ok(x) => x.len(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e.to_string()),
    };
    if size > 4 * 1024 * 1024 {
        return Err("Local scores file exceeds 4 MiB".into());
    }
    let f: ScoreFile = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if f.schema != 1
        || f.records.len() > 10000
        || f.records.iter().any(|r| {
            r.username.chars().count() > 64
                || r.username.chars().any(char::is_control)
                || r.score > 999_999_999_999
        })
    {
        return Err("Invalid local scores schema or record limits".into());
    }
    Ok(f.records)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    ReturnMenu,
    SaveSettings,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Button(usize),
    Arrow(usize, i32),
    Page(usize),
    Row(usize),
}
pub struct HiScores {
    pub background: crate::scenes::menu_triangles::MenuTriangles,
    pub server: Option<crate::score_server::Server>,
    pub currentOption: usize,
    pub selectedStage: usize,
    pub selectedCharacter: usize,
    pub timeOption: usize,
    pub myScoreOnly: bool,
    pub startingPosition: usize,
    pub showOptions: bool,
    pub changingName: bool,
    pub deleteConfirm: bool,
    pub currentSettingsOption: usize,
    pub showScores: bool,
    pub showScoresMenuSlideCurrent: u8,
    pub showScoresMenuSlideDuration: u8,
    pub showScoresMenuSlideNormalized: f64,
    pub showScoresIndex: usize,
    pub config: Config,
    pub showingArray: Vec<Score>,
    pub error: Option<String>,
    pub status: &'static str,
    all: Vec<Score>,
    path: PathBuf,
    last_attempt: Option<f64>,
    pub settings: crate::first_run::Settings,
}
impl HiScores {
    pub fn new(config: Config, path: PathBuf, settings: crate::first_run::Settings) -> Self {
        Self {
            server: None,
            background: crate::scenes::menu_triangles::MenuTriangles::default(),
            currentOption: 1,
            selectedStage: 0,
            selectedCharacter: 0,
            timeOption: 0,
            myScoreOnly: false,
            startingPosition: 0,
            showOptions: false,
            changingName: false,
            deleteConfirm: false,
            currentSettingsOption: 0,
            showScores: false,
            showScoresMenuSlideCurrent: 0,
            showScoresMenuSlideDuration: 8,
            showScoresMenuSlideNormalized: 0.,
            showScoresIndex: 0,
            config,
            showingArray: vec![],
            all: vec![],
            path,
            last_attempt: None,
            settings,
            error: None,
            status: "local",
        }
    }
    /// Status codes are log identifiers only: they are never rendered in the UI.
    pub fn set_status(&mut self, code: &'static str) {
        if self.status != code {
            println!("SCORES_STATUS {code}");
            self.status = code;
        }
    }
    /// Native Create/Step: eight fixed steps, reversible and clamped.
    pub fn Step(&mut self) {
        self.background.step();
        let query = crate::score_server::Query {
            stage: self.config.stages[self.selectedStage].id.clone(),
            character: self.config.characters[self.selectedCharacter].id.clone(),
            daily: self.timeOption != 0,
            // "My Score" is a client-side filter: the backend never receives identity.
        };
        let result = self.server.as_mut().and_then(|s| s.poll(query));
        match result {
            Some(crate::score_server::Poll::Waiting) => {
                self.all.clear();
                self.showingArray.clear();
                self.set_status("server_loading");
            }
            Some(crate::score_server::Poll::Ready(rows)) => {
                self.all = rows;
                self.error = None;
                self.EvaluatePlayerScores();
                self.set_status("server_ready");
                println!("SCORES_SERVER_READ_OK rows={}", self.showingArray.len());
            }
            Some(crate::score_server::Poll::Failed(e)) => {
                eprintln!("SCORES_SERVER_ERROR {e}");
                self.error = Some(e);
                self.set_status("server_error");
            }
            None => {}
        }

        self.showScoresMenuSlideCurrent = if self.showScores {
            self.showScoresMenuSlideCurrent
                .saturating_add(1)
                .min(self.showScoresMenuSlideDuration)
        } else {
            self.showScoresMenuSlideCurrent.saturating_sub(1)
        };
        self.showScoresMenuSlideNormalized =
            self.showScoresMenuSlideCurrent as f64 / self.showScoresMenuSlideDuration as f64;
    }
    pub fn CanFetchLeaderboard(&self, now: f64) -> bool {
        self.last_attempt.is_none_or(|last| now >= last + 5.)
    }
    pub fn FetchScores(&mut self, now: f64) {
        if let Some(server) = self.server.as_mut() {
            server.refresh();
            self.set_status("server_loading");
            return;
        }

        if !self.CanFetchLeaderboard(now) {
            self.set_status("cooldown");
            return;
        }
        self.last_attempt = Some(now);
        match read_local(&self.path) {
            Ok(rows) => {
                self.all = rows;
                self.error = None;
                self.set_status("local");
                self.EvaluatePlayerScores();
            }
            Err(e) => {
                self.error = Some(e);
                self.set_status("read_error");
            }
        }
    }
    pub fn EvaluatePlayerScores(&mut self) {
        let stage = &self.config.stages[self.selectedStage].id;
        let character = &self.config.characters[self.selectedCharacter].id;
        self.showingArray = self
            .all
            .iter()
            .filter(|r| {
                r.stage == *stage
                    && (character == "all" || r.character == *character)
                    && (self.timeOption == 0 || r.daily)
                    && (!self.myScoreOnly || r.is_player)
            })
            .cloned()
            .collect();
        // Stable sorting is a local-provider policy; native tie-break rules are not claimed.
        self.showingArray.sort_by(|a, b| b.score.cmp(&a.score));
        self.startingPosition = 0;
        self.showScores = false;
    }
    pub fn page_count(&self) -> usize {
        self.showingArray.len().div_ceil(10).max(1)
    }
    pub fn page(&mut self, p: usize) {
        self.startingPosition = p.min(self.page_count() - 1) * 10;
        self.showScores = false;
    }
    pub fn SelectUpDown(&mut self, dir: i32) {
        if self.deleteConfirm {
            self.currentSettingsOption = if dir < 0 { 6 } else { 7 };
            return;
        }
        if self.changingName {
            return;
        }
        if self.showScores {
            if !self.showingArray.is_empty() {
                self.showScoresIndex = (self.showScoresIndex as i32 + dir)
                    .clamp(0, self.showingArray.len() as i32 - 1)
                    as usize;
                self.startingPosition = self.showScoresIndex / 10 * 10;
            }
            return;
        }
        if self.showOptions {
            self.currentSettingsOption =
                (self.currentSettingsOption as i32 + dir).clamp(0, 5) as usize;
        } else {
            self.currentOption = (self.currentOption as i32 + dir).clamp(0, 6) as usize;
        }
    }
    pub fn SelectLeftRight(&mut self, dir: i32) {
        if self.changingName {
            self.currentSettingsOption = if dir < 0 { 6 } else { 7 };
            return;
        }
        if self.showOptions && self.currentSettingsOption == 0 {
            self.set_status("legacy_unavailable");
            return;
        }
        if self.showOptions || self.showScores {
            return;
        }
        match self.currentOption {
            1 => {
                self.selectedStage = (self.selectedStage as i32 + dir)
                    .rem_euclid(self.config.stages.len() as i32)
                    as usize
            }
            2 => {
                self.selectedCharacter = (self.selectedCharacter as i32 + dir)
                    .rem_euclid(self.config.characters.len() as i32)
                    as usize
            }
            3 => self.timeOption = (self.timeOption as i32 + dir).rem_euclid(2) as usize,
            _ => {
                let page = self.startingPosition / 10;
                self.page((page as i32 + dir).clamp(0, self.page_count() as i32 - 1) as usize);
                return;
            }
        }
        self.EvaluatePlayerScores();
        println!(
            "SCORES_FILTER_CHANGED stage={} character={} daily={} my_score={} rows={}",
            self.selectedStage,
            self.selectedCharacter,
            self.timeOption != 0,
            self.myScoreOnly,
            self.showingArray.len()
        );
        self.set_status(if self.server.is_some() {
            "server_loading"
        } else {
            "local"
        });
    }
    /// Name field input: own profile only, never the server.
    pub fn rename_input(&mut self, text: &str) {
        if !self.changingName {
            return;
        }
        let name: String = text.chars().take(24).filter(|c| !c.is_control()).collect();
        self.settings.username = name;
    }

    pub fn Confirmed(&mut self, now: f64) -> Action {
        if self.showScores {
            return Action::None;
        }
        if self.showOptions {
            if self.changingName || self.deleteConfirm {
                if self.currentSettingsOption == 7 {
                    self.currentSettingsOption = if self.changingName { 1 } else { 4 };
                    self.changingName = false;
                    self.deleteConfirm = false;
                    return Action::None;
                }
                if self.changingName {
                    // The interface behaves the same: the name is applied and the modal closes.
                    println!(
                        "SCORES_RENAME_APPLIED_LOCALLY name={} remote_write=skipped",
                        self.settings.username
                    );
                    self.changingName = false;
                    self.currentSettingsOption = 1;
                    return Action::SaveSettings;
                }
                // Delete: applied to the port own profile; the server is never touched.
                println!("SCORES_DELETE_APPLIED_LOCALLY remote_write=skipped");
                self.all.retain(|r| !r.is_player);
                self.EvaluatePlayerScores();
                if let Err(e) = write_local(&self.path, &self.all) {
                    eprintln!("SCORES_DELETE_LOCAL_WRITE_FAILED {e}");
                }
                self.deleteConfirm = false;
                self.currentSettingsOption = 4;
                return Action::None;
            }
            match self.currentSettingsOption {
                0 => {
                    self.set_status("legacy_unavailable");
                    return Action::None;
                }
                1 => {
                    self.changingName = true;
                    return Action::None;
                }
                2 => self.settings.hiscorenames = !self.settings.hiscorenames,
                3 => self.settings.hiscoreName = !self.settings.hiscoreName,
                4 => {
                    self.deleteConfirm = true;
                    self.currentSettingsOption = 7;
                    return Action::None;
                }
                5 => {
                    self.showOptions = false;
                    return Action::None;
                }
                _ => return Action::None,
            }
            return Action::SaveSettings;
        }
        match self.currentOption {
            0 => {
                if !self.showingArray.is_empty() {
                    self.showScores = true;
                    self.showScoresIndex = self.startingPosition;
                }
            }
            1 => self.FetchScores(now),
            2 | 3 => self.SelectLeftRight(1),
            4 => {
                // Native: the row filters the leaderboard to the player's own scores.
                self.myScoreOnly = !self.myScoreOnly;
                self.EvaluatePlayerScores();
                if self.myScoreOnly && self.showingArray.is_empty() {
                    println!(
                        "SCORES_MY_SCORE_EMPTY stage={} character={} daily={}",
                        self.config.stages[self.selectedStage].id,
                        self.config.characters[self.selectedCharacter].id,
                        self.timeOption != 0
                    );
                }
                println!(
                    "SCORES_MY_SCORE_FILTER on={} rows={}",
                    self.myScoreOnly,
                    self.showingArray.len()
                );
            }
            5 => return Action::ReturnMenu,
            6 => {
                self.showOptions = true;
                self.currentSettingsOption = 0;
            }
            _ => {}
        }
        Action::None
    }
    pub fn ReturnMenu(&mut self) -> Action {
        if self.changingName || self.deleteConfirm {
            self.currentSettingsOption = if self.changingName { 1 } else { 4 };
            self.changingName = false;
            self.deleteConfirm = false;
            return Action::None;
        }

        if self.showScores {
            self.showScores = false;
            Action::None
        } else if self.showOptions {
            self.showOptions = false;
            Action::None
        } else {
            Action::ReturnMenu
        }
    }
    pub fn button_rect(i: usize) -> (i32, i32, u32, u32) {
        if i == 6 {
            (601, 263, 29, 29)
        } else {
            (486, 51 + i as i32 * 34, 148, 29)
        }
    }
    pub fn row_rect(i: usize) -> (i32, i32, u32, u32) {
        (
            105 + (i / 5) as i32 * 180,
            50 + (i % 5) as i32 * 58,
            174,
            51,
        )
    }
    pub fn hit(&self, p: (f64, f64)) -> Option<Hit> {
        let inside = |r: (i32, i32, u32, u32)| {
            p.0 >= r.0 as f64
                && p.0 < (r.0 + r.2 as i32) as f64
                && p.1 >= r.1 as f64
                && p.1 < (r.1 + r.3 as i32) as f64
        };
        if self.showScores {
            for i in 0..10 {
                let (x, y, w, h) = Self::row_rect(i);
                let index = self.startingPosition + i;
                if index < self.showingArray.len() && inside((x + 38, y, w, h)) {
                    return Some(Hit::Row(index));
                }
            }
            return None;
        }
        if self.showOptions {
            if self.changingName {
                return if inside((325, 200, 90, 30)) {
                    Some(Hit::Button(7))
                } else if inside((225, 200, 90, 30)) {
                    Some(Hit::Button(6))
                } else {
                    None
                };
            }
            if self.deleteConfirm {
                return if inside((275, 192, 90, 30)) {
                    Some(Hit::Button(7))
                } else if inside((275, 162, 90, 30)) {
                    Some(Hit::Button(6))
                } else {
                    None
                };
            }
            if inside((230, 94, 20, 20)) {
                return Some(Hit::Arrow(0, -1));
            }
            if inside((414, 94, 20, 20)) {
                return Some(Hit::Arrow(0, 1));
            }
            return (0..6)
                .find(|&i| inside((242, if i == 5 { 295 } else { 91 + i as i32 * 34 }, 180, 29)))
                .map(Hit::Button);
        }
        for i in 1..=3 {
            let y = 51 + i as i32 * 34;
            if inside((490, y, 22, 29)) {
                return Some(Hit::Arrow(i, -1));
            }
            if inside((608, y, 22, 29)) {
                return Some(Hit::Arrow(i, 1));
            }
        }
        for i in 0..7 {
            if inside(Self::button_rect(i)) {
                return Some(Hit::Button(i));
            }
        }
        let page = self.startingPosition / 10;
        let first = page / 10 * 10;
        for i in 0..10 {
            if first + i < self.page_count() && inside((181 + i as i32 * 20, 335, 18, 22)) {
                return Some(Hit::Page(first + i));
            }
        }
        None
    }
    pub fn click(&mut self, hit: Hit, now: f64) -> Action {
        match hit {
            Hit::Button(i) => {
                if self.showOptions {
                    self.currentSettingsOption = i;
                } else {
                    self.currentOption = i;
                }
                self.Confirmed(now)
            }
            Hit::Arrow(i, d) => {
                if self.showOptions {
                    self.currentSettingsOption = i;
                    self.SelectLeftRight(d);
                    return Action::None;
                }
                self.currentOption = i;
                self.SelectLeftRight(d);
                Action::None
            }
            Hit::Page(p) => {
                self.page(p);
                Action::None
            }
            Hit::Row(i) => {
                self.showScores = true;
                self.showScoresIndex = i;
                Action::None
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn state() -> HiScores {
        let c = |id: &str| Choice {
            id: id.into(),
            text_key: id.into(),
            sprite: None,
        };
        HiScores::new(
            Config {
                schema: 1,
                stages: vec![c("s1"), c("s2")],
                characters: vec![c("ame"), c("all")],
            },
            PathBuf::from("missing-fixture.json"),
            crate::first_run::Settings::default(),
        )
    }
    fn row(n: u64) -> Score {
        Score {
            username: format!("Test {n}"),
            stage: "s1".into(),
            character: "ame".into(),
            score: n,
            duration_seconds: 300,
            level: 1,
            daily: n % 2 == 0,
            is_player: n == 1,
        }
    }
    #[test]
    fn native_version_arrow_hitbox_stays_at_unselected_anchor() {
        let mut s = state();
        s.showOptions = true;
        assert_eq!(s.hit((230., 94.)), Some(Hit::Arrow(0, -1)));
        assert_eq!(s.hit((229., 104.)), None);
        assert_eq!(s.hit((240., 93.)), None);
        assert_eq!(s.hit((240., 114.)), None);
        assert_eq!(s.hit((250., 104.)), Some(Hit::Button(0)));
        assert_eq!(s.hit((414., 94.)), Some(Hit::Arrow(0, 1)));
    }
    #[test]
    fn native_short_hitbox_includes_top_left_not_right_or_bottom() {
        let mut s = state();
        s.showOptions = true;
        s.changingName = true;
        assert_eq!(s.hit((225., 200.)), Some(Hit::Button(6)));
        assert_eq!(s.hit((224., 200.)), None);
        assert_eq!(s.hit((315., 215.)), None);
        assert_eq!(s.hit((270., 230.)), None);
        // La interfaz queda completa: aplica el nombre en el perfil propio y cierra.
        assert_eq!(s.click(Hit::Button(6), 0.), Action::SaveSettings);
        assert!(!s.changingName);
        assert_eq!(s.currentSettingsOption, 1);
    }
    #[test]
    fn modal_cancel_navigation_and_version_arrows() {
        let mut s = state();
        s.showOptions = true;
        s.currentSettingsOption = 1;
        s.Confirmed(0.);
        s.SelectLeftRight(1);
        assert_eq!(s.currentSettingsOption, 7);
        s.Confirmed(0.);
        assert!(!s.changingName);
        s.path = std::env::temp_dir().join("holocure-test-delete-fixture.json");
        s.currentSettingsOption = 4;
        s.Confirmed(0.);
        assert_eq!(s.currentSettingsOption, 7);
        s.SelectUpDown(-1);
        assert_eq!(s.currentSettingsOption, 6);
        s.Confirmed(0.);
        // "Si" aplica el borrado en el perfil propio y cierra el modal.
        assert!(!s.deleteConfirm);
        assert_eq!(s.currentSettingsOption, 4);
        assert!(s.all.iter().all(|r| !r.is_player));
        let original = s.currentOption;
        let hit = s.hit((237., 105.)).unwrap();
        assert_eq!(hit, Hit::Arrow(0, -1));
        s.click(hit, 0.);
        assert_eq!(s.currentOption, original);
        assert_eq!(s.status, "legacy_unavailable");
    }
    #[test]
    fn filter_and_pagination() {
        let mut s = state();
        s.all = (0..23).map(row).collect();
        s.EvaluatePlayerScores();
        assert_eq!(s.page_count(), 3);
        assert_eq!(s.showingArray[0].score, 22);
        s.page(99);
        assert_eq!(s.startingPosition, 20);
        s.currentOption = 3;
        s.SelectLeftRight(1);
        assert_eq!(s.showingArray.len(), 12);
        assert_eq!(s.startingPosition, 0);
        s.currentOption = 1;
        s.SelectLeftRight(1);
        assert!(s.showingArray.is_empty());
        assert_eq!(s.page_count(), 1);
    }
    #[test]
    fn native_navigation_clamps_and_filters_wrap() {
        let mut s = state();
        s.currentOption = 0;
        s.SelectUpDown(-1);
        assert_eq!(s.currentOption, 0);
        s.currentOption = 6;
        s.SelectUpDown(1);
        assert_eq!(s.currentOption, 6);
        s.currentOption = 1;
        s.SelectLeftRight(-1);
        assert_eq!(s.selectedStage, 1);
    }
    #[test]
    fn cooldown_exact_boundary() {
        let mut s = state();
        s.last_attempt = Some(1.);
        assert!(!s.CanFetchLeaderboard(5.999));
        assert!(s.CanFetchLeaderboard(6.));
    }
    #[test]
    fn my_score_and_back_stack() {
        let mut s = state();
        s.all = (0..23).map(row).collect();
        s.EvaluatePlayerScores();
        s.currentOption = 4;
        s.Confirmed(0.);
        assert!(s.myScoreOnly);
        assert_eq!(s.showingArray.len(), 1);
        assert!(s.showingArray.iter().all(|r| r.is_player));
        s.Confirmed(0.);
        assert!(!s.myScoreOnly);
        assert_eq!(s.showingArray.len(), 23);
        assert_eq!(s.ReturnMenu(), Action::ReturnMenu);
        // The detail view does consume the back stack.
        s.currentOption = 0;
        s.Confirmed(0.);
        assert!(s.showScores);
        assert_eq!(s.ReturnMenu(), Action::None);
        assert!(!s.showScores);
        assert_eq!(s.ReturnMenu(), Action::ReturnMenu);
        s.currentOption = 6;
        s.Confirmed(0.);
        assert!(s.showOptions);
        assert_eq!(s.ReturnMenu(), Action::None);
    }
    #[test]
    fn empty_my_score_is_honest() {
        let mut s = state();
        s.currentOption = 4;
        s.Confirmed(0.);
        assert!(s.myScoreOnly);
        assert!(s.showingArray.is_empty());
        assert!(!s.showScores);
        assert_eq!(s.page_count(), 1);
    }
    #[test]
    fn hit_edges_and_arrows() {
        let s = state();
        assert_eq!(s.hit((486., 51.)), Some(Hit::Button(0)));
        assert_eq!(s.hit((634., 51.)), None);
        assert_eq!(s.hit((499., 95.)), Some(Hit::Arrow(1, -1)));
        assert_eq!(s.hit((615., 277.)), Some(Hit::Button(6)));
    }
    #[test]
    fn preferences_are_independent() {
        let mut s = state();
        s.showOptions = true;
        s.currentSettingsOption = 2;
        assert_eq!(s.Confirmed(0.), Action::SaveSettings);
        assert!(!s.settings.hiscorenames);
        assert!(!s.settings.hiscoreName);
        s.currentSettingsOption = 3;
        s.Confirmed(0.);
        assert!(s.settings.hiscoreName);
    }
    #[test]
    fn native_slide_eight_steps_and_reversal() {
        let mut s = state();
        s.showScores = true;
        for n in 1..=8 {
            s.Step();
            assert_eq!(s.showScoresMenuSlideCurrent, n);
            assert_eq!(s.showScoresMenuSlideNormalized, n as f64 / 8.);
        }
        s.Step();
        assert_eq!(s.showScoresMenuSlideCurrent, 8);
        s.showScores = false;
        for n in (0..8).rev() {
            s.Step();
            assert_eq!(s.showScoresMenuSlideCurrent, n);
        }
        s.Step();
        assert_eq!(s.showScoresMenuSlideCurrent, 0);
    }
    #[test]
    fn check_scores_opens_view_without_refresh() {
        let mut s = state();
        s.all = vec![row(1)];
        s.EvaluatePlayerScores();
        s.currentOption = 0;
        s.Confirmed(0.);
        assert!(s.showScores);
        assert!(s.last_attempt.is_none());
    }
    #[test]
    fn modal_actions_apply_locally_and_never_reach_the_server() {
        let mut s = state();
        s.path = std::env::temp_dir().join("holocure-test-modal-fixture.json");
        s.showOptions = true;
        // Cambiar nombre: se aplica en el perfil propio y el modal cierra.
        s.currentSettingsOption = 1;
        assert_eq!(s.Confirmed(0.), Action::None);
        assert!(s.changingName);
        assert_eq!(s.click(Hit::Button(6), 0.), Action::SaveSettings);
        assert!(!s.changingName);
        // Borrar: se aplica en el perfil propio y el modal cierra.
        s.currentSettingsOption = 4;
        assert_eq!(s.Confirmed(0.), Action::None);
        assert!(s.deleteConfirm);
        assert_eq!(s.click(Hit::Button(6), 0.), Action::None);
        assert!(!s.deleteConfirm);
        assert!(s.all.iter().all(|r| !r.is_player));
        // Cancel still closes without applying anything.
        s.currentSettingsOption = 1;
        assert_eq!(s.Confirmed(0.), Action::None);
        assert!(s.changingName);
        assert_eq!(s.click(Hit::Button(7), 0.), Action::None);
        assert!(!s.changingName);
        assert_eq!(s.ReturnMenu(), Action::None);
        assert!(!s.showOptions);
        // Reopening and leaving through the Quit row still works.
        s.currentOption = 6;
        s.Confirmed(0.);
        assert!(s.showOptions);
        s.currentSettingsOption = 5;
        assert_eq!(s.Confirmed(0.), Action::None);
        assert!(!s.showOptions);
    }

    #[test]
    fn my_score_filter_combines_with_stage_character_and_time() {
        let mut s = state();
        s.all = (0..23).map(row).collect();
        s.currentOption = 4;
        s.Confirmed(0.);
        assert!(s.myScoreOnly);
        // Character "all" + My Score: there is still a single own row.
        s.selectedCharacter = 1;
        s.EvaluatePlayerScores();
        assert_eq!(s.showingArray.len(), 1);
        // Daily time: the own row (n=1) is not daily, so the list is empty.
        s.timeOption = 1;
        s.EvaluatePlayerScores();
        assert!(s.showingArray.is_empty());
        // Disabling the filter brings back the active filter list.
        s.currentOption = 4;
        s.Confirmed(0.);
        assert!(!s.myScoreOnly);
        s.timeOption = 0;
        s.EvaluatePlayerScores();
        assert_eq!(s.showingArray.len(), 23);
        // The daily filter does reduce the list.
        s.timeOption = 1;
        s.EvaluatePlayerScores();
        assert_eq!(s.showingArray.len(), 12);
    }
    #[test]
    fn six_settings_and_safe_cancel_hitboxes() {
        let mut s = state();
        s.showOptions = true;
        s.SelectUpDown(100);
        assert_eq!(s.currentSettingsOption, 5);
        s.currentSettingsOption = 4;
        s.Confirmed(0.);
        assert_eq!(s.hit((320., 207.)), Some(Hit::Button(7)));
        s.click(Hit::Button(7), 0.);
        assert!(!s.deleteConfirm);
        assert!(s.showOptions);
    }
    #[test]
    fn corrupt_file_is_not_overwritten() {
        let p = std::env::temp_dir().join(format!("bad-scores-{}.json", std::process::id()));
        fs::write(&p, "broken").unwrap();
        assert!(read_local(&p).is_err());
        assert_eq!(fs::read_to_string(&p).unwrap(), "broken");
        fs::remove_file(p).unwrap();
    }
}

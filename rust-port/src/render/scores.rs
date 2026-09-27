//! Original room background and sprites; layout measured from Steam captures.
//! Offline provider and simplified modal layouts are explicitly port-specific.
use crate::{
    language::LanguagePack,
    render::{
        sprites::{self, Sprite},
        text::Font,
    },
    scenes::scores::HiScores,
};
use sdl2::{
    pixels::Color,
    rect::{FPoint, FRect, Rect},
    render::{BlendMode, Canvas, TextureCreator},
    video::{Window, WindowContext},
};
use serde_json::Value;
use std::{collections::HashMap, path::Path};
pub struct ScoresAssets<'a> {
    sprites: HashMap<String, Vec<Sprite<'a>>>,
}
impl<'a> ScoresAssets<'a> {
    pub fn load(
        tc: &'a TextureCreator<WindowContext>,
        root: &Path,
        config: &crate::scenes::scores::Config,
    ) -> Result<Self, String> {
        let meta: Value = serde_json::from_slice(
            &std::fs::read(root.join("sprites.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let rooms: Value = serde_json::from_slice(
            &std::fs::read(root.join("rooms.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let room = rooms
            .as_array()
            .and_then(|a| a.iter().find(|r| r["name"] == "rm_HiScores"))
            .ok_or("Missing rm_HiScores")?;
        if room["views"][0]["ViewWidth"] != 640 || room["views"][0]["ViewHeight"] != 360 {
            return Err("Unsupported scores room view".into());
        }
        let mut wanted = vec![
            ("hud_new_title_BG".to_string(), 1),
            ("menu_hiscoreButtons".into(), 2),
            ("menu_hiscoreSettingsButton".into(), 2),
            ("hud_scrollArrows2".into(), 2),
            ("hud_scrollArrows3".into(), 2),
            ("menu_first".into(), 3),
            ("menu_scorecontainer".into(), 1),
            ("hud_initButtons".into(), 2),
            ("hud_toggleButton".into(), 4),
            ("hud_confirmButton".into(), 1),
            ("hud_unselectButton".into(), 1),
            ("menu_charselec_triangle".into(), 1),
            ("hud_optionsmenu".into(), 1),
            ("hud_OptionButton".into(), 2),
            ("hud_hiscoreOptionIcons".into(), 10),
            ("spr_option_widebox".into(), 1),
            ("hud_quitConfirm".into(), 1),
            ("menu_scorecontainer_run".into(), 1),
            ("menu_scorecontainer_selected".into(), 1),
        ];
        for c in &config.characters {
            if let Some(n) = &c.sprite {
                wanted.push((n.clone(), 1));
            }
        }
        let mut loaded = HashMap::new();
        for (name, count) in wanted {
            let m = meta
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["name"] == name)
                .ok_or_else(|| format!("Missing scores sprite: {name}"))?;
            let frames = (0..count)
                .map(|i| sprites::load_sprite(tc, root, &name, i, m, None))
                .collect::<Result<Vec<_>, _>>()?;
            loaded.insert(name, frames);
        }
        Ok(Self { sprites: loaded })
    }
}
// Native Steam hover traces: change resource, never rescale the idle bitmap.
fn small_button(c: &mut Canvas<Window>, sprite: &Sprite, x: f32, y: f32) -> Result<(), String> {
    sprites::draw(c, sprite, x, y, 1., 1.)
}

fn text(
    c: &mut Canvas<Window>,
    font: &mut Font,
    s: &str,
    x: f32,
    y: f32,
    align: u8,
    col: Color,
) -> Result<(), String> {
    font.text(c, s, x, y + font.init_baseline, align, col)
}
// Steam draw_text_outline: radius 1, 16/32 angular samples collapse to
// these eight unique integer offsets when alpha is one (the Scores labels).
fn outlined(
    c: &mut Canvas<Window>,
    f: &mut Font,
    s: &str,
    x: f32,
    y: f32,
    align: u8,
    fill: Color,
) -> Result<(), String> {
    for (dx, dy) in [
        (-1., -1.),
        (0., -1.),
        (1., -1.),
        (-1., 0.),
        (1., 0.),
        (-1., 1.),
        (0., 1.),
        (1., 1.),
    ] {
        f.text(c, s, x + dx, y + dy, align, Color::BLACK)?;
    }
    f.text(c, s, x, y, align, fill)
}
fn large(
    c: &mut Canvas<Window>,
    f: &mut Font,
    s: &str,
    x: f32,
    y: f32,
    align: u8,
) -> Result<(), String> {
    outlined(c, f, s, x, y, align, Color::WHITE)
}
fn fit(f: &Font, s: &str, max: i32) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        let mut next = out.clone();
        next.push(ch);
        if f.width(&next) > max {
            break;
        }
        out = next;
    }
    out
}
pub fn draw(
    c: &mut Canvas<Window>,
    font: &mut Font,
    assets: &mut ScoresAssets,
    big: &mut Font,
    tiny: &mut Font,
    s: &HiScores,
    lang: &LanguagePack,
) -> Result<(), String> {
    let t = |key| lang.text(key);
    let white = Color::WHITE;
    let shift = 38. * s.showScoresMenuSlideNormalized as f32;
    sprites::draw(c, &assets.sprites["hud_new_title_BG"][0], 0., 0., 1., 1.)?;
    let triangle = &mut assets.sprites.get_mut("menu_charselec_triangle").unwrap()[0];
    for e in s.background.active.iter().rev() {
        if e.sx <= 0. || e.sy <= 0. {
            continue;
        }
        triangle
            .tex
            .set_alpha_mod((e.alpha.clamp(0., 1.) * 255.).round() as u8);
        c.copy_ex_f(
            &triangle.tex,
            None,
            FRect::new(
                e.x - triangle.ox * e.sx,
                e.y - triangle.oy * e.sy,
                triangle.w as f32 * e.sx,
                triangle.h as f32 * e.sy,
            ),
            -(e.angle as f64),
            Some(FPoint::new(triangle.ox * e.sx, triangle.oy * e.sy)),
            false,
            false,
        )?;
    }
    triangle.tex.set_alpha_mod(255);
    if let Some(name) = &s.config.characters[s.selectedCharacter].sprite {
        let art = &mut assets
            .sprites
            .get_mut(name)
            .ok_or("Missing character art")?[0];
        art.tex.set_alpha_mod(128);
        sprites::draw(c, art, 50., 350., 3., 3.)?;
    }
    let heading = if s.showScores {
        t("scores.details_heading")
            .replace("{stage}", t(&s.config.stages[s.selectedStage].text_key))
            .replace(
                "{period}",
                t(if s.timeOption == 0 {
                    "scores.period_all_time"
                } else {
                    "scores.period_daily"
                }),
            )
    } else {
        t(if s.timeOption == 0 {
            "scores.heading_all_time"
        } else {
            "scores.heading_daily"
        })
        .to_owned()
    };
    large(c, big, &heading, 285. + shift, 15., 1)?;
    // No provider badge in the UI: provider state is reported in the logs.
    for i in 0..10 {
        let idx = s.startingPosition + i;
        if let Some(row) = s.showingArray.get(idx) {
            let (x, y, _, _) = HiScores::row_rect(i);
            let x = x as f32 + shift;
            let container = if idx < 3 {
                &assets.sprites["menu_first"][idx]
            } else {
                &assets.sprites["menu_scorecontainer"][0]
            };
            sprites::draw(c, container, x as f32, y as f32, 1., 1.)?;
            if s.showScores && idx == s.showScoresIndex {
                sprites::draw(
                    c,
                    &assets.sprites["menu_scorecontainer_selected"][0],
                    x as f32 - 4.,
                    y as f32 - 4.,
                    1.,
                    1.,
                )?;
            }
            let name = if s.settings.hiscorenames {
                row.username.as_str()
            } else {
                t("scores.anonymous")
            };
            clipped(
                c,
                font,
                name,
                124,
                x as f32 + 12.,
                y as f32 + 5.,
                0,
                Color::YELLOW,
            )?;
            text(
                c,
                font,
                &t("scores.score_value").replace("{score}", &row.score.to_string()),
                x as f32 + 12.,
                y as f32 + 20.,
                0,
                white,
            )?;
            let line = t("scores.run_values")
                .replace(
                    "{time}",
                    &format!(
                        "{}:{:02}",
                        row.duration_seconds / 60,
                        row.duration_seconds % 60
                    ),
                )
                .replace("{level}", &row.level.to_string());
            clipped(
                c,
                tiny,
                &line,
                156,
                x as f32 + 12.,
                y as f32 + 36.,
                0,
                white,
            )?;
            text(
                c,
                font,
                t("scores.rank"),
                x as f32 + 167.,
                y as f32 + 4.,
                2,
                Color::RGB(190, 190, 190),
            )?;
            large(
                c,
                big,
                &(idx + 1).to_string(),
                x as f32 + 167.,
                y as f32 + 17.,
                2,
            )?;
        }
    }
    if s.showingArray.is_empty() {
        text(c, font, t("scores.empty"), 285., 168., 1, white)?;
    }
    // No status line in the UI: the status code is reported in the logs.
    if s.showScoresMenuSlideCurrent == 0 {
        for i in 0..6 {
            sprites::draw(
                c,
                &assets.sprites["menu_hiscoreButtons"]
                    [usize::from(s.currentOption == i && !s.showOptions)],
                560.,
                65. + i as f32 * 34.,
                1.,
                1.,
            )?;
            let label = match i {
                0 => t("scores.check"),
                1 => t(&s.config.stages[s.selectedStage].text_key),
                2 => t(&s.config.characters[s.selectedCharacter].text_key),
                3 => t(if s.timeOption == 0 {
                    "scores.all_time"
                } else {
                    "scores.daily"
                }),
                4 => t("scores.my_score"),
                _ => t("scores.quit"),
            };
            text(
                c,
                font,
                label,
                560.,
                60. + i as f32 * 34.,
                1,
                if s.currentOption == i && !s.showOptions {
                    Color::BLACK
                } else {
                    white
                },
            )?;
            if (1..=3).contains(&i) {
                for (frame, x) in [(0, 500.), (1, 620.)] {
                    sprites::draw(
                        c,
                        &assets.sprites["hud_scrollArrows2"][frame],
                        x,
                        65. + i as f32 * 34.,
                        1.,
                        1.,
                    )?;
                }
            }
        }
        sprites::draw(
            c,
            &assets.sprites["menu_hiscoreSettingsButton"][usize::from(s.currentOption == 6)],
            560.,
            277.,
            1.,
            1.,
        )?;
    }
    let page = s.startingPosition / 10;
    let first = page / 10 * 10;
    for i in 0..10 {
        let n = first + i;
        if n >= s.page_count() {
            break;
        }
        let x = 190. + shift + i as f32 * 20.;
        if n == page {
            large(c, big, &(n + 1).to_string(), x, 333., 1)?;
        } else {
            text(c, font, &(n + 1).to_string(), x, 341., 1, white)?;
        }
    }
    text(c, font, t("scores.controls"), 630., 345., 2, white)?;
    if s.showScoresMenuSlideCurrent > 0 {
        // The original equipment panel; local records do not contain equipment.
        // Horizontal entry is measured at endpoints; intermediate panel placement remains unvalidated.
        let x = 6. - 136. * (1. - s.showScoresMenuSlideNormalized as f32);
        sprites::draw(
            c,
            &assets.sprites["menu_scorecontainer_run"][0],
            x,
            55.,
            1.,
            1.,
        )?;
        for (key, y) in [
            ("scores.weapons", 65.),
            ("scores.items", 145.),
            ("scores.support", 225.),
        ] {
            text(c, font, t(key), x + 18., y, 0, white)?;
        }
    }
    if s.showOptions {
        c.set_blend_mode(BlendMode::Blend);
        c.set_draw_color(Color::RGBA(0, 0, 0, 64));
        c.fill_rect(Rect::new(0, 0, 640, 360))?;
        if !s.showOptions {
            c.set_draw_color(Color::RGB(33, 109, 152));
            c.fill_rect(Rect::new(200, 90, 240, 205))?;
            c.set_draw_color(white);
            c.draw_rect(Rect::new(200, 90, 240, 205))?;
        }
        if s.showOptions {
            sprites::draw(c, &assets.sprites["hud_optionsmenu"][0], 320., 48., 1., 1.)?;
            outlined(
                c,
                big,
                t("scores.settings_heading"),
                320.,
                61.,
                1,
                Color::BLACK,
            )?;
            large(c, big, t("scores.settings_heading"), 320., 58., 1)?;
            for i in 0..6 {
                let y = if i == 5 { 295. } else { 91. + i as f32 * 34. };
                let selected = s.currentSettingsOption == i
                    || (s.changingName && i == 1)
                    || (s.deleteConfirm && i == 4);
                sprites::draw(
                    c,
                    &assets.sprites["hud_OptionButton"][usize::from(selected)],
                    332.,
                    y,
                    1.,
                    1.,
                )?;
                let key = match i {
                    0 => "scores.legacy_label",
                    1 => "scores.rename",
                    2 => "scores.show_names_label",
                    3 => "scores.use_name_label",
                    4 => "scores.delete",
                    _ => "scores.quit",
                };
                let col = if i == 4 {
                    Color::RGB(255, 0, 0)
                } else if selected {
                    Color::BLACK
                } else {
                    white
                };
                text(c, font, t(key), 254., y + 8., 0, col)?;
                if i <= 4 {
                    sprites::draw(
                        c,
                        &assets.sprites["hud_hiscoreOptionIcons"][i * 2 + usize::from(selected)],
                        222.,
                        y + 13.,
                        1.,
                        1.,
                    )?;
                }
                if i == 0 {
                    // Measured in Steam: selected arrows shift outward by four pixels.
                    let (name, left, right) = if selected {
                        ("hud_scrollArrows3", 236., 428.)
                    } else {
                        ("hud_scrollArrows2", 240., 424.)
                    };
                    for (frame, x) in [(0, left), (1, right)] {
                        sprites::draw(c, &assets.sprites[name][frame], x, y + 13., 1., 1.)?;
                    }
                }
                if i == 2 || i == 3 {
                    let checked = if i == 2 {
                        s.settings.hiscorenames
                    } else {
                        s.settings.hiscoreName
                    };
                    let frame = usize::from(checked) + 2 * usize::from(selected);
                    sprites::draw(
                        c,
                        &assets.sprites["hud_toggleButton"][frame],
                        401.,
                        y + 13.,
                        1.,
                        1.,
                    )?;
                }
            }
            // No read-only notice in Settings: it is reported in the logs.
            if s.changingName {
                sprites::draw(
                    c,
                    &assets.sprites["spr_option_widebox"][0],
                    320.,
                    130.,
                    1.,
                    1.,
                )?;
                large(c, font, t("scores.rename_prompt"), 320., 145., 1)?;
                c.set_draw_color(Color::RGB(39, 90, 113));
                c.fill_rect(Rect::new(229, 175, 182, 22))?;
                c.set_draw_color(white);
                c.draw_rect(Rect::new(229, 175, 182, 22))?;
                text(c, font, &s.settings.username, 239., 180., 0, white)?;
                for (x, key, _disabled, index) in [
                    (270., "scores.confirm", true, 6),
                    (370., "scores.cancel", false, 7),
                ] {
                    let selected = s.currentSettingsOption == index;
                    small_button(
                        c,
                        &assets.sprites[if selected {
                            "hud_confirmButton"
                        } else {
                            "hud_unselectButton"
                        }][0],
                        x,
                        215.,
                    )?;
                    text(
                        c,
                        font,
                        t(key),
                        x,
                        210.,
                        1,
                        if selected { Color::BLACK } else { white },
                    )?;
                }
            } else if s.deleteConfirm {
                sprites::draw(c, &assets.sprites["hud_quitConfirm"][0], 320., 120., 1., 1.)?;
                text(c, font, t("scores.delete"), 320., 133., 1, white)?;
                for (y, key, _disabled, index) in
                    [(177., "scores.yes", true, 6), (207., "scores.no", false, 7)]
                {
                    let selected = s.currentSettingsOption == index;
                    small_button(
                        c,
                        &assets.sprites[if selected {
                            "hud_confirmButton"
                        } else {
                            "hud_unselectButton"
                        }][0],
                        320.,
                        y,
                    )?;
                    text(
                        c,
                        font,
                        t(key),
                        320.,
                        y - 6.,
                        1,
                        if selected { Color::BLACK } else { white },
                    )?;
                }
            }
        }
    }
    // No error text in the UI: it is reported with eprintln!.
    Ok(())
}

fn clipped(
    c: &mut Canvas<Window>,
    font: &mut Font,
    s: &str,
    max: i32,
    x: f32,
    y: f32,
    align: u8,
    col: Color,
) -> Result<(), String> {
    let value = fit(font, s, max);
    text(c, font, &value, x, y, align, col)
}

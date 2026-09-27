#![allow(non_snake_case)] // Preserve recovered GameMaker resource names.
//! rm_InitRoom Draw_64 geometry from Steam YYC 0x1423372e0 and original resources.
//! Android GML corroborates steps 0/1/2; Steam adds consent and changes summary.
use crate::{
    first_run::GameManager,
    render::{
        sprites::{self, Sprite},
        text::Font,
    },
};
use sdl2::{
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
};
use serde_json::Value;
use std::path::Path;
pub struct InitRoomAssets<'a> {
    hud_initButtons: Vec<Sprite<'a>>,
    spr_enterArrow: Vec<Sprite<'a>>,
    pub language: crate::language::LanguagePack,
    pub languages: Vec<crate::language::Manifest>,
}
impl<'a> InitRoomAssets<'a> {
    pub fn load(tc: &'a TextureCreator<WindowContext>, root: &Path) -> Result<Self, String> {
        let data: Value = serde_json::from_slice(
            &std::fs::read(root.join("sprites.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let rooms: Value = serde_json::from_slice(
            &std::fs::read(root.join("rooms.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let room = rooms
            .as_array()
            .and_then(|r| r.iter().find(|r| r["name"] == "rm_InitRoom"))
            .ok_or("Missing rm_InitRoom")?;
        let view = &room["views"][0];
        if view["ViewWidth"] != 640 || view["ViewHeight"] != 360 {
            return Err("Unsupported rm_InitRoom view geometry".into());
        }
        let load = |name: &str| -> Result<Vec<Sprite<'a>>, String> {
            let meta = data
                .as_array()
                .and_then(|a| a.iter().find(|v| v["name"] == name))
                .ok_or_else(|| format!("Missing sprite {name}"))?;
            (0..2)
                .map(|frame| sprites::load_sprite(tc, root, name, frame, meta, None))
                .collect()
        };
        let language_root = std::env::var_os("HOLOCURE_LANGUAGE_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| root.join("languages"));
        let language = crate::language::LanguagePack::load(&language_root, "eng")?;
        let languages = crate::language::catalog(&language_root)?;
        Ok(Self {
            language,
            languages,
            hud_initButtons: load("hud_initButtons")?,
            spr_enterArrow: load("spr_enterArrow")?,
        })
    }
}
pub fn draw(
    c: &mut Canvas<Window>,
    font: &mut Font,
    g: &GameManager,
    assets: &InitRoomAssets,
) -> Result<(), String> {
    let white = Color::WHITE;
    let t = |key| assets.language.text(key);
    // Native hint is clipped by the 640-wide view. Keep its placement; do not invent a margin.
    // image_speed=0.05: Android Create and Steam passive samples agree (float32 runner).
    sprites::draw(
        c,
        &assets.spr_enterArrow[(g.image_index.floor() as usize) % 2],
        569.,
        333.,
        1.,
        1.,
    )?;
    native_text(c, font, t("first_run.hint"), 590., 335., 0, white)?;
    match g.initStep {
        0 => native_text(c, font, t("first_run.language"), 320., 120., 1, white)?, // English UI policy; native heading is multilingual.
        1 => {
            native_text(c, font, t("first_run.name"), 320., 120., 1, white)?;
            c.set_draw_color(white);
            // GameMaker rectangle endpoints are inclusive (255,165)-(385,185).
            // Device-pixel outline calibrated against the 1280x720 Steam capture.
            // Unlike SDL scaled draw_rect, native outlines stay one device pixel wide.
            let (sx, sy) = c.scale();
            c.set_scale(1., 1.)?;
            let result = c.draw_rect(Rect::new(
                (255. * sx).round() as i32 - 1,
                (165. * sy).round() as i32 - 1,
                (131. * sx).round() as u32 + 2,
                (21. * sy).round() as u32 + 2,
            ));
            c.set_scale(sx, sy)?;
            result?;
            native_text(c, font, &g.inputText, 264., 170., 0, white)?;
            if g.rectVis {
                c.fill_rect(Rect::new(264 + font.width(&g.inputText), 169, 8, 13))?;
            }
        }
        2 => {
            native_text(c, font, t("first_run.show_prompt"), 320., 120., 1, white)?;
            for (i, line) in t("first_run.warning").split('\n').enumerate() {
                native_text(c, font, line, 320., 250. + i as f32 * 15., 1, white)?;
            }
        }
        3 => {
            native_text(
                c,
                font,
                &t("first_run.named_player").replace("{username}", &g.settings.username),
                320.,
                70.,
                1,
                Color::YELLOW,
            )?;
            for (i, line) in t("first_run.consent").split('\n').enumerate() {
                native_text(c, font, line, 320., 105. + i as f32 * 15., 1, white)?;
            }
        }
        _ => {
            // Preserve the native label/value pairing, including its confusing final two labels.
            let rows = [
                (
                    t("first_run.language"),
                    assets
                        .languages
                        .iter()
                        .find(|l| l.id == g.settings.CurrentLanguage)
                        .map(|l| l.name.as_str())
                        .unwrap_or(&g.settings.CurrentLanguage),
                ),
                (t("first_run.name"), g.settings.username.as_str()),
                (
                    t("first_run.summary_display"),
                    if g.settings.hiscorenames {
                        t("first_run.show")
                    } else {
                        t("first_run.hide")
                    },
                ),
                (
                    t("first_run.summary_consent"),
                    if g.settings.hiscoreName {
                        t("first_run.allow")
                    } else {
                        t("first_run.decline")
                    },
                ),
            ];
            for (i, (label, value)) in rows.iter().enumerate() {
                let y = 90. + i as f32 * 30.;
                native_text(c, font, label, 200., y, 0, white)?;
                native_text(c, font, value, 400., y, 0, white)?;
            }
        }
    }
    for (i, key) in g.options().iter().enumerate() {
        let label = if g.initStep == 0 {
            assets
                .languages
                .iter()
                .find(|l| l.id == *key)
                .map(|l| l.name.as_str())
                .unwrap_or(key)
        } else {
            t(key)
        };
        let selected = i == g.currentOption;
        let (_, y, _, _) = g.button_rect(i);
        sprites::draw(
            c,
            &assets.hud_initButtons[usize::from(selected)],
            320.,
            y as f32,
            1.,
            1.,
        )?;
        native_text(
            c,
            font,
            label,
            320.,
            y as f32 + if g.initStep == 3 { 10. } else { 9. },
            1,
            if selected { Color::BLACK } else { white },
        )?;
    }
    if !g.error.is_empty() {
        native_text(
            c,
            font,
            t("first_run.error"),
            320.,
            305.,
            1,
            Color::RGB(255, 140, 140),
        )?;
    }
    Ok(())
}

// Original jpFont atlas needs a -1 logical-pixel baseline correction in this SDL path.
// Measured independently in native heading, warning, name and button text regions.
fn native_text(
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

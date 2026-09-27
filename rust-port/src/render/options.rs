//! `rm_Options` drawing, taken statement by statement from the original's own
//! `obj_Options` `Draw` event. Every coordinate below is that event's arithmetic
//! on `container = [320, 48]`, unchanged.
//!
//! Two quirks are reproduced rather than fixed, because the brief is a
//! bug-for-bug baseline:
//!
//! * `optionButtons` holds ten labels but `maxOptions[0]` is 7, and the labels
//!   from index 5 on do not line up with what those rows do. Row 5 is labelled
//!   `Vibration` and changes the language; row 6 is labelled `Language` and
//!   toggles the hiscore-name setting. The last three labels are unreachable.
//! * `graphicOptions` holds thirteen labels but `maxOptions[1]` is 12, so
//!   `Show Stamps` is unreachable and `showStamps` has no row at all.
use crate::{
    language::LanguagePack,
    render::{
        outline::outline_offsets,
        sprites::{self, Sprite},
        text::Font,
    },
    scenes::options::Options,
};
use sdl2::{
    pixels::Color,
    rect::Rect,
    render::{BlendMode, Canvas, TextureCreator},
    video::{Window, WindowContext},
};
use serde_json::Value;
use std::{collections::HashMap, path::Path};

/// Rows the original draws, whatever the page length.
const ROWS: usize = 7;

/// Sprite names this room draws, with the frame count each is used at.
const SPRITES: [(&str, usize); 12] = [
    ("hud_optionsmenu", 1),
    ("hud_OptionButton", 2),
    ("hud_optionIcons", 22),
    ("hud_graphicIcons", 26),
    ("hud_sliderBar", 1),
    ("hud_slider", 1),
    ("hud_toggleButton", 4),
    ("hud_scrollArrows", 2),
    ("hud_scrollArrows2", 2),
    ("hud_keybindIcons", 16),
    ("hud_controllerButtonIcons", 10),
    ("spr_brackets", 1),
];

pub struct OptionsAssets<'a> {
    sprites: HashMap<String, Vec<Sprite<'a>>>,
}

impl<'a> OptionsAssets<'a> {
    pub fn load(tc: &'a TextureCreator<WindowContext>, root: &Path) -> Result<Self, String> {
        let meta: Value = serde_json::from_slice(
            &std::fs::read(root.join("options-manifest.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let mut sprites = HashMap::new();
        for (name, frames) in SPRITES {
            let entry = &meta["sprites"][name];
            if entry.is_null() {
                return Err(format!("Missing {name} in options-manifest.json"));
            }
            let mut v = Vec::with_capacity(frames);
            for f in 0..frames {
                v.push(sprites::load_sprite(tc, root, name, f, entry, None)?);
            }
            sprites.insert(name.to_string(), v);
        }
        Ok(Self { sprites })
    }

    fn frame(&self, name: &str, index: usize) -> Result<&Sprite<'_>, String> {
        self.sprites
            .get(name)
            .and_then(|v| v.get(index))
            .ok_or_else(|| format!("Missing {name} frame {index}"))
    }
}

/// `draw_text_outline`: the original draws the text at a radius of 1 with 32
/// angular samples, which is eight offset copies plus the fill.
fn outlined(
    f: &mut Font,
    c: &mut Canvas<Window>,
    text: &str,
    x: f32,
    y: f32,
    align: u8,
    fill: Color,
) -> Result<(), String> {
    for (dx, dy) in outline_offsets(1.0, 32) {
        f.text(c, text, x + dx, y + dy, align, Color::BLACK)?;
    }
    f.text(c, text, x, y, align, fill)
}

/// Draw the room. `room` is the state machine; nothing here mutates it, exactly
/// like the original's `Draw` event, which only reads state and moves the cursor
/// when the pointer is over a row.
pub fn draw(
    c: &mut Canvas<Window>,
    font: &mut Font,
    big: &mut Font,
    assets: &mut OptionsAssets,
    room: &Options,
    lang: &LanguagePack,
) -> Result<(), String> {
    let white = Color::WHITE;
    let black = Color::BLACK;

    // container[0], container[1]
    let cx = 320.0f32;
    let cy = 48.0f32;

    sprites::draw(c, assets.frame("hud_optionsmenu", 0)?, cx, cy, 1., 1.)?;

    if room.keybind_menu {
        draw_keybinds(c, font, big, assets, room, lang, cx, cy, white, black)
    } else if room.controller_menu {
        draw_controller(c, font, big, assets, room, lang, cx, cy, white, black)
    } else {
        draw_pages(c, font, big, assets, room, lang, cx, cy, white, black)
    }?;

    // The scrollbar belongs to the two option pages only.
    if !room.keybind_menu && !room.controller_menu {
        let max = if room.option_page == 0 { 7.0 } else { 12.0 };
        let rect_height = 1540.0 / max;
        let scroll_dist = 220.0 / max;
        let top = cy + 44.0 + scroll_dist * room.show_option_range as f32;
        c.set_draw_color(white);
        c.fill_rect(Rect::new(
            (cx + 108.0) as i32,
            top as i32,
            2,
            rect_height.max(0.0) as u32,
        ))?;
        c.set_draw_color(black);
        sprites::draw(
            c,
            assets.frame("hud_scrollArrows", 0)?,
            cx + 109.0,
            cy + 39.0,
            1.,
            1.,
        )?;
        sprites::draw(
            c,
            assets.frame("hud_scrollArrows", 1)?,
            cx + 109.0,
            cy + 269.0,
            1.,
            1.,
        )?;
    }
    Ok(())
}

/// The two option pages, including the row icons, labels and value widgets.
fn draw_pages(
    c: &mut Canvas<Window>,
    font: &mut Font,
    big: &mut Font,
    assets: &mut OptionsAssets,
    room: &Options,
    lang: &LanguagePack,
    cx: f32,
    cy: f32,
    white: Color,
    black: Color,
) -> Result<(), String> {
    let t = |key: &str| lang.text(key).to_string();
    outlined(big, c, &t("options.title"), cx, cy + 10.0, 1, white)?;

    let icon_name = if room.option_page == 0 {
        "hud_optionIcons"
    } else {
        "hud_graphicIcons"
    };
    let label_prefix = if room.option_page == 0 {
        "options.page0"
    } else {
        "options.page1"
    };

    for i in 0..ROWS {
        let abs = i + room.show_option_range;
        let y_button = cy + 43.0 + (i as f32) * 34.0;
        let y_mid = cy + 56.0 + (i as f32) * 34.0;
        let y_text = cy + 51.0 + (i as f32) * 34.0;
        let col = if room.current_option == i { white } else { black };

        sprites::draw(
            c,
            assets.frame("hud_OptionButton", usize::from(room.current_option == i))?,
            cx + 12.0,
            y_button,
            1.,
            1.,
        )?;
        // The original indexes the icon sheet by the absolute row, times two,
        // plus one when the row is selected.
        let icon_frame = abs * 2 + usize::from(room.current_option == i);
        sprites::draw(c, assets.frame(icon_name, icon_frame)?, cx - 98.0, y_mid, 1., 1.)?;

        font.text(
            c,
            &t(&format!("{label_prefix}.{abs}")),
            cx - 66.0,
            y_text + font.init_baseline,
            0,
            col,
        )?;

        // Value widgets, one per row, exactly as the original's switch.
        if room.option_page == 0 {
            match abs {
                // Music volume slider.
                1 => slider(c, assets, cx, y_button, y_mid, room.settings.music_volume)?,
                // Sound volume slider.
                2 => slider(c, assets, cx, y_button, y_mid, room.settings.sound_volume)?,
                // Language selector: arrows at +-38, value centred at +52.
                5 => {
                    arrows(c, assets, cx + 52.0, y_mid, 38.0)?;
                    font.text(
                        c,
                        room.languages
                            .get(room.selected_language_option)
                            .map(String::as_str)
                            .unwrap_or(""),
                        cx + 52.0,
                        y_text + font.init_baseline,
                        1,
                        col,
                    )?;
                }
                // Hiscore-name toggle.
                6 => toggle(c, assets, cx + 81.0, y_mid, room.settings.hiscorenames, room.current_option == i)?,
                _ => {}
            }
        } else {
            match abs {
                // Resolution selector.
                0 => {
                    arrows(c, assets, cx + 52.0, y_mid, 38.0)?;
                    let (w, h) = crate::scenes::options::RESOLUTIONS[room.selected_resolution];
                    font.text(
                        c,
                        &format!("{w} x {h}"),
                        cx + 52.0,
                        y_text + font.init_baseline,
                        1,
                        col,
                    )?;
                }
                // Full screen.
                1 => toggle(c, assets, cx + 81.0, y_mid, room.settings.fullscreen, room.current_option == i)?,
                // Attack opacity slider, mapped from 0.3..=1.0 onto 70 px.
                2 => slider(
                    c,
                    assets,
                    cx,
                    y_button,
                    y_mid,
                    (room.settings.attack_alpha - 0.3) / 0.7,
                )?,
                // Damage numbers.
                3 => toggle(c, assets, cx + 81.0, y_mid, room.settings.show_damage_text, room.current_option == i)?,
                // Visual effects.
                4 => toggle(c, assets, cx + 81.0, y_mid, room.settings.light_fx, room.current_option == i)?,
                // Screen shake.
                5 => toggle(c, assets, cx + 81.0, y_mid, room.settings.screen_shake, room.current_option == i)?,
                // HUD portrait: a two-entry selector, not a boolean.
                6 => {
                    arrows(c, assets, cx + 52.0, y_mid, 38.0)?;
                    let idx = (room.settings.port_display > 0.5) as usize;
                    font.text(
                        c,
                        &t(&format!("options.port_display.{idx}")),
                        cx + 64.0,
                        y_text + font.init_baseline,
                        1,
                        col,
                    )?;
                }
                // HUD HP bar.
                7 => toggle(c, assets, cx + 81.0, y_mid, room.settings.show_hud_hp > 0.5, room.current_option == i)?,
                // HUD HP number.
                8 => toggle(c, assets, cx + 81.0, y_mid, room.settings.show_hp_val > 0.5, room.current_option == i)?,
                // Show mini HP.
                9 => toggle(c, assets, cx + 81.0, y_mid, room.settings.above_hp > 0.5, room.current_option == i)?,
                // Hide HP if full.
                10 => toggle(c, assets, cx + 81.0, y_mid, room.settings.hide_full_hp > 0.5, room.current_option == i)?,
                // Show skill radius.
                11 => toggle(c, assets, cx + 81.0, y_mid, room.settings.show_skill_radius > 0.5, room.current_option == i)?,
                _ => {}
            }
        }
    }
    Ok(())
}

/// Slider bar over the row button, with the knob at `value * 70` from the bar's
/// left edge, which is why attack opacity is rescaled before it arrives here.
fn slider(
    c: &mut Canvas<Window>,
    assets: &mut OptionsAssets,
    cx: f32,
    y_button: f32,
    y_mid: f32,
    value: f64,
) -> Result<(), String> {
    sprites::draw(c, assets.frame("hud_sliderBar", 0)?, cx + 10.0, y_button, 1., 1.)?;
    let v = value.clamp(0.0, 1.0) as f32;
    sprites::draw(c, assets.frame("hud_slider", 0)?, cx + 20.0 + v * 70.0, y_mid, 1., 1.)?;
    Ok(())
}

/// Left and right arrows around a centred value. `portDisplay` uses a different
/// left offset in the original, so the caller passes its own centre.
fn arrows(
    c: &mut Canvas<Window>,
    assets: &mut OptionsAssets,
    centre: f32,
    y_mid: f32,
    left: f32,
) -> Result<(), String> {
    sprites::draw(
        c,
        assets.frame("hud_scrollArrows2", 0)?,
        centre - left,
        y_mid,
        1.,
        1.,
    )?;
    sprites::draw(
        c,
        assets.frame("hud_scrollArrows2", 1)?,
        centre + left,
        y_mid,
        1.,
        1.,
    )?;
    Ok(())
}

/// `hud_toggleButton`: frame `value + 2 * selected`, so the four frames are
/// off-unselected, on-unselected, off-selected, on-selected.
fn toggle(
    c: &mut Canvas<Window>,
    assets: &mut OptionsAssets,
    x: f32,
    y_mid: f32,
    value: bool,
    selected: bool,
) -> Result<(), String> {
    let frame = usize::from(value) + 2 * usize::from(selected);
    sprites::draw(c, assets.frame("hud_toggleButton", frame)?, x, y_mid, 1., 1.)?;
    Ok(())
}

/// The keybind submenu. Row 6 is the "set all" row even though the label at that
/// index reads `Support`; the original draws only seven of the eight labels.
fn draw_keybinds(
    c: &mut Canvas<Window>,
    font: &mut Font,
    big: &mut Font,
    assets: &mut OptionsAssets,
    room: &Options,
    lang: &LanguagePack,
    cx: f32,
    cy: f32,
    white: Color,
    black: Color,
) -> Result<(), String> {
    let t = |key: &str| lang.text(key).to_string();
    outlined(big, c, &t("options.keybinds_title"), cx, cy + 10.0, 1, white)?;

    for i in 0..ROWS {
        let abs = i + room.show_option_range;
        let y_button = cy + 43.0 + (i as f32) * 34.0;
        let y_mid = cy + 56.0 + (i as f32) * 34.0;
        let y_text = cy + 51.0 + (i as f32) * 34.0;
        let col = if room.current_option == i { white } else { black };

        sprites::draw(
            c,
            assets.frame("hud_OptionButton", usize::from(room.current_option == i))?,
            cx + 12.0,
            y_button,
            1.,
            1.,
        )?;
        let icon_frame = abs * 2 + usize::from(room.current_option == i);
        sprites::draw(c, assets.frame("hud_keybindIcons", icon_frame)?, cx - 98.0, y_mid, 1., 1.)?;

        font.text(
            c,
            &t(&format!("options.keybind.{abs}")),
            cx - 66.0,
            y_text + font.init_baseline,
            0,
            col,
        )?;

        // While a key is being remapped the row is dimmed behind the key name.
        if room.remapping && room.current_option == i {
            c.set_blend_mode(BlendMode::Blend);
            c.set_draw_color(Color::RGBA(0, 0, 0, 128));
            c.fill_rect(Rect::new(
            (cx + 50.0) as i32,
            (cy + 48.0 + (i as f32) * 34.0) as i32,
            41,
            17,
        ))?;
            c.set_draw_color(Color::BLACK);
        }

        // Rows 0..5 show the bound key; row 6 is the "set all" row and shows none.
        if abs < 6 {
            let key = room.settings.the_buttons.get(abs).map(String::as_str).unwrap_or("");
            font.text(c, key, cx + 90.0, y_text + font.init_baseline, 2, col)?;
        }
    }
    Ok(())
}

/// The controller submenu. Row 6 has no arrows and no icon in the original, and
/// `OK!` appears once an assignment has been accepted.
fn draw_controller(
    c: &mut Canvas<Window>,
    font: &mut Font,
    big: &mut Font,
    assets: &mut OptionsAssets,
    room: &Options,
    lang: &LanguagePack,
    cx: f32,
    cy: f32,
    white: Color,
    black: Color,
) -> Result<(), String> {
    let t = |key: &str| lang.text(key).to_string();
    outlined(big, c, &t("options.controller_title"), cx, cy + 10.0, 1, white)?;
    sprites::draw(c, assets.frame("spr_brackets", 0)?, 228.0, 104.0, 1., 1.)?;

    for i in 0..ROWS {
        let abs = i + room.show_option_range;
        let y_button = cy + 43.0 + (i as f32) * 34.0;
        let y_mid = cy + 56.0 + (i as f32) * 34.0;
        let y_text = cy + 51.0 + (i as f32) * 34.0;
        let col = if room.current_option == i { white } else { black };

        sprites::draw(
            c,
            assets.frame("hud_OptionButton", usize::from(room.current_option == i))?,
            cx + 12.0,
            y_button,
            1.,
            1.,
        )?;
        font.text(
            c,
            &t(&format!("options.controller.{abs}")),
            cx - 66.0,
            y_text + font.init_baseline,
            0,
            col,
        )?;

        if abs != 6 {
            if room.current_option == i {
                c.set_blend_mode(BlendMode::Blend);
                c.set_draw_color(Color::RGBA(0, 0, 0, 128));
                c.fill_rect(Rect::new(
            (cx + 63.0) as i32,
            (cy + 46.0 + (i as f32) * 34.0) as i32,
            24,
            21,
        ))?;
                c.set_draw_color(Color::BLACK);
            }
            arrows(c, assets, cx + 75.0, y_mid, 18.0)?;
            // The icon sheet is indexed by the slot's position in the
            // controller button list.
            let position = room.current_positions.get(abs).copied().unwrap_or(0);
            sprites::draw(
                c,
                assets.frame("hud_controllerButtonIcons", position)?,
                cx + 75.0,
                y_mid,
                1.,
                1.,
            )?;
        }
    }

    if room.controller_set {
        outlined(font, c, &t("options.ok"), cx + 80.0, cy + 254.0, 1, white)?;
    }
    Ok(())
}

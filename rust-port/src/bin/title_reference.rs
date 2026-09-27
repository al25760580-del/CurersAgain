//! Reference application composition. Pure scene models and drawing helpers live in separate modules.
use holocure_core_port::render::{
    sprites::{draw, load_sprite, num, Sprite},
    text::Font,
};
use holocure_core_port::{
    menu::{Input, Screen, TitleMenu},
    title_dynamics::Bob,
    title_effects::EffectReplay,
};
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    rect::{FPoint, FRect, Rect},
    render::{BlendMode, Canvas, Vertex, VertexIndices},
    video::Window,
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
const LABEL_KEYS: [&str; 8] = [
    "title.leaderboards",
    "title.achievements",
    "title.shop",
    "title.play",
    "title.house",
    "title.settings",
    "title.credits",
    "title.quit",
];
fn create_scores_scene(
    root: &Path,
    profile: &Path,
    config: &holocure_core_port::scenes::scores::Config,
    settings: holocure_core_port::first_run::Settings,
    now: f64,
) -> Result<holocure_core_port::scenes::scores::HiScores, String> {
    let mut scene = holocure_core_port::scenes::scores::HiScores::new(
        config.clone(),
        profile.join("scores.json"),
        settings,
    );
    if let Some(config) = std::env::var_os("HOLOCURE_FIREBASE_CONFIG") {
        scene.server = Some(holocure_core_port::score_server::Server::new(
            root.join("firebase_readonly.py"),
            PathBuf::from(config),
        )?);
    }
    scene.FetchScores(now);
    println!(
        "SCORES_ENTER rm_HiScores (8); mode={}",
        if scene.server.is_some() {
            "OFFICIAL_READ_ONLY"
        } else {
            "LOCAL_OFFLINE"
        }
    );
    Ok(scene)
}
fn load_language<'a>(
    tc: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    root: &Path,
    id: &str,
) -> Result<
    (
        holocure_core_port::language::LanguagePack,
        Font<'a>,
        Vec<holocure_core_port::language::Manifest>,
        Value,
    ),
    String,
> {
    let pack = holocure_core_port::language::LanguagePack::load(root, id)?;
    let catalog = holocure_core_port::language::catalog(root)?;
    pack.require(&LABEL_KEYS)?;
    pack.require(&[
        "first_run.language",
        "first_run.hint",
        "first_run.name",
        "first_run.named_player",
        "first_run.default_name",
        "first_run.show_prompt",
        "first_run.warning",
        "first_run.consent",
        "first_run.summary_display",
        "first_run.summary_consent",
        "first_run.show",
        "first_run.hide",
        "first_run.allow",
        "first_run.decline",
        "first_run.start",
        "first_run.restart",
        "first_run.error",
        "title.controls",
        "title.version",
    ])?;
    let mut startup = serde_json::Map::new();
    for (scene, count) in [("pre_intro", 2), ("intro", 8)] {
        let mut lines = Vec::new();
        for i in 0..count {
            let key = format!("startup.{scene}.{i}");
            pack.require(&[&key])?;
            lines.push(Value::String(pack.text(&key).into()));
        }
        startup.insert(scene.into(), Value::Array(lines));
    }
    let font = Font::load_language(tc, &pack.font)?;
    Ok((pack, font, catalog, Value::Object(startup)))
}
struct Character<'a> {
    sprite: Sprite<'a>,
    x: f32,
    bob: Bob,
    previous_y: f64,
    start: f64,
    previous_start: f64,
    delay: i32,
}
/// Read the current framebuffer on the render thread, encode PNG on a worker.
/// Diagnostic only: GPU readback can disturb timing; never use capture-frame latency as baseline.
fn capture(c: &Canvas<Window>, path: &Path) -> Result<(), String> {
    let (w, h) = c.output_size()?;
    let pixels = c.read_pixels(None, PixelFormatEnum::RGBA32)?;
    let path = path.to_path_buf();
    std::thread::spawn(move || {
        match image::save_buffer(&path, &pixels, w, h, image::ColorType::Rgba8) {
            Ok(_) => println!("CAPTURE_SAVED {}", path.display()),
            Err(e) => eprintln!("CAPTURE_FAILED {e}"),
        }
    });
    Ok(())
}
/// Draw simulated title lines (depth 300) and triangles (depth 250), before characters.
/// Coordinates are logical 640×360; RenderDoc event 43 verifies centered 1-pixel line width.
/// Birth parameters still replay a finite capture; original RNG is not recovered.
fn draw_effects(
    c: &mut Canvas<Window>,
    triangle: &mut Sprite,
    effects: &EffectReplay,
    blend: f64,
) -> Result<(), String> {
    // Room-depth order verified from states: background 500, lines 300, triangles 250, cast 100.
    let mut vertices = Vec::new();
    for e in effects
        .active
        .iter()
        .rev()
        .filter(|e| !e.triangle && !e.gui)
    {
        let x = e.previous_x + (e.x - e.previous_x) * blend as f32;
        let y = e.previous_y + (e.y - e.previous_y) * blend as f32;
        let x2 = x + e.length;
        let h = e.width.max(1.0);
        let a = Color::RGBA(255, 255, 255, 204);
        let b = Color::RGBA(113, 201, 255, 204);
        let vertex = |x, y, color| Vertex {
            position: FPoint::new(x, y),
            color,
            tex_coord: FPoint::new(0.0, 0.0),
        };
        // RenderDoc event 43: world-space edges are y-0.5 and y+0.5 for width=1.
        let tl = vertex(x, y - h / 2.0, a);
        let tr = vertex(x2, y - h / 2.0, b);
        let bl = vertex(x, y + h / 2.0, a);
        let br = vertex(x2, y + h / 2.0, b);
        vertices.extend_from_slice(&[tl, tr, br, tl, br, bl]);
    }
    c.set_blend_mode(BlendMode::Blend);
    if !vertices.is_empty() {
        c.render_geometry(&vertices, None, VertexIndices::Sequential)?;
    }
    for e in effects.active.iter().rev().filter(|e| e.triangle) {
        let x = e.previous_x + (e.x - e.previous_x) * blend as f32;
        let y = e.previous_y + (e.y - e.previous_y) * blend as f32;
        let angle = e.previous_angle + (e.angle - e.previous_angle) * blend as f32;
        let alpha = e.previous_alpha + (e.alpha - e.previous_alpha) * blend as f32;
        triangle
            .tex
            .set_alpha_mod((alpha.clamp(0.0, 1.0) * 255.0).round() as u8);
        let sx = e.sx.max(0.00001);
        let sy = e.sy.max(0.00001);
        c.copy_ex_f(
            &triangle.tex,
            None,
            FRect::new(
                x - triangle.ox * sx,
                y - triangle.oy * sy,
                triangle.w as f32 * sx,
                triangle.h as f32 * sy,
            ),
            -(angle as f64),
            Some(FPoint::new(triangle.ox * sx, triangle.oy * sy)),
            false,
            false,
        )?;
    }
    triangle.tex.set_alpha_mod(255);
    Ok(())
}
/// Render only the visual startup rooms in logical 640×360 coordinates.
/// Original Intro.Draw uses sprite 1905/currentScene at (160,70), then black cover,
/// then font-18 text at (320,275), separation=20. PreIntro uses (320,100).
/// These captured English strings fit width=640; generic word wrapping is not implemented.
/// Audio and new-unlock overlays are explicitly outside this function's scope.
fn draw_startup(
    c: &mut Canvas<Window>,
    font: &mut Font,
    images: &[Sprite],
    state: &holocure_core_port::startup::Startup,
    texts: &Value,
) -> Result<(), String> {
    use holocure_core_port::startup::BootRoom;
    c.set_blend_mode(BlendMode::Blend);
    let paint_cover = |c: &mut Canvas<Window>| -> Result<(), String> {
        c.set_draw_color(Color::RGBA(
            0,
            0,
            0,
            (state.screen_alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
        ));
        c.fill_rect(Rect::new(0, 0, 640, 360))
    };
    let pre = state.room == BootRoom::PreIntro;
    if !pre && state.scene >= 0 {
        draw(c, &images[state.scene as usize], 160.0, 70.0, 1.0, 1.0)?;
        paint_cover(c)?;
    }
    if pre || state.scene >= 0 {
        let list = if pre {
            &texts["pre_intro"]
        } else {
            &texts["intro"]
        };
        font.tex.set_alpha_mod(
            (if pre { 1.0 } else { state.text_alpha }.clamp(0.0, 1.0) * 255.0).round() as u8,
        );
        if let Some(text) = list[state.text_index()].as_str() {
            for (line, text) in text.split('\n').enumerate() {
                font.text(
                    c,
                    text,
                    320.0,
                    if pre { 100.0 } else { 275.0 } + line as f32 * 20.0,
                    1,
                    Color::WHITE,
                )?;
            }
        }
        font.tex.set_alpha_mod(255);
        if pre {
            paint_cover(c)?;
        }
    }
    Ok(())
}
/// Reference application: fixed 60-Hz simulation, separate interpolation and presentation.
/// Ordinary startup with extracted audio; no unlock overlay or playable submenus.
/// F6 restarts visual boot; confirm during boot skips without confirming a title selection.
#[allow(non_snake_case)]
/// Apply what a confirmed option asks for. The original's `Confirmed` only
/// reports; the host applies, so the room itself stays testable without SDL.
///
/// Nothing here reaches the server: settings are written to the port's own
/// profile directory and the resolution only changes the port's render target.
fn apply_options_action(
    action: holocure_core_port::scenes::options::Action,
    slot: &mut Option<holocure_core_port::scenes::options::Options>,
    menu: &mut TitleMenu,
    profile_directory: &Path,
    manager: &mut holocure_core_port::first_run::GameManager,
    pending_resolution: &mut Option<usize>,
    pending_language: &mut Option<String>,
) {
    use holocure_core_port::scenes::options::Action;
    let Some(options) = slot.as_mut() else {
        return;
    };
    match action {
        Action::None => {}
        Action::Save => save_options(options, manager, profile_directory),
        Action::ApplyResolution(_, _) => {
            *pending_resolution = Some(options.selected_resolution);
            save_options(options, manager, profile_directory);
        }
        Action::SetLanguage(id) => {
            *pending_language = Some(id);
            save_options(options, manager, profile_directory);
        }
        Action::Exit => {
            menu.input(Input::Back);
            *slot = None;
        }
    }
}

/// Write the room's settings back to the profile and report the result. The
/// original calls `SaveSettings` at the same points.
fn save_options(
    options: &mut holocure_core_port::scenes::options::Options,
    manager: &mut holocure_core_port::first_run::GameManager,
    profile_directory: &Path,
) {
    manager.settings = options.settings.clone();
    match options.settings.SaveSettings(profile_directory) {
        Ok(()) => println!("OPTIONS_SAVED"),
        Err(e) => eprintln!("OPTIONS_SAVE_FAILED {e}"),
    }
}

fn main() -> Result<(), String> {
    let root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let bundled = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join("assets")));
            bundled
                .filter(|p| p.is_dir())
                .unwrap_or_else(|| PathBuf::from("../asset-lab"))
        });
    let manifest: Value = serde_json::from_slice(
        &fs::read(root.join("title-manifest.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let fontdata: Value = serde_json::from_slice(
        &fs::read(root.join("title-fonts.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let _custom_cursor = holocure_core_port::render::cursor::load(&root)?;
    _custom_cursor.set();
    sdl.mouse().show_cursor(true);
    println!("CURSOR_READY spr_GameCursor 48x50 hotspot=0,0");
    sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "0");
    let make_window = || {
        video
            .window("HoloCure / Rust · rm_Title (3)", 1280, 720)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())
    };
    let mut c = if std::env::var("HOLOCURE_SOFTWARE").is_ok() {
        make_window()?
            .into_canvas()
            .software()
            .build()
            .map_err(|e| e.to_string())?
    } else {
        match make_window()?
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Accelerated renderer unavailable ({e}); using paced software fallback");
                make_window()?
                    .into_canvas()
                    .software()
                    .build()
                    .map_err(|e| e.to_string())?
            }
        }
    };
    let renderer_name = c.info().name.to_string();
    let vsync = c.info().flags & 4 != 0;
    println!("RENDERER name={} vsync={}", renderer_name, vsync);
    let tc = c.texture_creator();
    let creation: Value = serde_json::from_slice(
        &fs::read(root.join("title-creation.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let mut chars = Vec::new();
    for x in manifest["characters"].as_array().unwrap() {
        let name = x["sprite"].as_str().unwrap();
        let fog = if x["unlocked"].as_bool().unwrap_or(false) {
            None
        } else {
            Some(num(x, "color") as u32)
        };
        let initial = creation["characters"]
            .as_object()
            .unwrap()
            .values()
            .find(|c| c["object"] == x["name"])
            .ok_or("Missing captured character creation parameters")?;
        chars.push(Character {
            sprite: load_sprite(&tc, &root, name, 0, &manifest["sprites"][name], fog)?,
            x: num(x, "x") as f32,
            bob: Bob {
                phase: num(initial, "lifetime") - std::f64::consts::PI / 60.0,
                y: num(initial, "ystart"),
                increment: std::f64::consts::PI / 60.0,
            },
            previous_y: num(initial, "ystart"),
            start: -300.0,
            previous_start: -300.0,
            delay: num(initial, "delay") as i32 + 1,
        });
    }
    let bg = load_sprite(
        &tc,
        &root,
        "hud_new_title_BG",
        0,
        &manifest["sprites"]["hud_new_title_BG"],
        None,
    )?;
    let bar = load_sprite(
        &tc,
        &root,
        "hud_titlebar",
        0,
        &manifest["sprites"]["hud_titlebar"],
        None,
    )?;
    let logo = load_sprite(
        &tc,
        &root,
        "spr_HoloCureTitle_full",
        0,
        &manifest["sprites"]["spr_HoloCureTitle_full"],
        None,
    )?;
    let mut icons = Vec::new();
    let mut selected = Vec::new();
    for i in 0..8 {
        icons.push(load_sprite(
            &tc,
            &root,
            "hud_title_icons",
            i,
            &manifest["sprites"]["hud_title_icons"],
            None,
        )?);
        selected.push(load_sprite(
            &tc,
            &root,
            "hud_title_icons_selected",
            i,
            &manifest["sprites"]["hud_title_icons_selected"],
            None,
        )?);
    }
    let mut small = Font::load(
        &tc,
        &root,
        &fontdata
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["id"] == 7)
            .unwrap(),
    )?;
    let mut init_room_assets =
        holocure_core_port::render::first_run::InitRoomAssets::load(&tc, &root)?;
    println!("FIRST_RUN_LOCAL: setup settings stay local; Scores networking requires explicit opt-in. Language packs are read from disk. F8 reloads text and glyphs.");
    let mut high_resolution = true;
    let mut room = tc
        .create_texture_target(PixelFormatEnum::RGBA8888, 1280, 720)
        .map_err(|e| e.to_string())?;
    let effect_data: Value = serde_json::from_slice(
        &fs::read(root.join("title-effects-from-start.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let mut effects = EffectReplay::new(&effect_data);
    let mut triangle = load_sprite(
        &tc,
        &root,
        "menu_charselec_triangle",
        0,
        &manifest["sprites"]["menu_charselec_triangle"],
        None,
    )?;
    let scores_config =
        holocure_core_port::scenes::scores::Config::load(&root.join("scores-config.toml"))?;
    let mut scores_assets =
        holocure_core_port::render::scores::ScoresAssets::load(&tc, &root, &scores_config)?;
    let mut hi_scores: Option<holocure_core_port::scenes::scores::HiScores> = None;
    let mut options_assets =
        holocure_core_port::render::options::OptionsAssets::load(&tc, &root)?;
    let mut options_room: Option<holocure_core_port::scenes::options::Options> = None;
    let mut options_pending_resolution: Option<usize> = None;
    let mut options_pending_language: Option<String> = None;
    let scores_clock = Instant::now();
    let mut events = sdl.event_pump()?;
    let mut menu = TitleMenu::ready_for_reference();
    menu.can_control = false;
    let mut startup = holocure_core_port::startup::Startup::default();
    let profile_directory = holocure_core_port::first_run::ProfileDirectory()?;
    let mut settings = holocure_core_port::first_run::Settings::LoadSettings(&profile_directory)?;
    if std::env::var("HOLOCURE_FORCE_SETUP").as_deref() == Ok("1") {
        settings.readyToStart = false;
    }

    let mut GameManager = holocure_core_port::first_run::GameManager::new(settings);
    // Diagnostic entry only for an already-completed, isolated port profile.
    if std::env::var("HOLOCURE_OPEN_SCORES").as_deref() == Ok("1") && !GameManager.active() {
        startup.room = holocure_core_port::startup::BootRoom::Title;
        menu.can_control = true;
        menu.input(Input::Hover(0));
        menu.input(Input::Confirm);
    }

    let mut text_input_active = false;
    println!(
        "PROFILE_DIRECTORY {} readyToStart={}",
        profile_directory.display(),
        GameManager.settings.readyToStart
    );

    let language_root = std::env::var_os("HOLOCURE_LANGUAGE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("languages"));
    let preferred = std::env::var("HOLOCURE_LANGUAGE")
        .unwrap_or_else(|_| GameManager.settings.CurrentLanguage.clone());
    let preferred = if init_room_assets.languages.iter().any(|l| l.id == preferred) {
        preferred
    } else {
        "eng".into()
    };
    let (pack, loaded_font, catalog, mut startup_texts) =
        load_language(&tc, &language_root, &preferred)?;
    let mut font = loaded_font;
    init_room_assets.language = pack;
    init_room_assets.languages = catalog;
    GameManager.settings.CurrentLanguage = preferred;
    GameManager.initLangOptions = init_room_assets
        .languages
        .iter()
        .map(|l| l.id.clone())
        .collect();
    GameManager.langOption = GameManager
        .initLangOptions
        .iter()
        .position(|id| *id == GameManager.settings.CurrentLanguage)
        .unwrap_or(0);
    GameManager.currentOption = GameManager.langOption;
    if !profile_directory.join("settings.json").exists() {
        GameManager.settings.username = init_room_assets
            .language
            .text("first_run.default_name")
            .into();
        GameManager.inputText = GameManager.settings.username.clone();
    }
    let mut scores_big = Font::load_language(&tc, &init_room_assets.language.big_font)?;
    let mut scores_tiny = Font::load_language(&tc, &init_room_assets.language.tiny_font)?;
    holocure_core_port::translation_limits::warn(
        &language_root.join("layout-limits.toml"),
        &init_room_assets.language,
        &font,
        &scores_big,
        &scores_tiny,
    );
    let mut intro_sprites = Vec::new();
    for i in 0..6 {
        intro_sprites.push(load_sprite(
            &tc,
            &root,
            "spr_IntroSequence",
            i,
            &manifest["sprites"]["spr_IntroSequence"],
            None,
        )?);
    }
    let sparkle_data: Value = serde_json::from_slice(
        &fs::read(root.join("title-sparkles.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let mut sparkle_images = Vec::new();
    for i in 0..3 {
        sparkle_images.push(load_sprite(
            &tc,
            &root,
            "spr_portraitspark",
            i,
            &manifest["sprites"]["spr_portraitspark"],
            None,
        )?);
    }
    let mut sparkles: Vec<holocure_core_port::title_sparkle::PortraitSpark> = Vec::new();
    let char_initial: Vec<_> = chars.iter().map(|c| (c.bob, c.delay)).collect();
    let mut logo_slide = -400.0f64;
    let mut previous_logo_slide = logo_slide;
    let mut logo_bob = Bob {
        phase: 0.0,
        y: 0.0,
        increment: std::f64::consts::PI / 120.0,
    };
    let mut accumulator = 0.0;
    let gain = std::env::var("HOLOCURE_VOLUME")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
        .filter(|x| x.is_finite())
        .unwrap_or(0.35);
    let mut audio = match holocure_core_port::audio::TitleAudio::open(&sdl, &root, gain) {
        Ok(a) => {
            println!("AUDIO_READY stereo=48000 gain={gain}");
            Some(a)
        }
        Err(e) => {
            eprintln!("AUDIO_DISABLED {e}");
            None
        }
    };
    let mut pointer = None;
    let mut now = Instant::now();
    let mut paused = false;
    let mut origins = false;
    let mut shot = std::env::var("HOLOCURE_CAPTURE_START").is_ok();
    let mut frames = 0u64;
    let mut logo_previous = logo_bob.y;
    let mut interpolate = true;
    let mut hud = true;
    let mut intervals: Vec<f64> = Vec::new();
    let mut metrics_start = Instant::now();
    let mut last_present = Instant::now();
    let mut summary = String::new();
    let mut dropped_time = 0.0;
    fs::create_dir_all("run-logs").map_err(|e| e.to_string())?;
    println!("TITLE_REFERENCE_READY: 47 characters, original sprites/fonts; room 640x360; GUI 1280x720. F1 scene HUD, F2 pause, F3 origins, F4 interpolation, F5 native/half-resolution target A/B, F6 replay PreIntro/Intro (does not reset your profile), F7 mute, F12 capture; mouse hover/left click on title. Enter/left click advance one startup room; Scores uses the explicitly selected local or official read-only provider; other title routes remain placeholders.");
    // Diagnostic exit bound for isolated packaging smoke tests; no input or save is injected.
    let mut smoke_frames = std::env::var("HOLOCURE_SMOKE_FRAMES")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .filter(|n| *n > 0 && *n <= 600);
    'running: loop {
        let frame_start = Instant::now();
        // Latch boot input for this event batch: repeated/queued Enter events must
        // not pass through the replacement room or confirm Play in the same batch.
        let init_was_active = GameManager.active();
        let was_scores = menu.screen == Screen::Scores;
        let was_options = menu.screen == Screen::Options;
        let mut init_confirm = false;
        let input_room = startup.room;
        let mut boot_enter_pressed = false;
        let mut pointer_moved = false;
        let mut left_pressed = false;
        let window_size = c.window().size();
        let pointer_view = holocure_core_port::render::viewport::Viewport::fit(
            (640, 360),
            window_size,
            holocure_core_port::render::viewport::ScalePolicy::Fit,
        )?;
        let mut reload_language = false;
        for e in events.poll_iter() {
            if matches!(
                &e,
                Event::KeyDown {
                    keycode: Some(Keycode::F8),
                    repeat: false,
                    ..
                }
            ) {
                reload_language = true;
                continue;
            }
            if menu.screen == Screen::Scores && !was_scores {
                continue;
            }
            if was_scores && menu.screen != Screen::Scores {
                continue;
            }
            if was_scores {
                use holocure_core_port::scenes::scores::{Action, Hit};
                if hi_scores.is_none() {
                    hi_scores = Some(create_scores_scene(
                        &root,
                        &profile_directory,
                        &scores_config,
                        GameManager.settings.clone(),
                        scores_clock.elapsed().as_secs_f64(),
                    )?);
                }
                let scene = hi_scores.as_mut().ok_or("Scores scene not initialized")?;
                let before = scene.settings.clone();
                let old_error = scene.error.clone();
                let old = (scene.currentOption, scene.currentSettingsOption);
                let now = scores_clock.elapsed().as_secs_f64();
                let mut action = Action::None;
                let mut confirmed = false;
                match &e {
                    Event::Quit { .. } => break 'running,
                    Event::KeyDown {
                        keycode: Some(key),
                        repeat: false,
                        ..
                    } => match *key {
                        Keycode::Escape | Keycode::LShift | Keycode::RShift => {
                            action = scene.ReturnMenu()
                        }
                        Keycode::Up => scene.SelectUpDown(-1),
                        Keycode::Down => scene.SelectUpDown(1),
                        Keycode::Left => scene.SelectLeftRight(-1),
                        Keycode::Right => scene.SelectLeftRight(1),
                        Keycode::PageUp => {
                            scene.page((scene.startingPosition / 10).saturating_sub(1))
                        }
                        Keycode::PageDown => scene.page(scene.startingPosition / 10 + 1),
                        Keycode::Return | Keycode::Space | Keycode::Z => {
                            action = scene.Confirmed(now);
                            confirmed = true;
                        }
                        Keycode::F12 => shot = true,
                        Keycode::F7 => {
                            if let Some(a) = audio.as_mut() {
                                a.toggle_mute();
                            }
                        }
                        _ => {}
                    },
                    Event::MouseMotion {
                        x, y, xrel, yrel, ..
                    } if *xrel != 0 || *yrel != 0 => {
                        if let Some(point) = pointer_view.to_logical((*x as f64, *y as f64)) {
                            match scene.hit(point) {
                                Some(Hit::Button(i)) | Some(Hit::Arrow(i, _)) => {
                                    if scene.showOptions {
                                        scene.currentSettingsOption = i;
                                    } else {
                                        scene.currentOption = i;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Event::MouseButtonDown {
                        mouse_btn: sdl2::mouse::MouseButton::Left,
                        x,
                        y,
                        ..
                    } => {
                        if scene.showScores {
                            scene.showScores = false;
                        } else if let Some(hit) = pointer_view
                            .to_logical((*x as f64, *y as f64))
                            .and_then(|p| scene.hit(p))
                        {
                            action = scene.click(hit, now);
                            confirmed = true;
                        }
                    }
                    Event::MouseWheel { y, .. } if !scene.showOptions && !scene.showScores => {
                        let page = (scene.startingPosition / 10) as i32;
                        scene.page((page - *y).max(0) as usize);
                    }
                    Event::TextInput { text, .. } if scene.changingName => {
                        scene.rename_input(text)
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Backspace),
                        repeat: true,
                        ..
                    } if scene.changingName => {
                        let trimmed: String = scene
                            .settings
                            .username
                            .chars()
                            .take(scene.settings.username.chars().count().saturating_sub(1))
                            .collect();
                        scene.rename_input(&trimmed);
                    }
                    _ => {}
                }
                if let Some(a) = audio.as_mut() {
                    if confirmed {
                        a.cue(holocure_core_port::audio::Cue::Confirm);
                    } else if old != (scene.currentOption, scene.currentSettingsOption) {
                        a.cue(holocure_core_port::audio::Cue::Select);
                    }
                }
                if action == Action::SaveSettings {
                    match scene.settings.SaveSettings(&profile_directory) {
                        Ok(()) => GameManager.settings = scene.settings.clone(),
                        Err(e) => {
                            scene.settings = before;
                            scene.error = Some(e);
                            scene.set_status("save_error");
                        }
                    }
                }
                if scene.error != old_error {
                    if let Some(error) = &scene.error {
                        eprintln!("SCORES_LOCAL_ERROR {error}");
                    }
                }
                if action == Action::ReturnMenu {
                    menu.input(Input::Back);
                    hi_scores = None;
                    if let Some(a) = audio.as_mut() {
                        a.cue(holocure_core_port::audio::Cue::Back);
                    }
                }
                continue;
            }
            if menu.screen == Screen::Options && !was_options {
                continue;
            }
            if was_options && menu.screen != Screen::Options {
                options_room = None;
                continue;
            }
            if was_options {
                // Each arm borrows the room only for its own statement, so the
                // helper can take the slot itself and clear it on exit.
                let mut with = |f: &mut dyn FnMut(&mut holocure_core_port::scenes::options::Options)| {
                    if let Some(room) = options_room.as_mut() {
                        f(room);
                    }
                };
                match &e {
                    Event::Quit { .. } => break 'running,
                    Event::KeyDown {
                        keycode: Some(key),
                        repeat: false,
                        ..
                    } => match *key {
                        Keycode::Escape | Keycode::LShift | Keycode::RShift => {
                            let mut leaving = false;
                            with(&mut |room| {
                                leaving = room.return_menu()
                                    == holocure_core_port::scenes::options::Action::Exit;
                            });
                            if leaving {
                                menu.input(Input::Back);
                                options_room = None;
                            }
                        }
                        Keycode::Up => with(&mut |room| room.select_up()),
                        Keycode::Down => with(&mut |room| room.select_down()),
                        Keycode::Left => with(&mut |room| room.select_left()),
                        Keycode::Right => with(&mut |room| room.select_right()),
                        Keycode::Return | Keycode::Space | Keycode::Z => {
                            let mut action = holocure_core_port::scenes::options::Action::None;
                            with(&mut |room| action = room.confirmed());
                            apply_options_action(
                                action,
                                &mut options_room,
                                &mut menu,
                                &profile_directory,
                                &mut GameManager,
                                &mut options_pending_resolution,
                                &mut options_pending_language,
                            );
                        }
                        Keycode::F12 => shot = true,
                        _ => {}
                    },
                    Event::MouseWheel { y, .. } => with(&mut |room| room.scroll(*y > 0)),
                    Event::MouseMotion { x, y, .. } => with(&mut |room| {
                        if let Some(i) = room.row_at(*x, *y) {
                            if room.current_option != i {
                                room.current_option = i;
                            }
                        }
                    }),
                    Event::MouseButtonDown { x, y, .. } => {
                        let mut action = holocure_core_port::scenes::options::Action::None;
                        let mut clicked = false;
                        with(&mut |room| match room.row_at(*x, *y) {
                            Some(i) if i != room.current_option => room.current_option = i,
                            Some(_) => {
                                action = room.confirmed();
                                clicked = true;
                            }
                            None => room.changed_settings |= room.slider_click(*x),
                        });
                        if clicked {
                            apply_options_action(
                                action,
                                &mut options_room,
                                &mut menu,
                                &profile_directory,
                                &mut GameManager,
                                &mut options_pending_resolution,
                                &mut options_pending_language,
                            );
                        }
                    }
                    _ => {}
                }
                continue;
            }
            if init_was_active {
                let before_option = GameManager.currentOption;
                match &e {
                    Event::Quit { .. } => break 'running,
                    Event::TextInput { text, .. } => GameManager.ReceiveInput(text),
                    Event::KeyDown {
                        keycode: Some(k),
                        repeat: false,
                        ..
                    } => match *k {
                        Keycode::Escape => break 'running,
                        Keycode::Up => GameManager.SelectUp(),
                        Keycode::Down => GameManager.SelectDown(),
                        Keycode::Return => init_confirm = true,
                        Keycode::Space if GameManager.initStep != 1 => init_confirm = true,
                        Keycode::Backspace => GameManager.Backspace(),
                        Keycode::F12 => shot = true,
                        Keycode::F7 => {
                            if let Some(a) = audio.as_mut() {
                                println!("AUDIO_MUTED {}", a.toggle_mute());
                            }
                        }
                        _ => {}
                    },
                    Event::KeyDown {
                        keycode: Some(Keycode::Backspace),
                        repeat: true,
                        ..
                    } => GameManager.Backspace(),
                    Event::MouseMotion {
                        x, y, xrel, yrel, ..
                    } => {
                        pointer = pointer_view.to_logical((*x as f64, *y as f64));
                        if *xrel != 0 || *yrel != 0 {
                            if let Some(point) = pointer {
                                if let Some(i) = GameManager.MouseOverButton(point) {
                                    GameManager.currentOption = i;
                                }
                            }
                        }
                    }
                    Event::MouseButtonDown {
                        mouse_btn: sdl2::mouse::MouseButton::Left,
                        x,
                        y,
                        ..
                    } => {
                        pointer = pointer_view.to_logical((*x as f64, *y as f64));
                        if let Some(point) = pointer {
                            // Native Draw_64 confirms the CURRENT option; hover handles selection.
                            if GameManager.MouseOverButton(point).is_some() {
                                init_confirm = true;
                            }
                        }
                    }
                    Event::Window {
                        win_event: sdl2::event::WindowEvent::Leave,
                        ..
                    } => pointer = None,
                    _ => {}
                }
                if before_option != GameManager.currentOption {
                    if let Some(a) = audio.as_mut() {
                        a.cue(holocure_core_port::audio::Cue::Select);
                    }
                }
                continue;
            }
            let selection_before = menu.current_option;
            match e {
                Event::Quit { .. } => break 'running,
                Event::MouseMotion {
                    x, y, xrel, yrel, ..
                } => {
                    pointer = pointer_view.to_logical((x as f64, y as f64));
                    pointer_moved |= xrel != 0 || yrel != 0;
                }
                Event::MouseButtonDown {
                    mouse_btn: sdl2::mouse::MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    pointer = pointer_view.to_logical((x as f64, y as f64));
                    left_pressed = true;
                }
                Event::MouseButtonDown {
                    mouse_btn: sdl2::mouse::MouseButton::Right,
                    ..
                } => {
                    if startup.room == holocure_core_port::startup::BootRoom::Title {
                        if menu.screen != Screen::Scores {
                            menu.input(Input::Back);
                        }
                    }
                }
                Event::Window {
                    win_event: sdl2::event::WindowEvent::Leave,
                    ..
                } => {
                    pointer = None;
                }
                Event::KeyDown {
                    keycode: Some(k),
                    repeat: false,
                    ..
                } => match k {
                    Keycode::Escape => break 'running,
                    Keycode::Left | Keycode::A => menu.input(Input::Left),
                    Keycode::Right | Keycode::D => menu.input(Input::Right),
                    Keycode::Up => menu.input(Input::Up),
                    Keycode::Down => menu.input(Input::Down),
                    Keycode::F6 => {
                        startup = Default::default();
                        sparkles.clear();
                        for (c, (bob, delay)) in chars.iter_mut().zip(&char_initial) {
                            c.bob = *bob;
                            c.previous_y = bob.y;
                            c.start = -300.0;
                            c.previous_start = -300.0;
                            c.delay = *delay;
                        }
                        logo_bob = Bob {
                            phase: 0.0,
                            y: 0.0,
                            increment: std::f64::consts::PI / 120.0,
                        };
                        logo_previous = 0.0;
                        logo_slide = -400.0;
                        previous_logo_slide = -400.0;
                        effects = EffectReplay::new(&effect_data);
                        frames = 0;
                        menu = TitleMenu::ready_for_reference();
                        menu.can_control = false;
                    }
                    Keycode::F1 => hud = !hud,
                    Keycode::F2 => {
                        paused = !paused;
                        if let Some(a) = audio.as_ref() {
                            a.pause(paused);
                        }
                    }
                    Keycode::F7 => {
                        if let Some(a) = audio.as_mut() {
                            println!("AUDIO_MUTED {}", a.toggle_mute());
                        }
                    }
                    Keycode::F4 => interpolate = !interpolate,
                    Keycode::F5 => {
                        high_resolution = !high_resolution;
                        let (w, h) = if high_resolution {
                            (1280, 720)
                        } else {
                            (640, 360)
                        };
                        room = tc
                            .create_texture_target(PixelFormatEnum::RGBA8888, w, h)
                            .map_err(|e| e.to_string())?;
                        println!("ROOM_TARGET {}x{}", w, h);
                    }
                    Keycode::F3 => origins = !origins,
                    Keycode::F12 => shot = true,
                    Keycode::Return
                        if startup.room != holocure_core_port::startup::BootRoom::Title =>
                    {
                        boot_enter_pressed = true;
                    }
                    Keycode::Z if startup.room != holocure_core_port::startup::BootRoom::Title => {}
                    Keycode::Return | Keycode::Z => {
                        if menu.can_control {
                            if let Some(a) = audio.as_mut() {
                                a.cue(holocure_core_port::audio::Cue::Confirm);
                            }
                        }
                        menu.input(Input::Confirm);
                        println!("VERIFIED_ROUTE {:?}; Scores is offline; other submenus remain placeholders",menu.screen);
                        if menu.screen == Screen::Exit {
                            break 'running;
                        }
                        if menu.screen != Screen::Scores {
                            menu.input(Input::Back);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
            if menu.current_option != selection_before
                && startup.room == holocure_core_port::startup::BootRoom::Title
            {
                if let Some(a) = audio.as_mut() {
                    a.cue(holocure_core_port::audio::Cue::Select);
                }
            }
        }
        if init_was_active && init_confirm {
            let before = GameManager.initStep;
            let finished = GameManager.EnterKey();
            if finished {
                if GameManager.Complete(&profile_directory) {
                    println!("SETTINGS_SAVED readyToStart=true; rm_InitRoom -> rm_Pre_Intro");
                } else {
                    eprintln!("{}", GameManager.error);
                }
            }
            if before != GameManager.initStep || !GameManager.active() {
                if let Some(a) = audio.as_mut() {
                    a.cue(holocure_core_port::audio::Cue::Confirm);
                }
            }
            println!(
                "INIT_STATE initStep={} currentOption={} nameOkay={} canType={}",
                GameManager.initStep,
                GameManager.currentOption,
                GameManager.nameOkay,
                GameManager.canType
            );
        }
        if let Some(id) = options_pending_language.take() {
            GameManager.settings.CurrentLanguage = id.clone();
            println!("OPTIONS_LANGUAGE {id}");
        }
        if reload_language
            || GameManager.settings.CurrentLanguage != init_room_assets.language.manifest.id
        {
            match load_language(&tc, &language_root, &GameManager.settings.CurrentLanguage) {
                Ok((pack, next_font, catalog, texts)) => {
                    let next_big = Font::load_language(&tc, &pack.big_font);
                    let next_tiny = Font::load_language(&tc, &pack.tiny_font);
                    let (Ok(next_big), Ok(next_tiny)) = (next_big, next_tiny) else {
                        eprintln!("LANGUAGE_RELOAD_FAILED: invalid scores font role");
                        GameManager.settings.CurrentLanguage =
                            init_room_assets.language.manifest.id.clone();
                        continue;
                    };
                    holocure_core_port::translation_limits::warn(
                        &language_root.join("layout-limits.toml"),
                        &pack,
                        &next_font,
                        &next_big,
                        &next_tiny,
                    );
                    scores_big = next_big;
                    scores_tiny = next_tiny;
                    font = next_font;
                    startup_texts = texts;
                    init_room_assets.language = pack;
                    init_room_assets.languages = catalog;
                    GameManager.initLangOptions = init_room_assets
                        .languages
                        .iter()
                        .map(|l| l.id.clone())
                        .collect();
                    GameManager.langOption = GameManager
                        .initLangOptions
                        .iter()
                        .position(|id| *id == GameManager.settings.CurrentLanguage)
                        .unwrap_or(0);
                    if GameManager.initStep == 0 {
                        GameManager.currentOption = GameManager
                            .currentOption
                            .min(GameManager.initLangOptions.len() - 1);
                    }
                    println!(
                        "LANGUAGE_LOADED {} custom_font={}",
                        init_room_assets.language.manifest.id,
                        init_room_assets.language.font.custom
                    );
                }
                Err(e) => {
                    eprintln!("LANGUAGE_RELOAD_FAILED; keeping last working pack: {e}");
                    GameManager.settings.CurrentLanguage =
                        init_room_assets.language.manifest.id.clone();
                }
            }
        }
        let needs_text = (GameManager.active() && GameManager.initStep == 1 && GameManager.canType)
            || hi_scores
                .as_ref()
                .is_some_and(|scene| scene.changingName);
        if needs_text != text_input_active {
            if needs_text {
                video.text_input().start();
            } else {
                video.text_input().stop();
            }
            text_input_active = needs_text;
        }
        if input_room != holocure_core_port::startup::BootRoom::Title
            && startup.advance_from_input(boot_enter_pressed, left_pressed)
        {
            println!(
                "BOOT_ADVANCE enter={} left_press={} {:?} -> {:?}",
                boot_enter_pressed, left_pressed, input_room, startup.room
            );
        }
        // Native global left press precedes Draw-time hover. Do not select a clicked index implicitly.
        if input_room == holocure_core_port::startup::BootRoom::Title
            && startup.room == holocure_core_port::startup::BootRoom::Title
            && left_pressed
        {
            if let Some(point) = pointer {
                if holocure_core_port::scenes::title::pointer::left_press(&mut menu, point) {
                    if let Some(a) = audio.as_mut() {
                        a.cue(holocure_core_port::audio::Cue::Confirm);
                    }
                    println!(
                        "MOUSE_ROUTE {:?}; Scores is offline; other submenus remain placeholders",
                        menu.screen
                    );
                    if menu.screen == Screen::Exit {
                        break 'running;
                    }
                    if menu.screen != Screen::Scores && menu.screen != Screen::Options {
                        menu.input(Input::Back);
                    }
                }
            }
        }
        if let Some(index) = options_pending_resolution.take() {
            let (w, h) = holocure_core_port::scenes::options::RESOLUTIONS[index];
            high_resolution = (w, h) == (1280, 720);
            room = tc
                .create_texture_target(PixelFormatEnum::RGBA8888, w, h)
                .map_err(|e| e.to_string())?;
            println!("OPTIONS_RESOLUTION {}x{}", w, h);
        }
        if let Some(room) = options_room.as_mut() {
            room.update();
        }
        if menu.screen == Screen::Options && options_room.is_none() {
            // The original's settings room offers exactly two languages even
            // though it ships four packs, so the row and the wrap are the same.
            let mut languages: Vec<String> = Vec::new();
            for id in ["eng", "jp"] {
                if init_room_assets.languages.iter().any(|l| l.id == id) {
                    languages.push(id.to_string());
                }
            }
            options_room = Some(holocure_core_port::scenes::options::Options::new(
                GameManager.settings.clone(),
                languages,
            ));
        }
        if menu.screen == Screen::Scores && hi_scores.is_none() {
            hi_scores = Some(create_scores_scene(
                &root,
                &profile_directory,
                &scores_config,
                GameManager.settings.clone(),
                scores_clock.elapsed().as_secs_f64(),
            )?);
        }
        let raw_dt = now.elapsed().as_secs_f64();
        let dt = raw_dt.min(0.25);
        dropped_time += (raw_dt - dt).max(0.0);
        now = Instant::now();
        if !paused {
            accumulator += dt;
        }
        // Fixed-step source-derived startup and entrance; rendering interpolation is separate.
        while accumulator >= 1.0 / 60.0 {
            if menu.screen == Screen::Scores {
                if let Some(scene) = hi_scores.as_mut() {
                    scene.Step();
                }
                accumulator -= 1.0 / 60.0;
                continue;
            }
            if GameManager.active() {
                GameManager.step();
                accumulator -= 1.0 / 60.0;
                continue;
            }
            if startup.room != holocure_core_port::startup::BootRoom::Title {
                startup.step();
            }
            if startup.room == holocure_core_port::startup::BootRoom::Title {
                for ch in &mut chars {
                    ch.previous_y = ch.bob.y;
                    holocure_core_port::title_dynamics::step_character(&mut ch.bob);
                    ch.previous_start = ch.start;
                    holocure_core_port::title_dynamics::step_entrance(&mut ch.start, &mut ch.delay);
                }
                logo_previous = logo_bob.y;
                logo_bob.step();
                previous_logo_slide = logo_slide;
                if logo_slide.abs() > 1e-5 {
                    logo_slide *= 0.8;
                }
                effects.step();
                frames += 1;
                sparkles.retain_mut(|s| s.step());
                let tick = (frames - 1) % sparkle_data["period_steps"].as_u64().unwrap() + 1;
                for b in sparkle_data["births"].as_array().unwrap() {
                    if b["at"].as_u64() == Some(tick) {
                        sparkles.push(holocure_core_port::title_sparkle::PortraitSpark::new(
                            num(b, "x") as f32,
                            num(b, "y") as f32,
                        ));
                    }
                }

                menu.can_control = frames >= 5;
            }
            accumulator -= 1.0 / 60.0;
        }
        if let Some(a) = audio.as_mut() {
            a.room(startup.room);
        }
        if startup.room == holocure_core_port::startup::BootRoom::Title {
            if let Some(point) = pointer {
                if holocure_core_port::scenes::title::pointer::hover(
                    &mut menu,
                    point,
                    pointer_moved,
                ) {
                    println!("MOUSE_SELECTION {}", menu.current_option);
                    if let Some(a) = audio.as_mut() {
                        a.cue(holocure_core_port::audio::Cue::Select);
                    }
                }
            }
        }
        let blend = if interpolate && !paused {
            accumulator * 60.0
        } else {
            1.0
        };
        let mut error: Option<String> = None;
        c.with_texture_canvas(&mut room, |c| {
            let result = (|| -> Result<(), String> {
                let scale = if high_resolution { 2.0 } else { 1.0 };
                c.set_scale(scale, scale)?;
                c.set_draw_color(Color::BLACK);
                c.clear();
                if let Some(room) = options_room.as_ref() {
                    return holocure_core_port::render::options::draw(
                        c,
                        &mut font,
                        &mut scores_big,
                        &mut options_assets,
                        room,
                        &init_room_assets.language,
                    );
                }
                if let Some(scene) = hi_scores.as_ref() {
                    return holocure_core_port::render::scores::draw(
                        c,
                        &mut font,
                        &mut scores_assets,
                        &mut scores_big,
                        &mut scores_tiny,
                        scene,
                        &init_room_assets.language,
                    );
                }
                if GameManager.active() {
                    return holocure_core_port::render::first_run::draw(
                        c,
                        &mut font,
                        &GameManager,
                        &init_room_assets,
                    );
                }
                if startup.room != holocure_core_port::startup::BootRoom::Title {
                    return draw_startup(c, &mut font, &intro_sprites, &startup, &startup_texts);
                }
                draw(c, &bg, 0.0, 0.0, 1.0, 1.0)?;
                draw_effects(c, &mut triangle, &effects, blend)?;
                for ch in &chars {
                    draw(
                        c,
                        &ch.sprite,
                        ch.x,
                        (ch.previous_y + (ch.bob.y - ch.previous_y) * blend) as f32
                            + (ch.previous_start + (ch.start - ch.previous_start) * blend) as f32,
                        1.0,
                        1.0,
                    )?;
                }
                // Stationary portrait sparks: native depth 80, after cast depth 100, before menu.
                for star in &sparkles {
                    draw(c, &sparkle_images[star.frame()], star.x, star.y, 1.0, 1.0)?;
                }
                draw(c, &bar, 0.0, 273.0, 1.0, 0.55)?;
                for i in 0..8 {
                    draw(
                        c,
                        if i == menu.current_option {
                            &selected[i]
                        } else {
                            &icons[i]
                        },
                        170.0 + i as f32 * 50.0,
                        305.0,
                        1.0,
                        1.0,
                    )?;
                }
                let x = 170.0 + menu.current_option as f32 * 50.0;
                font.text(
                    c,
                    init_room_assets
                        .language
                        .text(LABEL_KEYS[menu.current_option]),
                    x,
                    328.0,
                    1,
                    Color::BLACK,
                )?;
                font.text(
                    c,
                    init_room_assets
                        .language
                        .text(LABEL_KEYS[menu.current_option]),
                    x,
                    327.0,
                    1,
                    Color::WHITE,
                )?;
                font.text(
                    c,
                    &init_room_assets
                        .language
                        .text("title.version")
                        .replace("{version}", manifest["title"]["version"].as_str().unwrap()),
                    630.0,
                    8.0,
                    2,
                    Color::WHITE,
                )?;
                // Native commandPromps: font 18, right aligned at (630,345), radius 1, 16 passes.
                // Single-line scope only: original separation=10 and wrap width=500 do not wrap this text.
                for (dx, dy) in holocure_core_port::title_outline::outline_offsets(1.0, 16) {
                    font.text(
                        c,
                        init_room_assets.language.text("title.controls"),
                        630.0 + dx,
                        345.0 + dy,
                        2,
                        Color::BLACK,
                    )?;
                }
                font.text(
                    c,
                    init_room_assets.language.text("title.controls"),
                    630.0,
                    345.0,
                    2,
                    Color::WHITE,
                )?;
                if hud {
                    small.text(c, &summary, 5.0, 5.0, 0, Color::RGB(110, 240, 230))?;
                }
                if origins {
                    c.set_draw_color(Color::RGB(255, 75, 90));
                    for ch in &chars {
                        let x = ch.x as i32;
                        let y = ch.bob.y as i32;
                        c.draw_line((x - 3, y), (x + 3, y))?;
                        c.draw_line((x, y - 3), (x, y + 3))?;
                    }
                }
                Ok(())
            })();
            if let Err(e) = result {
                error = Some(e);
            }
        })
        .map_err(|e| e.to_string())?;
        if let Some(e) = error {
            return Err(e);
        }
        c.set_scale(1.0, 1.0)?;
        c.set_draw_color(Color::BLACK);
        c.clear();
        c.copy(&room, None, None)?;
        if menu.screen == Screen::Title
            && startup.room == holocure_core_port::startup::BootRoom::Title
        {
            draw(
                &mut c,
                &logo,
                640.0,
                20.0 + (logo_previous + (logo_bob.y - logo_previous) * blend) as f32
                    + (previous_logo_slide + (logo_slide - previous_logo_slide) * blend) as f32,
                0.72,
                0.72,
            )?;
        }
        if shot {
            capture(&c, Path::new("run-logs/title-reference.png"))?;
            shot = false;
            println!("CAPTURE run-logs/title-reference.png frame_offset={frames}");
        }
        if smoke_frames == Some(1) {
            let (w, h) = c.output_size()?;
            image::save_buffer(
                "run-logs/packaging-smoke.png",
                &c.read_pixels(None, PixelFormatEnum::RGBA32)?,
                w,
                h,
                image::ColorType::Rgba8,
            )
            .map_err(|e| e.to_string())?;
            println!("PACKAGING_SMOKE_OK: framebuffer captured; no setup confirmation injected");
            break 'running;
        }
        if let Some(n) = smoke_frames.as_mut() {
            *n -= 1;
        }
        c.present();
        if !vsync {
            let target = Duration::from_secs_f64(1.0 / 60.0);
            let used = frame_start.elapsed();
            if used < target {
                std::thread::sleep(target - used);
            }
        }
        let elapsed_ms = last_present.elapsed().as_secs_f64() * 1000.0;
        last_present = Instant::now();
        intervals.push(elapsed_ms);
        if metrics_start.elapsed().as_secs_f64() >= 1.0 {
            let elapsed = metrics_start.elapsed().as_secs_f64();
            let fps = intervals.len() as f64 / elapsed;
            let mut sorted = intervals.clone();
            sorted.sort_by(f64::total_cmp);
            let percentile = |q: f64| sorted[((sorted.len() - 1) as f64 * q).round() as usize];
            let status = serde_json::json!({"schema":1,"scene":if menu.screen==Screen::Options{"rm_Title (3) / obj_Options"}else if menu.screen==Screen::Scores{"rm_HiScores (8)"}else if GameManager.active(){"rm_InitRoom (0)"}else{startup.scene_name()},"initStep":GameManager.initStep,"readyToStart":GameManager.settings.readyToStart,"startup_lifetime":startup.lifetime,"current_option":if let Some(room)=options_room.as_ref(){room.absolute_option()}else if let Some(scene)=hi_scores.as_ref(){scene.currentOption}else if GameManager.active(){GameManager.currentOption}else{menu.current_option},"audio_enabled":audio.is_some(),"renderer":renderer_name,"vsync":vsync,"fps":fps,"p95_ms":percentile(0.95),"p99_ms":percentile(0.99),"max_ms":sorted.last(),"simulation_step":frames,"interpolation":interpolate,"paused":paused,"room_target":if high_resolution{"1280x720"}else{"640x360"},"effects":effects.active.len(),"effect_spawn_mode":"captured schedule replay, not original RNG","discarded_wall_time_s":dropped_time});
            fs::write("run-logs/rust-scene-status.json.tmp", status.to_string())
                .map_err(|e| e.to_string())?;
            fs::rename(
                "run-logs/rust-scene-status.json.tmp",
                "run-logs/rust-scene-status.json",
            )
            .map_err(|e| e.to_string())?;
            use std::io::Write;
            let mut history = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("run-logs/rust-frame-metrics.jsonl")
                .map_err(|e| e.to_string())?;
            writeln!(history, "{}", status).map_err(|e| e.to_string())?;
            summary = if hi_scores.is_some() {
                format!("rm_HiScores (8) / offline / {:.0} FPS", fps)
            } else if GameManager.active() {
                format!(
                    "rm_InitRoom (0) / initStep {} / {:.0} FPS",
                    GameManager.initStep, fps
                )
            } else {
                format!(
                    "{} / {} / {:.0} FPS",
                    startup.scene_name(),
                    init_room_assets
                        .language
                        .text(LABEL_KEYS[menu.current_option]),
                    fps
                )
            };
            c.window_mut()
                .set_title(&format!(
                    "HoloCure / Rust · {} · {}",
                    summary, renderer_name
                ))
                .map_err(|e| e.to_string())?;
            println!("FRAME_METRICS {}", status);
            intervals.clear();
            metrics_start = Instant::now();
        }
    }
    Ok(())
}

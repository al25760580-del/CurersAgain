//! Opt-in rendered smoke test. Hidden window, no desktop input, no player-profile writes.
#[test]
#[ignore = "requires SDL display, HOLOCURE_ASSET_ROOT and HOLOCURE_TEST_OUTPUT"]
fn render_five_init_screens() {
    use holocure_core_port::{
        first_run::{GameManager, Settings},
        render::{first_run, text::Font},
    };
    use sdl2::pixels::{Color, PixelFormatEnum};
    let root = std::path::PathBuf::from(std::env::var_os("HOLOCURE_ASSET_ROOT").unwrap());
    let output = std::path::PathBuf::from(std::env::var_os("HOLOCURE_TEST_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let mut c = video
        .window("First-run rendering test", 1280, 720)
        .hidden()
        .build()
        .unwrap()
        .into_canvas()
        .software()
        .build()
        .unwrap();
    let tc = c.texture_creator();
    let assets = first_run::InitRoomAssets::load(&tc, &root).unwrap();
    let mut font = Font::load_language(&tc, &assets.language.font).unwrap();
    let mut g = GameManager::new(Settings::default());
    g.settings.username = "MilaTestPlayerName".into();
    g.inputText = g.settings.username.clone();
    g.settings.hiscoreName = true;
    g.rectVis = true;
    c.set_scale(2., 2.).unwrap();
    for step in 0..5 {
        g.initStep = step;
        // Match the independently identified arrow frame of each archived screenshot.
        g.image_index = if step == 2 || step == 3 { 1. } else { 0. };
        c.set_draw_color(Color::BLACK);
        c.clear();
        first_run::draw(&mut c, &mut font, &g, &assets).unwrap();
        let pixels = c.read_pixels(None, PixelFormatEnum::RGBA32).unwrap();
        assert!(
            pixels
                .chunks_exact(4)
                .filter(|p| p[0] > 100 && p[1] > 100 && p[2] > 100)
                .count()
                > 1000
        );
        image::save_buffer(
            output.join(format!("initStep-{step}.png")),
            &pixels,
            1280,
            720,
            image::ColorType::Rgba8,
        )
        .unwrap();
    }
}

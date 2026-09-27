#[test]
#[ignore = "requires SDL display, HOLOCURE_ASSET_ROOT and HOLOCURE_TEST_OUTPUT"]
fn scores_screens_and_title_route() {
    use holocure_core_port::{
        first_run::Settings,
        language::LanguagePack,
        menu::{Input, Screen, TitleMenu},
        render::{scores::ScoresAssets, text::Font},
        scenes::scores::{Action, Config, HiScores},
    };
    use sdl2::pixels::{Color, PixelFormatEnum};
    use std::path::PathBuf;
    let root = PathBuf::from(std::env::var_os("HOLOCURE_ASSET_ROOT").unwrap());
    let out = PathBuf::from(std::env::var_os("HOLOCURE_TEST_OUTPUT").unwrap());
    std::fs::create_dir_all(&out).unwrap();
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let mut c = video
        .window("Scores smoke test", 1280, 720)
        .hidden()
        .build()
        .unwrap()
        .into_canvas()
        .software()
        .build()
        .unwrap();
    let tc = c.texture_creator();
    c.set_scale(2., 2.).unwrap();
    let config = Config::load(&root.join("scores-config.toml")).unwrap();
    let mut assets = ScoresAssets::load(&tc, &root, &config).unwrap();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/scores.demo.json");
    let mut scores = HiScores::new(config, path, Settings::default());
    let mut menu = TitleMenu::ready_for_reference();
    menu.input(Input::Hover(0));
    menu.input(Input::Confirm);
    assert_eq!(menu.screen, Screen::Scores);
    for lang in ["eng", "es"] {
        let pack = LanguagePack::load(&root.join("languages"), lang).unwrap();
        let mut font = Font::load_language(&tc, &pack.font).unwrap();
        let mut big = Font::load_language(&tc, &pack.big_font).unwrap();
        let mut tiny = Font::load_language(&tc, &pack.tiny_font).unwrap();
        for state in [
            "empty",
            "page1",
            "page3",
            "details",
            "rename",
            "rename-confirm",
            "rename-cancel",
            "delete",
            "delete-yes",
            "version-selected",
            "settings",
        ] {
            scores.showOptions = false;
            scores.changingName = false;
            scores.deleteConfirm = false;
            scores.showScores = false;
            match state {
                "empty" => {
                    scores.showingArray.clear();
                }
                "page1" => scores.FetchScores(if lang == "eng" { 0. } else { 10. }),
                "page3" => scores.page(2),
                "details" => {
                    scores.currentOption = 4;
                    scores.Confirmed(20.);
                }
                "rename-confirm" | "rename-cancel" => {
                    scores.showOptions = true;
                    scores.changingName = true;
                    scores.currentSettingsOption = if state == "rename-confirm" { 6 } else { 7 };
                }
                "delete-yes" => {
                    scores.showOptions = true;
                    scores.deleteConfirm = true;
                    scores.currentSettingsOption = 6;
                }
                "version-selected" => {
                    scores.showOptions = true;
                    scores.currentSettingsOption = 0;
                }
                "settings" => {
                    scores.currentOption = 6;
                    scores.Confirmed(20.);
                }
                "rename" | "delete" => {
                    scores.showOptions = true;
                    scores.currentSettingsOption = if state == "rename" { 1 } else { 4 };
                    scores.Confirmed(20.);
                }
                _ => {}
            }
            for _ in 0..if state == "settings" { 180 } else { 8 } {
                scores.Step();
            }
            c.set_draw_color(Color::BLACK);
            c.clear();
            holocure_core_port::render::scores::draw(
                &mut c,
                &mut font,
                &mut assets,
                &mut big,
                &mut tiny,
                &scores,
                &pack,
            )
            .unwrap();
            image::save_buffer(
                out.join(format!("{lang}-{state}.png")),
                &c.read_pixels(None, PixelFormatEnum::RGBA32).unwrap(),
                1280,
                720,
                image::ColorType::Rgba8,
            )
            .unwrap();
        }
    }
    assert_eq!(scores.ReturnMenu(), Action::None);
    assert_eq!(scores.ReturnMenu(), Action::ReturnMenu);
    menu.input(Input::Back);
    assert_eq!(menu.screen, Screen::Title);
    assert_eq!(menu.current_option, 0);
}

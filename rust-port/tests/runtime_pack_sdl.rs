//! Runtime editing test: no player profiles, no native game or injected input.
#[test]
#[ignore = "requires SDL desktop and HOLOCURE_TEST_OUTPUT"]
fn custom_glyph_fallback_and_live_disk_edits() {
    use holocure_core_port::{language::LanguagePack, render::text::Font};
    use std::{fs, path::Path};
    fn copy(a: &Path, b: &Path) {
        fs::create_dir_all(b).unwrap();
        for f in fs::read_dir(a).unwrap() {
            let f = f.unwrap().path();
            if f.is_file() {
                fs::copy(&f, b.join(f.file_name().unwrap())).unwrap();
            }
        }
    }
    let project = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = std::env::temp_dir().join(format!("runtime-pack-test-{}", std::process::id()));
    assert!(!root.exists());
    fs::create_dir(&root).unwrap();
    copy(&project.join("languages/eng"), &root.join("eng"));
    copy(&project.join("language-examples/es"), &root.join("es"));
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let mut c = video
        .window("Runtime pack test", 640, 100)
        .hidden()
        .build()
        .unwrap()
        .into_canvas()
        .software()
        .build()
        .unwrap();
    let tc = c.texture_creator();
    let base = LanguagePack::load(&root, "eng").unwrap();
    let original = Font::load_language(&tc, &base.font).unwrap();
    let pack = LanguagePack::load(&root, "es").unwrap();
    let mut font = Font::load_language(&tc, &pack.font).unwrap();
    assert_eq!(font.width("Player"), original.width("Player"));
    assert!(font.width("ñ") > 0);
    assert_eq!(pack.text("title.house"), base.text("title.house"));
    c.set_draw_color(sdl2::pixels::Color::BLACK);
    c.clear();
    font.text(
        &mut c,
        "Settings · áéíóú ÁÉÍÓÚ ñÑ",
        12.,
        12.,
        0,
        sdl2::pixels::Color::WHITE,
    )
    .unwrap();
    let output = std::path::PathBuf::from(std::env::var_os("HOLOCURE_TEST_OUTPUT").unwrap());
    fs::create_dir_all(&output).unwrap();
    image::save_buffer(
        output.join("custom-font.png"),
        &c.read_pixels(None, sdl2::pixels::PixelFormatEnum::RGBA32)
            .unwrap(),
        640,
        100,
        image::ColorType::Rgba8,
    )
    .unwrap();
    let path = root.join("es/texts.toml");
    fs::write(
        &path,
        fs::read_to_string(&path)
            .unwrap()
            .lines()
            .map(|line| {
                if line.starts_with("\"first_run.start\"") {
                    "\"first_run.start\" = \"BEGIN EDITED\"".to_string()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
    let edited = LanguagePack::load(&root, "es").unwrap();
    assert_eq!(edited.text("first_run.start"), "BEGIN EDITED");
    assert_ne!(pack.text("first_run.start"), edited.text("first_run.start"));
    let path = root.join("es/glyphs.toml");
    let mut data: toml::Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    data["glyphs"]["ñ"][4] = toml::Value::Integer(12);
    fs::write(&path, toml::to_string(&data).unwrap()).unwrap();
    let updated = Font::load_language(&tc, &edited.font).unwrap();
    assert_eq!(updated.width("ñ"), 12);
    assert_ne!(font.width("ñ"), 12);
    assert!(updated.visual_width("ñ") >= 12);
    assert!(font.visual_width("ñ") > 0);
    let limits = holocure_core_port::translation_limits::Limits::load(
        &project.join("languages/layout-limits.toml"),
    )
    .unwrap();
    let mut strings = edited.strings.clone();
    let long = "W".repeat(200);
    strings.insert("scores.check".into(), long.clone());
    let warnings = limits.check(&strings, |_, s| updated.visual_width(s));
    assert!(warnings
        .iter()
        .any(|w| w.key == "scores.check" && w.width > w.max_width && w.chars > w.max_chars));
    assert_eq!(strings["scores.check"], long);

    data["atlas_width"] = toml::Value::Integer(999);
    fs::write(&path, toml::to_string(&data).unwrap()).unwrap();
    assert!(Font::load_language(&tc, &edited.font).is_err());
    assert_eq!(updated.width("ñ"), 12);
    let path = root.join("es/pack.toml");
    fs::write(
        &path,
        fs::read_to_string(&path)
            .unwrap()
            .replace("fallback=\"eng\"", "fallback=\"es\""),
    )
    .unwrap();
    assert!(LanguagePack::load(&root, "es").is_err());
    fs::remove_dir_all(&root).unwrap();
}

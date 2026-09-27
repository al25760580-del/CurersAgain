//! Opt-in native SDL smoke test: hidden window, no input sent to Steam or desktop.
#[test]
#[ignore = "requires desktop SDL video and HOLOCURE_ASSET_ROOT"]
fn creates_original_color_cursor() {
    let root = std::env::var_os("HOLOCURE_ASSET_ROOT").expect("set asset root");
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let _window = video
        .window("Rust cursor isolated smoke test", 64, 64)
        .hidden()
        .build()
        .unwrap();
    let cursor = holocure_core_port::render::cursor::load(std::path::Path::new(&root)).unwrap();
    cursor.set();
}

//! External integration tests: actual Steam title observations and future viewport contracts.
//! These exercise production modules; no duplicate implementation of menu rules.
use holocure_core_port::{
    menu::{Input, Screen, TitleMenu},
    render::viewport::{ScalePolicy, Viewport},
};
#[test]
fn title_matches_sixteen_live_steam_actions() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/bridge-title-cycle.json")).unwrap();
    let mut m = TitleMenu::ready_for_reference();
    m.current_option = data["initial"].as_u64().unwrap() as usize;
    let rows = data["steps"].as_array().unwrap();
    assert_eq!(rows.len(), 16);
    for row in rows {
        let o = &row["observation"];
        assert_eq!(o["status"], "observed");
        assert_eq!(o["backend"], "existing_game_script");
        assert_eq!(o["room"], "rm_Title");
        assert_eq!(m.current_option, o["before"].as_u64().unwrap() as usize);
        m.input(match row["action"].as_str().unwrap() {
            "SelectRight" => Input::Right,
            "SelectLeft" => Input::Left,
            _ => panic!("Unsupported observed action"),
        });
        assert_eq!(m.current_option, o["after"].as_u64().unwrap() as usize);
        assert_eq!(m.screen, Screen::Title);
    }
    assert_eq!(m.current_option, data["final"].as_u64().unwrap() as usize);
}
#[test]
fn separate_room_and_gui_scales_at_4k() {
    let room = Viewport::fit((640, 360), (3840, 2160), ScalePolicy::IntegerWhenPossible).unwrap();
    let gui = Viewport::fit((1280, 720), (3840, 2160), ScalePolicy::IntegerWhenPossible).unwrap();
    assert_eq!(room.scale, 6.0);
    assert_eq!(gui.scale, 3.0);
    assert_eq!(
        room.to_output((320.0, 180.0)),
        gui.to_output((640.0, 360.0))
    );
    assert_eq!(room.to_logical((1920.0, 1080.0)), Some((320.0, 180.0)));
}
#[test]
fn ultrawide_bars_are_not_menu_hits() {
    let v = Viewport::fit((640, 360), (3440, 1440), ScalePolicy::IntegerWhenPossible).unwrap();
    assert_eq!(v.x, 440.0);
    assert_eq!(v.to_logical((439.0, 700.0)), None);
    assert_eq!(v.to_logical((3000.0, 700.0)), None);
    assert_eq!(v.to_logical((440.0, 0.0)), Some((0.0, 0.0)));
}
#[test]
fn small_windows_downscale_without_clipping() {
    let v = Viewport::fit((640, 360), (320, 180), ScalePolicy::IntegerWhenPossible).unwrap();
    assert_eq!(v.scale, 0.5);
    assert_eq!(v.width, 320.0);
}
#[test]
fn invalid_viewports_and_nonfinite_input_are_rejected() {
    assert!(Viewport::fit((0, 360), (1280, 720), ScalePolicy::Fit).is_err());
    let v = Viewport::fit((640, 360), (1280, 720), ScalePolicy::Fit).unwrap();
    assert!(v.to_logical((f64::NAN, 1.0)).is_none());
}

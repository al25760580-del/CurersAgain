//! Title pointer hit tests recovered from MouseOverButton 0x140905f80, case 27.
//! Case mapping is verified from the original runner's switch table: "titleButton".
//! Bounds are x±20/y±20, left/top inclusive and right/bottom exclusive.
//! Draw hover calls center (170+50*i,305); Global Left Pressed Mouse_53
//! at 0x1435c1050 calls center (170+50*i,310), then Confirmed WITHOUT assigning i.
//! Ordinary title scope: camera-relative logical 640×360, camera origin 0, scale 1.
//! Native helper overrides its supplied scale; arbitrary game-camera zoom is not covered here.
use super::menu::{Input, TitleMenu};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitKind {
    Hover,
    Click,
}
/// Point is already transformed from window coordinates into logical room coordinates.
pub fn hit(point: (f64, f64), kind: HitKind) -> Option<usize> {
    let y = match kind {
        HitKind::Hover => 305.0,
        HitKind::Click => 310.0,
    };
    (0..8).find(|i| {
        let x = 170.0 + *i as f64 * 50.0;
        point.0 >= x - 20.0 && point.0 < x + 20.0 && point.1 >= y - 20.0 && point.1 < y + 20.0
    })
}
/// Apply only actual motion, after click/keyboard processing. A stationary cursor must
/// not undo a keyboard selection. Returns whether the original select cue should play.
pub fn hover(menu: &mut TitleMenu, point: (f64, f64), moved: bool) -> bool {
    if !moved || !menu.can_control {
        return false;
    }
    menu.idle_time = 0.0;
    if let Some(i) = hit(point, HitKind::Hover) {
        if i != menu.current_option {
            menu.input(Input::Hover(i));
            return true;
        }
    }
    false
}
/// Global left-press validates a button zone but confirms the CURRENT selection, as native.
/// Hover is deliberately separate. No automatic input is sent to the original game.
pub fn left_press(menu: &mut TitleMenu, point: (f64, f64)) -> bool {
    if menu.can_control && hit(point, HitKind::Click).is_some() {
        menu.input(Input::Confirm);
        true
    } else {
        false
    }
}
#[cfg(test)]
mod tests {
    use super::super::menu::Screen;
    use super::*;
    #[test]
    fn native_half_open_edges() {
        assert_eq!(hit((150.0, 285.0), HitKind::Hover), Some(0));
        assert_eq!(hit((190.0, 305.0), HitKind::Hover), None);
        assert_eq!(hit((170.0, 325.0), HitKind::Hover), None);
    }
    #[test]
    fn hover_and_click_are_offset() {
        assert_eq!(hit((170.0, 287.0), HitKind::Hover), Some(0));
        assert_eq!(hit((170.0, 287.0), HitKind::Click), None);
        assert_eq!(hit((170.0, 327.0), HitKind::Click), Some(0));
        assert_eq!(hit((170.0, 330.0), HitKind::Click), None);
    }
    #[test]
    fn stationary_cursor_does_not_steal_keyboard_selection() {
        let mut m = TitleMenu::ready_for_reference();
        assert!(!hover(&mut m, (170.0, 305.0), false));
        assert_eq!(m.current_option, 3);
        assert!(hover(&mut m, (170.0, 305.0), true));
        assert_eq!(m.current_option, 0);
        assert!(!hover(&mut m, (170.0, 305.0), true));
    }
    #[test]
    fn click_confirms_current_option_not_implicit_hover() {
        let mut m = TitleMenu::ready_for_reference();
        assert!(left_press(&mut m, (170.0, 310.0)));
        assert_eq!(m.screen, Screen::Characters);
    }
    #[test]
    fn outside_and_control_gate_do_nothing() {
        let mut m = TitleMenu::default();
        assert!(!hover(&mut m, (170.0, 305.0), true));
        assert!(!left_press(&mut m, (170.0, 310.0)));
        m.can_control = true;
        assert!(!left_press(&mut m, (195.0, 310.0)));
        assert_eq!(m.screen, Screen::Title);
    }
}

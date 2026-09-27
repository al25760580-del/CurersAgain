//! Readable reconstruction: scene simulation, rendering and gameplay have separate ownership.
pub mod gameplay;
pub mod render;
pub mod scenes;
// Compatibility aliases preserve existing callers while the modules move.
pub use gameplay::attack_controller::*;
pub use render::outline as title_outline;
pub use scenes::startup;
pub use scenes::title::{
    effects as title_effects, menu, motion as title_dynamics, sparkle as title_sparkle,
};

pub mod audio;

pub mod first_run;

pub mod language;

pub mod translation_limits;

pub mod native_save;

pub mod score_server;

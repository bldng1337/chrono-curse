//! The game's main screen states and transitions between them.

mod game_over;
mod gameplay;
mod loading;
mod splash;
mod title;
mod worldgen;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();

    app.add_plugins((
        game_over::plugin,
        gameplay::plugin,
        loading::plugin,
        splash::plugin,
        title::plugin,
        worldgen::plugin,
    ));
}

/// The game's main screen states.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub enum Screen {
    #[default]
    Splash,
    Title,
    Loading,
    WorldGen,
    Gameplay,
    GameOver,
}

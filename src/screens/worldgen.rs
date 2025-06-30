//! A loading screen during which game assets are loaded if necessary.
//! This reduces stuttering, especially for audio on Wasm.

use bevy::prelude::*;

use crate::{game::worldgen::WorldGen, screens::Screen, theme::prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::WorldGen), spawn_loading_screen);

    app.add_systems(
        Update,
        enter_gameplay_screen.run_if(in_state(Screen::WorldGen).and(world_gen_finished)),
    );
}

fn spawn_loading_screen(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Loading Screen"),
        BackgroundColor(Color::BLACK),
        StateScoped(Screen::WorldGen),
        children![widget::label("Generating World...")],
    ));
}

fn enter_gameplay_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Gameplay);
}

fn world_gen_finished(resource_handles: Res<WorldGen>) -> bool {
    resource_handles.is_finished()
}

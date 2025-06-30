use bevy::prelude::*;

pub(crate) mod ghost;
mod knight;
mod statue;

#[derive(Clone, Default, Component)]
pub struct Enemy;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((statue::plugin, knight::plugin, ghost::plugin));
}

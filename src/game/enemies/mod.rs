use bevy::prelude::*;

mod knight;
mod statue;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((statue::plugin, knight::plugin));
}

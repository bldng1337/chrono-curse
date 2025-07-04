use bevy::prelude::*;

pub mod collider;
mod cosmetic;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        collider::plugin,
        cosmetic::plugin,
    ));
}

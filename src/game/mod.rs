use bevy::prelude::*;

pub mod worldgen;
mod player;
mod world;
mod inputs;
mod platforms;
mod animate;
mod enemies;
mod health;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        worldgen::plugin,
        world::plugin,
        inputs::plugin,
        player::plugin,
        platforms::plugin,
        animate::plugin,
        enemies::plugin,
        health::plugin,
    ));
}

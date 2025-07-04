use bevy::prelude::*;

pub(crate) mod age;
mod animate;
mod enemies;
mod health;
mod inputs;
mod platforms;
pub(crate) mod player;
mod projectile;
mod ui;
mod world;
pub mod worldgen;
mod ysort;

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
        ysort::plugin,
        age::plugin,
        projectile::plugin,
        ui::plugin,
    ));
}

use std::time::Duration;

use crate::{
    asset_tracking::LoadResource, game::age::Dead, screens::Screen, AgedSystems, AppSystems, PausableSystems
};
use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        execute_animations
            .in_set(AppSystems::Update)
            .in_set(AgedSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        PreUpdate,
        update_directions
            .in_set(AppSystems::Update)
            .in_set(AgedSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}
#[derive(Component, Default)]
pub struct Directional {
    pub flipdir: bool,
}

fn update_directions(
    mut query: Query<(&LinearVelocity, &mut Sprite, &Directional), Without<Dead>>,
) {
    for (vel, mut sprite, dir) in query.iter_mut() {
        if vel.0.x.abs() < 10.0 {
            continue;
        }
        let dir = (vel.0.x < 0.0) != dir.flipdir;
        sprite.flip_x = dir;
    }
}

#[derive(Component)]
pub struct AnimationConfig {
    first_sprite_index: usize,
    last_sprite_index: usize,
    fps: u8,
    frame_timer: Timer,
    looping: bool,
    playing: bool,
}

impl AnimationConfig {
    pub(crate) fn new(first: usize, last: usize, fps: u8, playing: bool, looping: bool) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps),
            looping: looping,
            playing: playing,
        }
    }

    fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }

    pub(crate) fn is_playing(&mut self) -> bool {
        self.playing
    }

    pub(crate) fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    pub(crate) fn set_frames(&mut self, first: usize, last: usize) {
        self.first_sprite_index = first;
        self.last_sprite_index = last;
    }

    pub(crate) fn set_fps(&mut self, fps: u8) {
        self.fps = fps;
    }

    pub(crate) fn play(&mut self) {
        if self.playing {
            return;
        }
        self.frame_timer = AnimationConfig::timer_from_fps(self.fps);
        self.playing = true;
    }

    pub(crate) fn stop(&mut self) {
        self.playing = false;
    }
}

// This system loops through all the sprites in the `TextureAtlas`, from  `first_sprite_index` to
// `last_sprite_index` (both defined in `AnimationConfig`).
fn execute_animations(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut Sprite), Without<Dead>>,
) {
    for (mut config, mut sprite) in &mut query {
        // We track how long the current sprite has been displayed for
        config.frame_timer.tick(time.delta());

        // If it has been displayed for the user-defined amount of time (fps)...
        if config.frame_timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                if !config.playing {
                    atlas.index = config.first_sprite_index;
                    return;
                }
                if atlas.index == config.last_sprite_index {
                    // ...and it IS the last frame, then we move back to the first frame and stop.
                    atlas.index = config.first_sprite_index;
                    if config.looping {
                        config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
                    } else {
                        config.playing = false;
                    }
                } else {
                    // ...and it is NOT the last frame, then we move to the next frame...
                    atlas.index += 1;
                    // ...and reset the frame timer to start counting all over again
                    config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
                }
            }
        }
    }
}

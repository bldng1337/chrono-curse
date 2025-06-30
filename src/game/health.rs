use std::time::Duration;

use bevy::{ecs::system::command, prelude::*};

use crate::{AgedSystems, AppSystems, PausableSystems, game::age::Dead, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (tick_timer, hurt_effect)
            .in_set(AppSystems::Update)
            .in_set(AgedSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Component)]
pub struct Health {
    pub(crate) health: f32,
    max_health: f32,
    hurt_time: Timer,
}

impl Health {
    pub fn new(health: f32) -> Self {
        Self {
            health: health,
            hurt_time: Timer::new(Duration::from_secs_f32(0.0), TimerMode::Once),
            max_health: health,
        }
    }

    pub fn get_percent(&self) -> f32 {
        self.health / self.max_health
    }

    pub fn damage(&mut self, damage: f32) {
        if !self.hurt_time.finished() {
            return;
        }
        self.health -= damage;
        self.hurt_time = Timer::new(Duration::from_secs_f32(0.5), TimerMode::Once);
    }
}

fn tick_timer(
    mut query: Query<(&mut Health, Entity), Without<Dead>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut health, entity) in query.iter_mut() {
        health.hurt_time.tick(time.delta());
        if health.health <= 0.0 {
            commands.entity(entity).insert(Dead);
        }
    }
}

fn hurt_effect(mut query: Query<(&Health, &mut Sprite)>) {
    for (health, mut sprite) in query.iter_mut() {
        if !health.hurt_time.finished() {
            let t = (health.hurt_time.elapsed_secs() * 10.0).floor() as i32;
            if t % 2 == 0 {
                sprite.color = Color::linear_rgb(1.0, 0.0, 0.0);
            } else {
                sprite.color = Color::linear_rgb(1.0, 1.0, 1.0);
            }
        }else{
            sprite.color = Color::linear_rgb(1.0, 1.0, 1.0);
        }
    }
}

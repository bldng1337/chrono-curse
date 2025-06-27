use std::time::Duration;

use bevy::prelude::*;

use crate::{AppSystems, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        tick_timer
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Component)]
pub struct Health {
    health: f32,
    hurt_time: Timer,
}

impl Health {
    pub fn new(health: f32) -> Self {
        Self {
            health: health,
            hurt_time: Timer::new(Duration::from_secs_f32(0.5), TimerMode::Once),
        }
    }

    pub fn damage(&mut self, damage: f32) {
        if !self.hurt_time.finished() {
            return;
        }
        self.health -= damage;
        println!("ouch");
        self.hurt_time = Timer::new(Duration::from_secs_f32(1.0), TimerMode::Once);
    }
}

fn tick_timer(mut query: Query<&mut Health>, time: Res<Time>) {
    for mut health in query.iter_mut() {
        health.hurt_time.tick(time.delta());
    }
}

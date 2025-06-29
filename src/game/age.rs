use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_egui::egui::lerp;

use crate::{AppSystems, game::health::Health, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        record.run_if(in_state(Screen::Gameplay).and(not(should_turnback))),
    );
    app.add_systems(
        Update,
        time_reverse
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay).and(should_turnback)),
    );
}

#[derive(Component)]
pub struct Aged {
    time: f64,
    turnback: bool,
    record: Timer,
}
impl Aged {
    pub fn try_set_turnback(&mut self, value: bool) {
        match (self.time, value) {
            (0.1.., true) => self.turnback = true,
            (_, false) | (..=0.0, _) => self.turnback = false,
            _ => (),
        }
    }
}

impl Default for Aged {
    fn default() -> Self {
        Self {
            time: 100.0,
            turnback: false,
            record: Timer::from_seconds(0.5, TimerMode::Once),
        }
    }
}

#[derive(Component, Default)]
pub struct Timed {
    history: VecDeque<Snapshot>,
    currtime: f64,
    currentsnapshot: Option<Snapshot>,
}
#[derive(Clone)]
struct Snapshot {
    time: f64,
    pos: Vec3,
    health: f32,
}

fn lerp_snapshot(snapa: Snapshot, snapb: Snapshot, t: f64) -> Snapshot {
    Snapshot {
        time: lerp(snapa.time..=snapb.time, t),
        health: lerp(snapa.health..=snapb.health, t as f32),
        pos: Vec3 {
            x: lerp(snapa.pos.x..=snapb.pos.x, t as f32),
            y: lerp(snapa.pos.y..=snapb.pos.y, t as f32),
            z: lerp(snapa.pos.z..=snapb.pos.z, t as f32),
        },
    }
}

fn should_turnback(aged_query: Query<&Aged>) -> bool {
    let Ok(aged) = aged_query.single() else {
        return false;
    };
    aged.turnback
}

fn time_reverse(
    mut query: Query<(&mut Transform, &mut Health, &mut Timed)>,
    mut no_health_query: Query<(&mut Transform, &mut Timed), Without<Health>>,
    mut aged_query: Query<&mut Aged>,
    time: Res<Time>,
) {
    let Ok(mut aged) = aged_query.single_mut() else {
        return;
    };
    aged.time -= time.delta_secs_f64();
    if aged.time <= 0.0 {
        aged.time = 0.0;
        aged.turnback = false;
    }
    for (mut transform, mut health, mut timed) in query.iter_mut() {
        match (
            (&timed.currentsnapshot).clone(),
            timed.history.back().cloned(),
        ) {
            (Some(curr), Some(prev)) => {
                timed.currtime -= time.delta_secs_f64();
                if timed.currtime < prev.time {
                    timed.currentsnapshot = timed.history.pop_back();
                    continue;
                }
                let delta = (timed.currtime - prev.time) / (curr.time - prev.time);
                let snapshot = lerp_snapshot(prev, curr, delta);
                transform.translation = snapshot.pos;
                health.health = snapshot.health;
            }
            (None, Some(_)) => {
                let time = timed.currtime;
                timed.currentsnapshot = Some(Snapshot {
                    time: time,
                    pos: transform.translation,
                    health: health.health,
                })
            }
            (Some(snapshot), None) => {
                transform.translation = snapshot.pos;
                health.health = snapshot.health;
            }
            _ => (),
        }
    }
    for (mut transform, mut timed) in no_health_query.iter_mut() {
        match (
            (&timed.currentsnapshot).clone(),
            timed.history.back().cloned(),
        ) {
            (Some(curr), Some(prev)) => {
                timed.currtime -= time.delta_secs_f64();
                if timed.currtime < prev.time {
                    timed.currentsnapshot = timed.history.pop_back();
                    continue;
                }
                let delta = (timed.currtime - prev.time) / (curr.time - prev.time);
                let snapshot = lerp_snapshot(prev, curr, delta);
                transform.translation = snapshot.pos;
            }
            (None, Some(_)) => {
                let time = timed.currtime;
                timed.currentsnapshot = Some(Snapshot {
                    time: time,
                    pos: transform.translation,
                    health: 0.0,
                })
            }
            (Some(snapshot), None) => {
                transform.translation = snapshot.pos;
            }
            _ => (),
        }
    }
}

fn record(
    mut query: Query<(&Transform, &Health, &mut Timed)>,
    mut no_health_query: Query<(&Transform, &mut Timed), Without<Health>>,
    mut aged_query: Query<&mut Aged>,
    time: Res<Time>,
) {
    let Ok(mut aged) = aged_query.single_mut() else {
        return;
    };
    aged.record.tick(time.delta());
    if !aged.record.finished() {
        return;
    }
    let elapsed = aged.record.elapsed_secs_f64();
    for (transform, health, mut timed) in query.iter_mut() {
        if timed.history.len() > 200 {
            for _ in 0..(timed.history.len() - 200) {
                timed.history.pop_front();
            }
        }
        timed.currtime += elapsed / 2.0;
        timed.currentsnapshot = None;
        let time = timed.currtime;
        timed.history.push_back(Snapshot {
            time: time,
            pos: transform.translation,
            health: health.health,
        });
    }
    for (transform, mut timed) in no_health_query.iter_mut() {
        if timed.history.len() > 200 {
            for _ in 0..(timed.history.len() - 200) {
                timed.history.pop_front();
            }
        }
        timed.currtime += elapsed;
        timed.currentsnapshot = None;
        let time = timed.currtime;
        timed.history.push_back(Snapshot {
            time: time,
            pos: transform.translation,
            health: 0.0,
        });
    }
    aged.record.reset();
}

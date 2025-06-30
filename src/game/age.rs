use std::{collections::VecDeque, f64::INFINITY};

use avian2d::prelude::{ColliderDisabled, RigidBodyDisabled};
use bevy::{
    math::{VectorSpace, f64},
    prelude::*,
};
use bevy_egui::egui::lerp;

use crate::{
    AgedSystems, AppSystems, PausableSystems, Turnback,
    game::{health::Health, player::Player},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PreUpdate, update_turnback);
    app.add_systems(
        Update,
        respawn
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        Update,
        record_spawn
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay).or(in_state(Screen::WorldGen))),
    );

    app.add_systems(
        Update,
        (tick_timer_record, record_death, die)
            .in_set(AgedSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        Update,
        (
            start_record,
            record_health,
            record_pos,
            record_sprite,
            finish_record,
        )
            .chain()
            .in_set(AgedSystems)
            .run_if(in_state(Screen::Gameplay).and(should_record)),
    );
    app.add_systems(
        Update,
        (time_reverse, reverse_health, reverse_pos, reverse_sprite)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay).and(in_state(Turnback(true)))),
    );
}

#[derive(PartialEq)]
pub enum Age {
    Young,
    Old,
    Ancient,
}

#[derive(Component)]
pub struct Aged {
    pub time: f64,
    turnback: bool,
    record: Timer,
}
impl Aged {
    pub fn to_age(&self) -> Age {
        match self.time {
            0.0..33.0 => Age::Ancient,
            33.3..66.6 => Age::Old,
            _ => Age::Young,
        }
    }
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
            record: Timer::from_seconds(0.2, TimerMode::Once),
        }
    }
}

#[derive(Component, Default)]
pub struct Dead;

#[derive(Component, Default)]
pub struct Timed {
    history: VecDeque<Snapshot>,
    currtime: f64,
    currentsnapshot: Option<Snapshot>,
    revertsnapshot: Option<Snapshot>,
}
#[derive(Clone, Debug)]
struct SpriteData {
    txt_layout: Option<TextureAtlas>,
    sprite: Handle<Image>,
    flip_x: bool,
}

#[derive(Clone, Debug)]
enum Snapshot {
    Moment {
        time: f64,
        pos: Vec3,
        //health_data
        health: f32,
        //sprite data
        sprite: Option<SpriteData>,
    },
    Spawn {
        time: f64,
    },
    Die {
        time: f64,
    },
}

impl Snapshot {
    fn get_time(&self) -> f64 {
        *match self {
            Snapshot::Moment {
                time,
                pos: _,
                health: _,
                sprite: _,
            } => time,
            Snapshot::Spawn { time } => time,
            Snapshot::Die { time } => time,
        }
    }
}

// #[derive(Clone)]
// struct Snapshot {
//     time: f64,
//     pos: Vec3,
//     health: f32,
// }

fn lerp_snapshot(snapa: Snapshot, snapb: Snapshot, t: f64) -> Snapshot {
    match (snapa, snapb) {
        (
            Snapshot::Moment {
                time: timea,
                pos: posa,
                health: healtha,
                sprite: spritea,
            },
            Snapshot::Moment {
                time: timeb,
                pos: posb,
                health: healthb,
                sprite: spriteb,
            },
        ) => Snapshot::Moment {
            time: lerp(timea..=timeb, t),
            health: lerp(healtha..=healthb, t as f32),
            pos: Vec3 {
                x: lerp(posa.x..=posb.x, t as f32),
                y: lerp(posa.y..=posb.y, t as f32),
                z: lerp(posa.z..=posb.z, t as f32),
            },
            sprite: if t < 0.5 { spritea } else { spriteb },
        },
        (snapa, snapb) => {
            if t < 0.5 {
                snapa
            } else {
                snapb
            }
        }
    }
}

// fn should_turnback(aged_query: Query<&Aged>) -> bool {
//     let Ok(aged) = aged_query.single() else {
//         return false;
//     };
//     aged.turnback
// }

fn update_turnback(mut next_turnback: ResMut<NextState<Turnback>>, aged_query: Query<&Aged>) {
    let should_turnback = match aged_query.single() {
        Ok(aged) => aged.turnback,
        Err(_) => false,
    };
    next_turnback.set(Turnback(should_turnback));
}

fn time_reverse(
    mut query: Query<(&mut Timed, Entity)>,
    mut aged_query: Query<&mut Aged>,
    mut command: Commands,
    time: Res<Time>,
) {
    let Ok(mut aged) = aged_query.single_mut() else {
        return;
    };
    aged.time -= time.delta_secs_f64() * 10.0;
    if aged.time <= 0.0 {
        aged.time = 0.0;
        aged.turnback = false;
    }
    for (mut timed, entity) in query.iter_mut() {
        match (
            (&timed.currentsnapshot).clone(),
            timed.history.back().cloned(),
        ) {
            (Some(curr), Some(prev)) => {
                timed.currtime -= time.delta_secs_f64();
                if timed.currtime < prev.get_time() {
                    match &timed.currentsnapshot {
                        Some(Snapshot::Die { time: _ }) => {
                            command.entity(entity).remove::<Dead>();
                        }
                        _ => (),
                    }
                    timed.currentsnapshot = timed.history.pop_back();
                    continue;
                }
                let delta =
                    (timed.currtime - prev.get_time()) / (curr.get_time() - prev.get_time());
                let snapshot = lerp_snapshot(prev, curr, delta);
                timed.revertsnapshot = Some(snapshot);
            }
            (None, Some(_)) => {
                timed.currentsnapshot = timed.history.pop_back();
                timed.revertsnapshot = timed.currentsnapshot.clone();
            }
            (Some(snapshot), None) => {
                timed.revertsnapshot = Some(snapshot);
            }
            _ => {
                timed.revertsnapshot = None;
            }
        }
        match &timed.revertsnapshot {
            Some(Snapshot::Spawn { time: _ }) => {
                command.entity(entity).despawn();
                timed.revertsnapshot = None;
            }
            _ => (),
        }
    }
}

fn reverse_pos(mut query: Query<(&Timed, &mut Transform)>) {
    for (timed, mut transform) in query.iter_mut() {
        if let Some(snapshot) = &timed.revertsnapshot {
            if let Snapshot::Moment {
                time: _,
                pos,
                health: _,
                sprite: _,
            } = snapshot
            {
                transform.translation = *pos;
            }
        }
    }
}

fn reverse_health(mut query: Query<(&Timed, &mut Health)>) {
    for (timed, mut healthcomp) in query.iter_mut() {
        if let Some(snapshot) = &timed.revertsnapshot {
            if let Snapshot::Moment {
                time: _,
                pos: _,
                health,
                sprite: _,
            } = snapshot
            {
                healthcomp.health = *health;
            }
        }
    }
}

fn reverse_sprite(mut query: Query<(&Timed, &mut Sprite)>) {
    for (timed, mut spritecomp) in query.iter_mut() {
        if let Some(snapshot) = &timed.revertsnapshot {
            if let Snapshot::Moment {
                time: _,
                pos: _,
                health: _,
                sprite,
            } = snapshot
            {
                if let Some(sprite) = sprite {
                    spritecomp.image = sprite.sprite.clone();
                    spritecomp.texture_atlas = sprite.txt_layout.clone();
                    spritecomp.flip_x = sprite.flip_x;
                }
            }
        }
    }
}

fn respawn(mut query: RemovedComponents<Dead>, mut command: Commands) {
    query.read().for_each(|entity| {
        if let Ok(mut entity) = command.get_entity(entity) {
            entity
                .remove::<(RigidBodyDisabled, ColliderDisabled)>()
                .insert(Visibility::Visible);
        }
    });
}

fn die(query: Query<Entity, Added<Dead>>, mut command: Commands) {
    for entity in query.iter() {
        command
            .entity(entity)
            .insert((RigidBodyDisabled, ColliderDisabled, Visibility::Hidden));
    }
}

fn record_spawn(
    mut query: Query<(&mut Timed), (Added<Timed>, Without<Player>)>,
    screen: Res<State<Screen>>,
) {
    if screen.into_inner() == &Screen::WorldGen {
        return;
    }
    for mut timed in query.iter_mut() {
        let time = timed.currtime;
        timed.history.push_back(Snapshot::Spawn { time: time });
    }
}

fn record_death(mut query: Query<(&mut Timed), Added<Dead>>) {
    for mut timed in query.iter_mut() {
        let time = timed.currtime;
        timed.history.push_back(Snapshot::Die { time: time });
    }
}

fn tick_timer_record(mut aged_query: Query<&mut Aged>, time: Res<Time>) {
    let Ok(mut aged) = aged_query.single_mut() else {
        return;
    };
    aged.record.tick(time.delta());
}
fn should_record(aged_query: Query<&Aged>) -> bool {
    let Ok(aged) = aged_query.single() else {
        return false;
    };
    aged.record.finished()
}

fn start_record(
    mut query: Query<(&mut Timed), Without<Dead>>,
    mut query_dead: Query<(&mut Timed, Entity), With<Dead>>,
    mut aged_query: Query<&mut Aged>,
    mut command: Commands,
) {
    let Ok(mut aged) = aged_query.single_mut() else {
        return;
    };
    const MAX: usize = 400;
    let elapsed = aged.record.elapsed_secs_f64();
    aged.record.reset();
    for mut timed in query.iter_mut() {
        if timed.history.len() > MAX {
            for _ in 0..(timed.history.len() - MAX) {
                timed.history.pop_front();
            }
        }
        timed.currtime += elapsed / 1.5;
        timed.revertsnapshot = None;
        let time = timed.currtime;
        timed.currentsnapshot = Some(Snapshot::Moment {
            time,
            pos: Vec3::ZERO,
            health: 0.0,
            sprite: None,
        });
    }
    for (mut timed, entity) in query_dead.iter_mut() {
        timed.currtime += elapsed / 1.5;
        if let Some(Snapshot::Die { time }) = timed.history.back() {
            if timed.currtime - 60.0 > *time {
                command.entity(entity).despawn();
            }
        }
    }
}

fn record_pos(mut query: Query<(&mut Timed, &Transform), Without<Dead>>) {
    for (mut timed, transform) in query.iter_mut() {
        if let Some(snapshot) = &timed.currentsnapshot {
            match snapshot {
                Snapshot::Moment {
                    time,
                    pos: _,
                    health,
                    sprite,
                } => {
                    timed.currentsnapshot = Some(Snapshot::Moment {
                        time: *time,
                        pos: transform.translation,
                        health: *health,
                        sprite: sprite.clone(), //We could also use None here since it will always be None but clone should not be much slower and is more robust
                    });
                }
                _ => (),
            }
        }
    }
}

fn record_health(mut query: Query<(&mut Timed, &Health), Without<Dead>>) {
    for (mut timed, healthcomp) in query.iter_mut() {
        if let Some(snapshot) = &timed.currentsnapshot {
            match snapshot {
                Snapshot::Moment {
                    time,
                    pos,
                    health: _,
                    sprite,
                } => {
                    timed.currentsnapshot = Some(Snapshot::Moment {
                        time: *time,
                        pos: *pos,
                        health: healthcomp.health,
                        sprite: sprite.clone(), //We could also use None here since it will always be None but clone should not be much slower and is more robust
                    });
                }
                _ => (),
            }
        }
    }
}

fn record_sprite(mut query: Query<(&mut Timed, &Sprite), Without<Dead>>) {
    for (mut timed, spritecomp) in query.iter_mut() {
        if let Some(snapshot) = &timed.currentsnapshot {
            match snapshot {
                Snapshot::Moment {
                    time,
                    pos,
                    health,
                    sprite: _,
                } => {
                    timed.currentsnapshot = Some(Snapshot::Moment {
                        time: *time,
                        pos: *pos,
                        health: *health,
                        sprite: Some(SpriteData {
                            txt_layout: spritecomp.texture_atlas.clone(),
                            sprite: spritecomp.image.clone(),
                            flip_x: spritecomp.flip_x,
                        }),
                    });
                }
                _ => (),
            }
        }
    }
}

fn finish_record(mut query: Query<&mut Timed, Without<Dead>>) {
    for mut timed in query.iter_mut() {
        if let Some(curr) = &timed.currentsnapshot {
            let curr = curr.clone();
            timed.currentsnapshot = None;
            timed.history.push_back(curr);
        }
    }
}

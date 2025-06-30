use std::f64::consts::E;

use avian2d::{
    parry::na::ComplexField,
    prelude::{
        Collider, GravityScale, LinearVelocity, LockedAxes, RayCaster, RigidBody, ShapeCaster,
        SpatialQuery, SpatialQueryFilter,
    },
};
use bevy::{platform::collections::HashSet, prelude::*};
use bevy_ecs_ldtk::{LdtkEntity, app::LdtkEntityAppExt};
use bevy_tnua::{
    TnuaAction, TnuaAnimatingState,
    builtins::{TnuaBuiltinDash, TnuaBuiltinJumpState},
    math::Float,
    prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController},
};
use bevy_tnua_avian2d::TnuaAvian2dSensorShape;
use rand::Rng;

use crate::{
    AgedSystems, AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    game::{
        age::{Dead, Timed},
        animate::{AnimationConfig, Directional},
        health::Health,
        player::Player,
        projectile::Projectile,
        ysort::{ENTITY_LAYER, YSort},
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<GhostAssets>();
    app.register_ldtk_entity_for_layer::<GhostBundle>("enemies", "ghost");

    app.add_systems(
        Update,
        init_ghost
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::WorldGen)),
    );
    app.add_systems(
        Update,
        update_ghost
            .in_set(AgedSystems)
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        Update,
        animate_ghost
            .in_set(AgedSystems)
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Clone, Default, Debug, PartialEq)]
enum State {
    #[default]
    Roaming,
    Aggro,
    Attacking,
}

#[derive(Clone, Component)]
struct Ghost {
    state: State,
    dir: Vec2,
    shootdir: Vec2,
    dirty: bool,
    attackcooldown: Timer,
    roamcooldown: Timer,
}

impl Default for Ghost {
    fn default() -> Self {
        Self {
            state: Default::default(),
            dir: Default::default(),
            dirty: Default::default(),
            shootdir: Default::default(),
            attackcooldown: Timer::from_seconds(1.0, TimerMode::Once),
            roamcooldown: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct GhostAssets {
    #[dependency]
    pub sprite_walk: Handle<Image>,
    pub atlas_walk: Handle<TextureAtlasLayout>,

    #[dependency]
    pub sprite_attack: Handle<Image>,
    pub atlas_attack: Handle<TextureAtlasLayout>,

    #[dependency]
    pub sprite_proj: Handle<Image>,
    pub atlas_proj: Handle<TextureAtlasLayout>,
}

impl FromWorld for GhostAssets {
    fn from_world(world: &mut World) -> Self {
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let layout = TextureAtlasLayout::from_grid(UVec2::new(704, 704), 3, 1, None, None);
        let atlas_attack = texture_atlas_layouts.add(layout);

        let layout = TextureAtlasLayout::from_grid(UVec2::new(704, 704), 3, 1, None, None);
        let atlas_walk = texture_atlas_layouts.add(layout);

        let layout = TextureAtlasLayout::from_grid(UVec2::new(224, 224), 9, 1, None, None);
        let atlas_proj = texture_atlas_layouts.add(layout);

        let assets = world.resource::<AssetServer>();
        Self {
            sprite_walk: assets.load("sprites/entities/enemies/Ghost/walk.png"),
            sprite_attack: assets.load("sprites/entities/enemies/Ghost/attack.png"),
            sprite_proj: assets.load("sprites/entities/enemies/Ghost/projectile.png"),
            atlas_walk,
            atlas_attack,
            atlas_proj,
        }
    }
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
struct GhostBundle {
    ghost: Ghost,
}

fn init_ghost(
    mut query: Query<(Entity, &mut Transform), Added<Ghost>>,
    mut commands: Commands,
    assets: Res<GhostAssets>,
) {
    for (entity, mut transform) in query.iter_mut() {
        let Ok(mut command) = commands.get_entity(entity) else {
            continue;
        };
        transform.translation.z = 2.0;
        let atlas = TextureAtlas {
            layout: assets.atlas_walk.clone(),
            index: 0,
        };
        command.insert((
            Timed::default(),
            Sprite {
                image: assets.sprite_walk.clone(),
                texture_atlas: Some(atlas.clone()),
                custom_size: Some(Vec2::new(100.0, 70.0)),
                ..Default::default()
            },
            AnimationConfig::new(0, 2, 4, true, true, Some(atlas), assets.sprite_walk.clone()),
            Directional {
                flipdir: true,
                ..Default::default()
            },
            YSort::new(ENTITY_LAYER, 64.0),
            GravityScale(0.0),
            RigidBody::Kinematic,
            Collider::capsule(16.0, 32.0),
            LockedAxes::ROTATION_LOCKED,
            Name::new("Ghost"),
        ));
    }
}

fn update_ghost(
    mut query: Query<(&GlobalTransform, &mut Ghost, &mut LinearVelocity, Entity), Without<Dead>>,
    mut player_query: Query<(&GlobalTransform, Entity, &mut Health), With<Player>>,
    spatial_query: SpatialQuery,
    time: Res<Time>,
) {
    let Ok((player, playerentity, mut health)) = player_query.single_mut() else {
        return;
    };
    let playerpos = player.translation().xy();
    for (transform, mut ghost, mut controller, entity) in query.iter_mut() {
        ghost.attackcooldown.tick(time.delta());
        ghost.roamcooldown.tick(time.delta());
        let ghostpos = transform.translation().xy();
        let mut sees_obstacle = false;
        let mut sees_floor = 0.0;
        let mut sees_player = false;
        let mut dir = ghost.dir;
        let filter = SpatialQueryFilter::default().with_excluded_entities([entity, playerentity]);
        if let Ok(dir2) = Dir2::from_xy(dir.x, dir.y) {
            let rayhit = spatial_query.cast_ray(ghostpos, dir2, 50.0, true, &filter);
            if let Some(rayhit) = rayhit {
                if rayhit.distance <= 120.0 {
                    sees_obstacle = true;
                }
            }
        }
        let rayhit = spatial_query.cast_ray(ghostpos, Dir2::NEG_Y, 1000.0, true, &filter);
        if let Some(rayhit) = rayhit {
            if rayhit.distance <= 150.0 {
                sees_floor = (200.0 - rayhit.distance) / 200.0;
            }
            if rayhit.distance >= 250.0 {
                sees_floor = -(rayhit.distance - 250.0) / 200.0;
            }
        }
        let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);
        if let Ok(ndir) = Dir2::new(playerpos - ghostpos) {
            let rayhit = spatial_query.cast_ray(ghostpos, ndir, 500.0, true, &filter);
            if let Some(rayhit) = rayhit {
                sees_player = rayhit.entity == playerentity;
            }
        }
        let mut nextstate = ghost.state.clone();
        match ghost.state {
            State::Roaming => {
                if sees_obstacle {
                    ghost.dir = ghost.dir * -1.0;
                }
                if sees_player {
                    nextstate = State::Aggro;
                }
                if ghost.roamcooldown.finished() {
                    let rng = &mut rand::thread_rng();
                    ghost.roamcooldown =
                        Timer::from_seconds(rng.gen_range(2.0..=4.0), TimerMode::Once);
                    ghost.dir = Vec2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0));
                }
            }
            State::Aggro => {
                ghost.dir = (playerpos - ghostpos).normalize();
                let distance = ghostpos.distance_squared(playerpos);
                if distance > (800.0).powi(2) && !sees_player {
                    nextstate = State::Roaming;
                }
                if sees_obstacle {
                    dir = ghost.dir * -1.0;
                }
                if distance < (400.0).powi(2) {
                    ghost.attackcooldown.reset();
                    nextstate = State::Attacking;
                }
            }
            State::Attacking => {
                let deltapos = playerpos - ghostpos;
                let distance = deltapos.length_squared();
                ghost.shootdir = deltapos;
                if distance < (300.0).powi(2) {
                    let dir = deltapos.normalize_or_zero();
                    ghost.dir = dir * -1.0;
                } else {
                    ghost.dir = Vec2::ZERO;
                }
                if distance > (500.0).powi(2) {
                    nextstate = State::Aggro;
                }
            }
        }
        dir.y += sees_floor;
        // dir.y /= 10.0;
        // println!("adsad {}", dir);
        controller.0 = dir * 100.0;
        if nextstate != ghost.state {
            ghost.state = nextstate;
            ghost.dirty = true;
        }
    }
}

fn animate_ghost(
    mut query: Query<
        (
            &mut Ghost,
            &mut AnimationConfig,
            &mut Sprite,
            Entity,
            &GlobalTransform,
        ),
        Without<Dead>,
    >,
    player_query: Query<Entity, With<Player>>,
    mut commands: Commands,
    assets: Res<GhostAssets>,
) {
    for (mut ghost, mut animconf, mut sprite, entity, global) in query.iter_mut() {
        if ghost.state == State::Attacking {
            sprite.flip_x = ghost.shootdir.x > 0.0;
            if let Some(map) = &sprite.texture_atlas {
                if map.index == 2 && ghost.attackcooldown.finished() {
                    if let Ok(player) = player_query.single() {
                        ghost.attackcooldown.reset();
                        let size = Vec2::splat(75.0);
                        let atlas = TextureAtlas {
                            layout: assets.atlas_proj.clone(),
                            index: 0,
                        };
                        commands.spawn((
                            Transform::from_translation(global.translation()),
                            Timed::default(),
                            Sprite {
                                image: assets.sprite_proj.clone(),
                                texture_atlas: Some(atlas.clone()),
                                custom_size: Some(size),
                                ..Default::default()
                            },
                            AnimationConfig::new(
                                0,
                                8,
                                4,
                                true,
                                true,
                                Some(atlas),
                                assets.sprite_proj.clone(),
                            ),
                            Directional {
                                flipdir: true,
                                ..Default::default()
                            },
                            Projectile {
                                dir: ghost.shootdir,
                                dmg: 15.0,
                                owner: entity,
                                target: Some(player),
                                size: size,
                            },
                        ));
                    }
                }
            }
        }

        if !ghost.dirty {
            continue;
        }
        ghost.dirty = false;
        match ghost.state {
            State::Roaming | State::Aggro => {
                animconf.update_sprite(
                    Some(TextureAtlas {
                        layout: assets.atlas_walk.clone(),
                        index: 0,
                    }),
                    assets.sprite_walk.clone(),
                );
                animconf.set_frames(0, 2);
                animconf.set_looping(true);
                animconf.play();
            }
            State::Attacking => {
                animconf.update_sprite(
                    Some(TextureAtlas {
                        layout: assets.atlas_attack.clone(),
                        index: 0,
                    }),
                    assets.sprite_attack.clone(),
                );
                animconf.set_frames(0, 2);
                animconf.set_looping(true);
                animconf.play();
            }
        }
    }
}

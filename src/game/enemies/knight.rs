use avian2d::{
    parry::na::ComplexField,
    prelude::{
        Collider, LockedAxes, RayCaster, RigidBody, ShapeCaster, SpatialQuery, SpatialQueryFilter,
    },
};
use bevy::prelude::*;
use bevy_ecs_ldtk::{LdtkEntity, app::LdtkEntityAppExt};
use bevy_tnua::{
    TnuaAction, TnuaAnimatingState,
    builtins::{TnuaBuiltinDash, TnuaBuiltinJumpState},
    math::Float,
    prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController},
};
use bevy_tnua_avian2d::TnuaAvian2dSensorShape;

use crate::{
    asset_tracking::LoadResource, game::{
        age::Timed, animate::{AnimationConfig, Directional}, health::Health, player::Player, ysort::{YSort, ENTITY_LAYER}
    }, screens::Screen, AppSystems
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<KnightAssets>();
    app.register_ldtk_entity_for_layer::<KnightBundle>("enemies", "knight");

    app.add_systems(
        Update,
        init_knight
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::WorldGen)),
    );
    app.add_systems(
        Update,
        update_knight
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        Update,
        animate_knight
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
struct Knight {
    state: State,
    dir: bool,
    dirty: bool,
    attackcooldown: Timer,
}

impl Default for Knight {
    fn default() -> Self {
        Self {
            state: Default::default(),
            dir: Default::default(),
            dirty: Default::default(),
            attackcooldown: Timer::from_seconds(1.0, TimerMode::Once),
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct KnightAssets {
    #[dependency]
    pub sprite_walk: Handle<Image>,
    pub atlas_walk: Handle<TextureAtlasLayout>,

    #[dependency]
    pub sprite_attack: Handle<Image>,
    pub atlas_attack: Handle<TextureAtlasLayout>,
}

impl FromWorld for KnightAssets {
    fn from_world(world: &mut World) -> Self {
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let layout = TextureAtlasLayout::from_grid(UVec2::new(256, 256), 9, 1, None, None);
        let atlas_attack = texture_atlas_layouts.add(layout);
        let layout = TextureAtlasLayout::from_grid(UVec2::new(512, 512), 4, 1, None, None);
        let atlas_walk = texture_atlas_layouts.add(layout);
        let assets = world.resource::<AssetServer>();
        Self {
            sprite_walk: assets.load("sprites/entities/enemies/TimeKnight/walk.png"),
            sprite_attack: assets.load("sprites/entities/enemies/TimeKnight/attack.png"),
            atlas_walk,
            atlas_attack,
        }
    }
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
struct KnightBundle {
    knight: Knight,
}

fn init_knight(
    mut query: Query<(Entity, &mut Transform), Added<Knight>>,
    mut commands: Commands,
    assets: Res<KnightAssets>,
) {
    for (entity, mut transform) in query.iter_mut() {
        let Ok(mut command) = commands.get_entity(entity) else {
            continue;
        };
        transform.translation.z = 2.0;
        command.insert((
            Timed::default(),
            Sprite {
                image: assets.sprite_walk.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: assets.atlas_walk.clone(),
                    index: 0,
                }),
                custom_size: Some(Vec2::new(100.0, 70.0)),
                ..Default::default()
            },
            YSort::new(ENTITY_LAYER, 64.0),
            RigidBody::Dynamic,
            Collider::capsule(16.0, 32.0),
            // TnuaAnimatingState::<AnimationState>::default(),
            TnuaController::default(),
            TnuaAvian2dSensorShape(Collider::rectangle(31.0, 0.0)),
            LockedAxes::ROTATION_LOCKED,
            Name::new("Knight"),
            AnimationConfig::new(0, 3, 8, true, true),
            Directional {
                flipdir: true,
                ..Default::default()
            },
            RayCaster::new(Vec2::ZERO, Dir2::X),
        ));
    }
}

fn update_knight(
    mut query: Query<(&GlobalTransform, &mut Knight, &mut TnuaController, Entity)>,
    mut player_query: Query<(&GlobalTransform, Entity, &mut Health), With<Player>>,
    spatial_query: SpatialQuery,
    time: Res<Time>,
) {
    let Ok((player, playerentity, mut health)) = player_query.single_mut() else {
        return;
    };
    let playerpos = player.translation().xy();
    for (transform, mut knight, mut controller, entity) in query.iter_mut() {
        knight.attackcooldown.tick(time.delta());
        let knightpos = transform.translation().xy();
        let mut sees_obstacle = false;
        let mut sees_player = false;
        let mut dir = match knight.dir {
            true => Vec2::new(1.0, 0.0),
            false => Vec2::new(-1.0, 0.0),
        };
        let filter = SpatialQueryFilter::default().with_excluded_entities([entity, playerentity]);
        let rayhit =
            spatial_query.cast_ray(knightpos + dir * 25.0, Dir2::NEG_Y, 50.0, true, &filter);
        if let Some(rayhit) = rayhit {
            if rayhit.distance <= 10.0 || rayhit.distance >= 35.0 {
                sees_obstacle = true;
            }
        } else {
            sees_obstacle = true;
        }
        let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);
        if let Ok(ndir) = Dir2::new(playerpos - knightpos) {
            let rayhit = spatial_query.cast_ray(knightpos, ndir, 500.0, true, &filter);
            if let Some(rayhit) = rayhit {
                sees_player = rayhit.entity == playerentity;
            }
        }
        let mut nextstate = knight.state.clone();
        // println!("My State is: {:?}", Knight.state);
        match knight.state {
            State::Roaming => {
                if sees_obstacle {
                    knight.dir = !knight.dir;
                    dir.x = 0.0;
                }
                if sees_player {
                    nextstate = State::Aggro;
                }
            }
            State::Aggro => {
                knight.dir = (playerpos.x - knightpos.x) > 0.0;
                let distance = knightpos.distance_squared(playerpos);
                if distance > (2000.0).powi(2) && !sees_player {
                    nextstate = State::Roaming;
                }
                if sees_obstacle {
                    dir.x = 0.0;
                }
                if distance < (200.0).powi(2) && knight.attackcooldown.finished() {
                    knight.attackcooldown.reset();
                    nextstate = State::Attacking;
                    dir.x = 0.0;
                }
            }
            State::Attacking => {
                let dir = (playerpos - knightpos).normalize();
                knight.dir = (playerpos.x - knightpos.x) > 0.0;
                controller.action(TnuaBuiltinDash {
                    displacement: Vec3::new(dir.x, dir.y, 0.0) * 300.0,
                    speed: 600.0,
                    allow_in_air: true,
                    acceleration: 800.0,
                    // acceleration: Float::INFINITY,
                    brake_acceleration: Float::INFINITY,
                    brake_to_speed: 250.0,
                    ..TnuaBuiltinDash::default()
                });
                let res = spatial_query.shape_intersections(
                    &Collider::circle(50.0),
                    knightpos,
                    0.0,
                    &SpatialQueryFilter::default(),
                );
                if res.contains(&playerentity) {
                    health.damage(15.0);
                }
                continue;
                // dir.x = 0.0;
            }
        }
        if nextstate != knight.state {
            knight.state = nextstate;
            knight.dirty = true;
        }
        controller.basis(TnuaBuiltinWalk {
            // The `desired_velocity` determines how the character will move.
            desired_velocity: Vec3::new(dir.x, 0.0, 0.0) * 100.0,
            acceleration: Float::INFINITY,
            // The `float_height` must be greater (even if by little) from the distance between the
            // character's center and the lowest point of its collider.
            float_height: 33.0,
            // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they have
            // sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn what they do.
            ..TnuaBuiltinWalk::default()
        });
    }
}

fn animate_knight(
    mut query: Query<(&mut Knight, &mut AnimationConfig, &mut Sprite)>,
    assets: Res<KnightAssets>,
) {
    for (mut Knight, mut animconf, mut sprite) in query.iter_mut() {
        if Knight.state == State::Attacking && !animconf.is_playing() {
            Knight.state = State::Aggro;
        }
        if !Knight.dirty {
            continue;
        }
        Knight.dirty = false;
        match Knight.state {
            State::Roaming | State::Aggro => {
                sprite.image = assets.sprite_walk.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: assets.atlas_walk.clone(),
                    index: 0,
                });
                animconf.set_frames(0, 3);
                animconf.set_looping(true);
                animconf.play();
            }
            State::Attacking => {
                sprite.image = assets.sprite_attack.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: assets.atlas_attack.clone(),
                    index: 0,
                });
                animconf.set_frames(0, 8);
                animconf.set_looping(false);
                animconf.play();
            }
        }
    }
}

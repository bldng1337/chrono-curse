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
    builtins::TnuaBuiltinJumpState,
    math::Float,
    prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController},
};
use bevy_tnua_avian2d::TnuaAvian2dSensorShape;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    game::{
        age::Timed,
        animate::{AnimationConfig, Directional},
        health::Health,
        player::Player,
        ysort::{ENTITY_LAYER, YSort},
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<StatueAssets>();
    app.register_ldtk_entity_for_layer::<StatueBundle>("enemies", "statue");

    app.add_systems(
        Update,
        init_statue
            .in_set(PausableSystems)
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::WorldGen)),
    );
    app.add_systems(
        Update,
        update_statue
            .in_set(PausableSystems)
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        Update,
        animate_statue
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

#[derive(Clone, Default, Component)]
struct Statue {
    state: State,
    dir: bool,
    dirty: bool,
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct StatueAssets {
    #[dependency]
    pub sprite_walk: Handle<Image>,
    pub atlas_walk: Handle<TextureAtlasLayout>,

    #[dependency]
    pub sprite_attack: Handle<Image>,
    pub atlas_attack: Handle<TextureAtlasLayout>,
}

impl FromWorld for StatueAssets {
    fn from_world(world: &mut World) -> Self {
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let layout = TextureAtlasLayout::from_grid(UVec2::new(448, 448), 5, 1, None, None);
        let atlas_walk = texture_atlas_layouts.add(layout);
        let layout = TextureAtlasLayout::from_grid(UVec2::new(512, 512), 4, 1, None, None);
        let atlas_attack = texture_atlas_layouts.add(layout);
        let assets = world.resource::<AssetServer>();
        Self {
            sprite_walk: assets.load("sprites/entities/enemies/statue/walk.png"),
            sprite_attack: assets.load("sprites/entities/enemies/statue/attack.png"),
            atlas_walk,
            atlas_attack,
        }
    }
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
struct StatueBundle {
    statue: Statue,
}

fn init_statue(
    mut query: Query<(Entity, &mut Transform), Added<Statue>>,
    mut commands: Commands,
    assets: Res<StatueAssets>,
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
            Name::new("Statue"),
            AnimationConfig::new(0, 3, 6, true, true),
            Directional {
                flipdir: true,
                ..Default::default()
            },
            RayCaster::new(Vec2::ZERO, Dir2::X),
        ));
    }
}

fn update_statue(
    mut query: Query<(&GlobalTransform, &mut Statue, &mut TnuaController, Entity)>,
    mut player_query: Query<(&GlobalTransform, Entity, &mut Health), With<Player>>,
    spatial_query: SpatialQuery,
) {
    let Ok((player, playerentity, mut health)) = player_query.single_mut() else {
        return;
    };
    let playerpos = player.translation().xy();
    for (transform, mut statue, mut controller, entity) in query.iter_mut() {
        let statuepos = transform.translation().xy();
        let mut sees_obstacle = false;
        let mut sees_player = false;
        let mut dir = match statue.dir {
            true => Vec2::new(1.0, 0.0),
            false => Vec2::new(-1.0, 0.0),
        };
        let filter = SpatialQueryFilter::default().with_excluded_entities([entity, playerentity]);
        let rayhit =
            spatial_query.cast_ray(statuepos + dir * 25.0, Dir2::NEG_Y, 50.0, true, &filter);
        if let Some(rayhit) = rayhit {
            if rayhit.distance <= 10.0 || rayhit.distance >= 35.0 {
                sees_obstacle = true;
            }
        } else {
            sees_obstacle = true;
        }
        let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);
        if let Ok(ndir) = Dir2::new(playerpos - statuepos) {
            let rayhit = spatial_query.cast_ray(statuepos, ndir, 500.0, true, &filter);
            if let Some(rayhit) = rayhit {
                sees_player = rayhit.entity == playerentity;
            }
        }
        let mut nextstate = statue.state.clone();
        // println!("My State is: {:?}", statue.state);
        match statue.state {
            State::Roaming => {
                if sees_obstacle {
                    statue.dir = !statue.dir;
                    dir.x = 0.0;
                }
                if sees_player {
                    nextstate = State::Aggro;
                }
            }
            State::Aggro => {
                statue.dir = (playerpos.x - statuepos.x) > 0.0;
                let distance = statuepos.distance_squared(playerpos);
                if distance > (2000.0).powi(2) && !sees_player {
                    nextstate = State::Roaming;
                }
                if sees_obstacle {
                    dir.x = 0.0;
                }
                if distance < (50.0).powi(2) {
                    nextstate = State::Attacking;
                    dir.x = 0.0;
                }
            }
            State::Attacking => {
                statue.dir = (playerpos.x - statuepos.x) > 0.0;
                let res = spatial_query.shape_intersections(
                    &Collider::circle(50.0),
                    statuepos,
                    0.0,
                    &SpatialQueryFilter::default(),
                );
                if res.contains(&playerentity) {
                    health.damage(15.0);
                }
                dir.x = 0.0;
            }
        }
        if nextstate != statue.state {
            statue.state = nextstate;
            statue.dirty = true;
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

fn animate_statue(
    mut query: Query<(&mut Statue, &mut AnimationConfig, &mut Sprite)>,
    assets: Res<StatueAssets>,
) {
    for (mut statue, mut animconf, mut sprite) in query.iter_mut() {
        if statue.state == State::Attacking && !animconf.is_playing() {
            statue.state = State::Aggro;
        }
        if !statue.dirty {
            continue;
        }
        statue.dirty = false;
        match statue.state {
            State::Roaming | State::Aggro => {
                sprite.image = assets.sprite_walk.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: assets.atlas_walk.clone(),
                    index: 0,
                });
                animconf.set_frames(0, 4);
                animconf.set_looping(true);
                animconf.play();
            }
            State::Attacking => {
                sprite.image = assets.sprite_attack.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: assets.atlas_attack.clone(),
                    index: 0,
                });
                animconf.set_frames(0, 3);
                animconf.set_looping(false);
                animconf.play();
            }
        }
    }
}

use avian2d::prelude::{Collider, LinearVelocity, LockedAxes, RigidBody};
use bevy::{
    core_pipeline::{
        bloom::Bloom,
        tonemapping::{DebandDither, Tonemapping},
    },
    prelude::*,
};
use bevy_ecs_ldtk::{LdtkEntity, app::LdtkEntityAppExt};
use bevy_light_2d::light::AmbientLight2d;
use bevy_tnua::{
    TnuaAction, TnuaAnimatingState,
    builtins::{TnuaBuiltinDash, TnuaBuiltinJumpState},
    prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController},
};
use bevy_tnua_avian2d::TnuaAvian2dSensorShape;

use crate::{
    AppSystems,
    asset_tracking::LoadResource,
    game::{
        animate::{AnimationConfig, Directional},
        health::Health,
        ysort::{ENTITY_LAYER, YSort},
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // app.register_type::<LevelAssets>();
    app.load_resource::<PlayerYoungAssets>();
    // app.init_resource::<WorldGen>();
    // app.add_systems(OnEnter(Screen::WorldGen), init_world_gen);
    app.add_systems(
        Update,
        handle_animating
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        Update,
        turn_book
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );

    app.add_systems(OnEnter(Screen::Gameplay), init_player);
    app.register_ldtk_entity::<PlayerSpawn>("player");
}

#[derive(Debug)]
pub enum AnimationState {
    Standing,
    Running(f32),
    Dashing,
    Jumping,
    Falling,
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerYoungAssets {
    #[dependency]
    pub sprite_idle: Handle<Image>,
    pub atlas_idle: Handle<TextureAtlasLayout>,

    #[dependency]
    pub sprite_run: Handle<Image>,
    pub atlas_run: Handle<TextureAtlasLayout>,

    #[dependency]
    pub sprite_jump: Handle<Image>,
    pub atlas_jump: Handle<TextureAtlasLayout>,

    #[dependency]
    pub sprite_book: Handle<Image>,
    pub atlas_book: Handle<TextureAtlasLayout>,
}

impl FromWorld for PlayerYoungAssets {
    fn from_world(world: &mut World) -> Self {
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let layout = TextureAtlasLayout::from_grid(UVec2::new(704, 704), 3, 1, None, None);
        let atlas_idle = texture_atlas_layouts.add(layout);
        let layout = TextureAtlasLayout::from_grid(UVec2::new(320, 320), 6, 1, None, None);
        let atlas_run = texture_atlas_layouts.add(layout);
        let layout = TextureAtlasLayout::from_grid(UVec2::new(320, 320), 8, 1, None, None);
        let atlas_jump = texture_atlas_layouts.add(layout);
        let layout = TextureAtlasLayout::from_grid(UVec2::new(280, 280), 8, 1, None, None);
        let atlas_book = texture_atlas_layouts.add(layout);
        let assets = world.resource::<AssetServer>();
        Self {
            sprite_idle: assets.load("sprites/entities/player/young/idle.png"),
            sprite_run: assets.load("sprites/entities/player/young/run.png"),
            sprite_jump: assets.load("sprites/entities/player/young/jump.png"),
            sprite_book: assets.load("sprites/entities/player/magicbook.png"),
            atlas_idle,
            atlas_run,
            atlas_jump,
            atlas_book,
        }
    }
}

#[derive(Clone, Default, Component)]
pub struct Player {
    pub dashtimer: Timer,
}

#[derive(Clone, Default, Component)]
pub struct Book;

fn init_player(
    mut commands: Commands,
    spawn: Query<&Transform, With<PlayerSpawn>>,
    cameras: Query<Entity, With<Camera2d>>,
    playerassets: Res<PlayerYoungAssets>,
) {
    for entity in cameras {
        commands.entity(entity).despawn();
    }
    let mut transform = spawn.iter().next().unwrap().clone();
    transform.translation.y += 32.0;
    transform.translation.z = 3.0;
    let texture_atlas = TextureAtlas {
        layout: playerassets.atlas_idle.clone(),
        index: 0,
    };
    let texture_atlas_book = TextureAtlas {
        layout: playerassets.atlas_book.clone(),
        index: 0,
    };
    commands
        .spawn((
            //Player
            transform,
            Health::new(100.0),
            YSort::new(ENTITY_LAYER, 64.0),
            // The player character needs to be configured as a dynamic rigid body of the physics
            // engine.
            RigidBody::Dynamic,
            Collider::capsule(16.0, 32.0),
            TnuaAnimatingState::<AnimationState>::default(),
            // This is Tnua's interface component.
            TnuaController::default(),
            Player {
                dashtimer: Timer::from_seconds(1.5, TimerMode::Once),
            },
            // A sensor shape is not strictly necessary, but without it we'll get weird results.
            TnuaAvian2dSensorShape(Collider::rectangle(31.0, 0.0)),
            // Tnua can fix the rotation, but the character will still get rotated before it can do so.
            // By locking the rotation we can prevent this.
            LockedAxes::ROTATION_LOCKED,
            Name::new("Player"),
            Sprite {
                image: playerassets.sprite_idle.clone(),
                texture_atlas: Some(texture_atlas),
                custom_size: Some(Vec2::new(100.0, 70.0)),
                ..Default::default()
            },
            AnimationConfig::new(0, 2, 8, true, true),
            Directional {
                flipdir: true,
                ..Default::default()
            },
        ))
        .with_child((
            //Book
            Transform::from_xyz(35.0, 10.0, 1.0),
            Book,
            Sprite {
                image: playerassets.sprite_book.clone(),
                texture_atlas: Some(texture_atlas_book),
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..Default::default()
            },
            AnimationConfig::new(0, 7, 8, true, true),
        ))
        .with_child((
            //Camera
            Camera2d,
            Camera {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            AmbientLight2d {
                brightness: 0.45,
                ..default()
            },
            Tonemapping::TonyMcMapface,
            Bloom::default(),
            DebandDither::Enabled,
        ));
}

#[derive(Clone, Default, Component, LdtkEntity)]
pub struct PlayerSpawn {}

fn turn_book(
    player_query: Query<(&LinearVelocity,), With<Player>>,
    mut book_query: Query<(&mut Transform, &mut Sprite), With<Book>>,
) {
    let Ok((vel,)) = player_query.single() else {
        return;
    };
    let Ok((mut transform, mut sprite)) = book_query.single_mut() else {
        return;
    };
    if vel.x.abs() < 100.0 {
        return;
    }
    let dir = vel.x.signum();
    transform.translation.x = 35.0 * dir;
    sprite.flip_x = dir > 0.0;
}

fn handle_animating(
    mut player_query: Query<
        (
            &TnuaController,
            &mut TnuaAnimatingState<AnimationState>,
            &mut Sprite,
            &mut AnimationConfig,
        ),
        With<Player>,
    >,
    playerassets: Res<PlayerYoungAssets>,
) {
    let Ok((controller, mut animating_state, mut sprite, mut animation)) =
        player_query.single_mut()
    else {
        return;
    };
    let current_status_for_animating = match controller.action_name() {
        // Unless you provide the action names yourself, prefer matching against the `NAME` const
        // of the `TnuaAction` trait. Once `type_name` is stabilized as `const` Tnua will use it to
        // generate these names automatically, which may result in a change to the name.
        Some(TnuaBuiltinJump::NAME) => {
            // In case of jump, we want to cast it so that we can get the concrete jump state.
            let (_, jump_state) = controller
                .concrete_action::<TnuaBuiltinJump>()
                .expect("action name mismatch");
            // Depending on the state of the jump, we need to decide if we want to play the jump
            // animation or the fall animation.
            match jump_state {
                TnuaBuiltinJumpState::NoJump => return,
                TnuaBuiltinJumpState::StartingJump { .. } => AnimationState::Jumping,
                TnuaBuiltinJumpState::SlowDownTooFastSlopeJump { .. } => AnimationState::Jumping,
                TnuaBuiltinJumpState::MaintainingJump { .. } => AnimationState::Jumping,
                TnuaBuiltinJumpState::StoppedMaintainingJump => AnimationState::Jumping,
                TnuaBuiltinJumpState::FallSection => AnimationState::Falling,
            }
        }
        Some(TnuaBuiltinDash::NAME) => AnimationState::Dashing,
        // Tnua should only have the `action_name` of the actions you feed to it. If it has
        // anything else - consider it a bug.
        Some(other) => panic!("Unknown action {other}"),
        // No action name means that no action is currently being performed - which means the
        // animation should be decided by the basis.
        None => {
            // If there is no action going on, we'll base the animation on the state of the
            // basis.
            let Some((_, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>() else {
                // Since we only use the walk basis in this example, if we can't get get this
                // basis' state it probably means the system ran before any basis was set, so we
                // just stkip this frame.
                return;
            };
            if basis_state.standing_on_entity().is_none() {
                // The walk basis keeps track of what the character is standing on. If it doesn't
                // stand on anything, `standing_on_entity` will be empty - which means the
                // character has walked off a cliff and needs to fall.
                AnimationState::Falling
            } else {
                let speed = basis_state.running_velocity.length();
                if 0.01 < speed {
                    AnimationState::Running(0.1 * speed)
                } else {
                    AnimationState::Standing
                }
            }
        }
    };

    let animating_directive = animating_state.update_by_discriminant(current_status_for_animating);

    match animating_directive {
        bevy_tnua::TnuaAnimatingStateDirective::Maintain { state } => {}
        bevy_tnua::TnuaAnimatingStateDirective::Alter { old_state, state } => match state {
            AnimationState::Standing => {
                sprite.image = playerassets.sprite_idle.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: playerassets.atlas_idle.clone(),
                    index: 0,
                });
                animation.set_frames(0, 2);
                animation.play();
            }
            AnimationState::Running(_) => {
                sprite.image = playerassets.sprite_run.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: playerassets.atlas_run.clone(),
                    index: 1,
                });
                animation.set_frames(1, 5);
                animation.play();
            }
            AnimationState::Jumping => {
                sprite.image = playerassets.sprite_jump.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: playerassets.atlas_jump.clone(),
                    index: 0,
                });
                animation.set_frames(0, 5);
                animation.play();
            }
            AnimationState::Falling => {
                sprite.image = playerassets.sprite_jump.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: playerassets.atlas_jump.clone(),
                    index: 4,
                });
                animation.stop();
            }
            AnimationState::Dashing => {
                sprite.image = playerassets.sprite_idle.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: playerassets.atlas_idle.clone(),
                    index: 0,
                });
                animation.set_frames(0, 0);
                animation.stop();
            }
        },
    }
}

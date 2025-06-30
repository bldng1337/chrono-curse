use crate::game::age::Timed;
use crate::game::animate::{AnimationConfig, Directional};
use crate::game::enemies::ghost::GhostAssets;
use crate::game::player::Book;
use crate::game::projectile::ProjectileTarget;
use crate::{
    AgedSystems, AppSystems, PausableSystems,
    game::{
        age::Aged,
        player::{Player, SpellCap},
        projectile::Projectile,
    },
    screens::Screen,
};
use bevy::math::VectorSpace;
use bevy::prelude::Vec2;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_enhanced_input::prelude::*;
use bevy_light_2d::light::PointLight2d;
use bevy_tnua::{builtins::TnuaBuiltinDash, math::Float, prelude::*};

#[derive(InputContext)]
struct DefaultContext;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Attack;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Turnback;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Jump;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Dash;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

pub(super) fn plugin(app: &mut App) {
    app.add_input_context::<DefaultContext>();
    app.add_systems(Startup, init_inputs);
    // app.add_observer(apply_movement);

    app.add_systems(
        Update,
        aged.in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        Update,
        shoot
            .in_set(AppSystems::Update)
            .in_set(AgedSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        Update,
        movement
            .in_set(AppSystems::Update)
            .in_set(AgedSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

fn init_inputs(mut commands: Commands) {
    let mut actions = Actions::<DefaultContext>::default();
    actions
        .bind::<Jump>()
        .to((KeyCode::Space, GamepadButton::South));
    actions.bind::<Dash>().to((
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
        GamepadButton::Start,
    ));
    actions.bind::<Attack>().to((MouseButton::Left,));
    actions.bind::<Turnback>().to((KeyCode::KeyR,));
    actions
        .bind::<Move>()
        .to((
            Cardinal::wasd_keys(),
            Cardinal::arrow_keys(),
            Axial::left_stick(),
            // (KeyCode::Space, GamepadButton::South).with_modifiers_each(SwizzleAxis::YYY),
        ))
        .with_modifiers(DeadZone::default()); //, SmoothNudge::default()
    commands.spawn(actions);
}

fn aged(actions: Single<&Actions<DefaultContext>>, mut aged: Single<&mut Aged, With<Player>>) {
    let actions = actions.into_inner();
    let mut aged = aged.into_inner();
    aged.try_set_turnback(actions.state::<Turnback>().unwrap() == ActionState::Fired);
}

fn shoot(
    actions: Single<&Actions<DefaultContext>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut spells: Single<(&mut SpellCap, Entity), With<Player>>,
    book: Single<(&GlobalTransform), With<Book>>,
    mut command: Commands,
    assets: Res<GhostAssets>,
    time: Res<Time>,
) {
    let (mut spells, entity) = spells.into_inner();
    spells.timer.tick(time.delta());
    let (transform) = book.into_inner();
    let Ok((camera, camera_transform)) = q_camera.single() else {
        return;
    };

    let Ok(window) = q_window.single() else {
        return;
    };

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        let pos = world_position;
        let shootpos=transform.translation();
        if actions.state::<Attack>().unwrap() == ActionState::Fired && spells.timer.finished() {
            spells.timer =
                Timer::from_seconds((1.0 / (1.0 + spells.speed / 10.0)) as f32, TimerMode::Once);
            let dir = (pos - shootpos.xy()).normalize();
            let atlas = TextureAtlas {
                layout: assets.atlas_proj.clone(),
                index: 0,
            };
            let size = Vec2::splat(60.0);
            let mut spellcommand = command.spawn((
                StateScoped(Screen::Gameplay),
                Transform::from_translation(shootpos),
                Timed::default(),
                Directional {
                    flipdir: true,
                    ..Default::default()
                },
                Projectile {
                    target: ProjectileTarget::Enemies,
                    size: size / 2.0,
                    dmg: (40.0 + spells.strength * 20.0) as f32,
                    dir: dir * 230.0 + (spells.speed * 50.0) as f32, //
                },
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
                    Some(atlas.clone()),
                    assets.sprite_proj.clone(),
                ),
            ));
            for item in &spells.items {
                match item.name.as_str() {
                    "Book of Fire" => {
                        let atlas = TextureAtlas {
                            layout: assets.atlas_proj_fire.clone(),
                            index: 0,
                        };
                        spellcommand.insert((
                            Sprite {
                                image: assets.sprite_proj_fire.clone(),
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
                                Some(atlas.clone()),
                                assets.sprite_proj_fire.clone(),
                            ),
                        ));
                    }
                    "Book of Current" => {
                        let atlas = TextureAtlas {
                            layout: assets.atlas_proj_electro.clone(),
                            index: 0,
                        };
                        spellcommand.insert((
                            Sprite {
                                image: assets.sprite_proj_electro.clone(),
                                texture_atlas: Some(atlas.clone()),
                                custom_size: Some(size),
                                ..Default::default()
                            },
                            AnimationConfig::new(
                                3,
                                5,
                                4,
                                true,
                                true,
                                Some(atlas.clone()),
                                assets.sprite_proj_electro.clone(),
                            ),
                        ));
                    }
                    "Duplex" => {
                        // command.spawn((
                        //     Transform::from_translation(transform.translation() + Vec3::Y * 30.0),
                        //     Timed::default(),
                        //     Directional {
                        //         flipdir: true,
                        //         ..Default::default()
                        //     },
                        //     Projectile {
                        //         owner: entity,
                        //         target: None,
                        //         size: size / 2.0,
                        //         dmg: (spells.strength * 2.0) as f32,
                        //         dir: dir * 200.0 + (spells.speed * 50.0) as f32,
                        //     },
                        //     Sprite {
                        //         image: assets.sprite_proj.clone(),
                        //         texture_atlas: Some(atlas.clone()),
                        //         custom_size: Some(size),
                        //         ..Default::default()
                        //     },
                        //     AnimationConfig::new(
                        //         0,
                        //         8,
                        //         4,
                        //         true,
                        //         true,
                        //         Some(atlas.clone()),
                        //         assets.sprite_proj.clone(),
                        //     ),
                        // ));
                    }
                    _ => (),
                }
            }
        }
    }
}

fn movement(
    actions: Single<&Actions<DefaultContext>>,
    mut query: Query<(&mut TnuaController, &mut Aged, &mut Player)>,
    time: Res<Time>,
) {
    let Ok((mut controller, mut aged, mut player)) = query.single_mut() else {
        return;
    };
    player.dashtimer.tick(time.delta());
    let actions = actions.into_inner();

    let direction = actions.value::<Move>().unwrap();

    controller.basis(TnuaBuiltinWalk {
        // The `desired_velocity` determines how the character will move.
        desired_velocity: Vec3::new(direction.x, 0.0, 0.0) * 250.0,
        acceleration: Float::INFINITY,
        // The `float_height` must be greater (even if by little) from the distance between the
        // character's center and the lowest point of its collider.
        float_height: 33.0,
        air_acceleration: 800.0,
        // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they have
        // sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn what they do.
        ..TnuaBuiltinWalk::default()
    });

    if actions.state::<Dash>().unwrap() == ActionState::Fired
        && direction.length_squared() > 0.0
        && player.dashtimer.finished()
    {
        player.dashtimer.reset();
        controller.action(TnuaBuiltinDash {
            displacement: Vec3::new(direction.x, direction.y, 0.0) * 70.0,
            speed: 800.0,
            allow_in_air: true,
            acceleration: Float::INFINITY,
            brake_acceleration: Float::INFINITY,
            brake_to_speed: 250.0,
            ..TnuaBuiltinDash::default()
        });
    }

    if actions.state::<Jump>().unwrap() == ActionState::Fired || direction.y > 0.0 {
        controller.action(TnuaBuiltinJump {
            height: 90.0,
            ..TnuaBuiltinJump::default()
        });
    }
}

// /// Apply movement when `Move` action considered fired.
// fn apply_movement(mtrigger: Trigger<Fired<Move>>, mut query: Query<&mut TnuaController>) {
//     let Ok(mut controller) = query.single_mut() else {
//         return;
//     };
//     let direction = mtrigger.value;
//     controller.basis(TnuaBuiltinWalk {
//         // The `desired_velocity` determines how the character will move.
//         desired_velocity: Vec3::new(direction.x, 0.0, 0.0) * 64.0,
//         // The `float_height` must be greater (even if by little) from the distance between the
//         // character's center and the lowest point of its collider.
//         float_height: 33.0,
//         // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they have
//         // sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn what they do.
//         ..TnuaBuiltinWalk::default()
//     });
//     if mtrigger.value.y > 0.0 {
//         controller.action(TnuaBuiltinJump {
//             // The height is the only mandatory field of the jump button.
//             height: 80.0,
//             // `TnuaBuiltinJump` also has customization fields with sensible defaults.
//             ..TnuaBuiltinJump::default()
//         });
//     }
//     println!("{:?}", mtrigger.value);

//     // We defined the output of `Move` as `Vec2`,
//     // but since translation expects `Vec3`, we extend it to 3 axes.
//     // transform.translation += trigger.value.extend(0.0);
// }

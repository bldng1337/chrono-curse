use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_tnua::{builtins::TnuaBuiltinDash, math::Float, prelude::*};

use crate::{
    AppSystems,
    game::{age::Aged, player::Player},
    screens::Screen,
};

#[derive(InputContext)]
struct Default;

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
    app.add_input_context::<Default>();
    app.add_systems(Startup, init_inputs);
    // app.add_observer(apply_movement);
    app.add_systems(
        Update,
        movement
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
}

fn init_inputs(mut commands: Commands) {
    let mut actions = Actions::<Default>::default();
    actions
        .bind::<Jump>()
        .to((KeyCode::Space, GamepadButton::South));
    actions.bind::<Dash>().to((
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
        GamepadButton::Start,
    ));
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

fn movement(
    actions: Single<&Actions<Default>>,
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
        air_acceleration: 500.0,
        // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they have
        // sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn what they do.
        ..TnuaBuiltinWalk::default()
    });
    aged.try_set_turnback(actions.state::<Turnback>().unwrap() == ActionState::Fired);
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

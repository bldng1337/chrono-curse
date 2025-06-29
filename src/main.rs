// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
mod audio;
#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod menus;
mod screens;
mod theme;

use avian2d::{
    PhysicsPlugins,
    prelude::{Gravity, Physics, PhysicsDebugPlugin},
};
use bevy::{asset::AssetMetaCheck, ecs::schedule::ScheduleLabel, prelude::*};
use bevy_ecs_ldtk::LdtkPlugin;
use bevy_enhanced_input::prelude::*;
use bevy_light_2d::prelude::Light2dPlugin;
use bevy_tnua::prelude::TnuaControllerPlugin;
use bevy_tnua_avian2d::TnuaAvian2dPlugin;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Game".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
        );
        app.add_plugins((
            LdtkPlugin,
            PhysicsPlugins::default(),
            EnhancedInputPlugin,
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian2dPlugin::new(FixedUpdate),
            Light2dPlugin,
        ));

        // Add other plugins.
        app.add_plugins((
            asset_tracking::plugin,
            audio::plugin,
            game::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            menus::plugin,
            screens::plugin,
            theme::plugin,
        ))
        .insert_resource(Gravity(Vec2::NEG_Y * 19.6 * 38.0));
        app.insert_resource(Time::<Physics>::default());
        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, ScheduleLabel)]
struct PausableFixedSystems;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Name::new("Camera"), Camera2d));
}

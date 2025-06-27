//! Development tools for the game. This plugin is only enabled in dev builds.

use avian2d::prelude::{PhysicsDebugPlugin, PhysicsGizmos};
use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_just_pressed, prelude::*,
    ui::UiDebugOptions,
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);
    app.insert_resource(Debug(false));
    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
    );
    app.add_plugins(EguiPlugin {
        enable_multipass_for_primary_context: true,
    });
    app.add_plugins((
        WorldInspectorPlugin::new().run_if(resource_equals(Debug(true))),
        PhysicsDebugPlugin::default(),
    ));
    app.insert_gizmo_config(
        PhysicsGizmos {
            raycast_color: Some(Color::NONE),
            ..default()
        },
        GizmoConfig {
            enabled: false,
            ..Default::default()
        },
    );
}

#[derive(Resource, PartialEq)]
struct Debug(bool);

const TOGGLE_KEY: KeyCode = KeyCode::End;

fn toggle_debug_ui(
    mut options: ResMut<UiDebugOptions>,
    mut debug: ResMut<Debug>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    options.toggle();
    debug.0 = !debug.0;
    let (config, _) = config_store.config_mut::<PhysicsGizmos>();
    config.enabled = !config.enabled;
}

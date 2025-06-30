//! A splash screen that plays briefly at startup.

use crate::game::age::Dead;
use crate::game::player::Player;
use crate::{AppSystems, theme::prelude::*};
use crate::{asset_tracking::ResourceHandles, menus::Menu, screens::Screen, theme::widget};
use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    input::common_conditions::input_just_pressed,
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    // app.add_systems(OnEnter(Screen::GameOver), spawn_splash_screen);
    // app.add_systems(
    //     Update,
    //     game_over
    //         .in_set(AppSystems::Update)
    //         .run_if(in_state(Screen::Gameplay)),
    // );
}

const SPLASH_BACKGROUND_COLOR: Color = Color::srgb(0.157, 0.157, 0.157);

fn spawn_splash_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        widget::ui_root("Game Over"),
        BackgroundColor(SPLASH_BACKGROUND_COLOR),
        StateScoped(Screen::GameOver),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Auto,
            ..default()
        },
        children![
            (
                Name::new("Splash image"),
                Node {
                    margin: UiRect::all(Val::Auto),
                    height: Val::Percent(30.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                children![
                    (
                        Name::new("Game"),
                        ImageNode::new(asset_server.load_with_settings(
                            // This should be an embedded asset for instant loading, but that is
                            // currently [broken on Windows Wasm builds](https://github.com/bevyengine/bevy/issues/14246).
                            "UIElements/Over.png",
                            |settings: &mut ImageLoaderSettings| {
                                // Make an exception for the splash image in case
                                // `ImagePlugin::default_nearest()` is used for pixel art.
                                settings.sampler = ImageSampler::nearest();
                            },
                        )),
                    ),
                    (
                        Name::new("Over"),
                        ImageNode::new(asset_server.load_with_settings(
                            // This should be an embedded asset for instant loading, but that is
                            // currently [broken on Windows Wasm builds](https://github.com/bevyengine/bevy/issues/14246).
                            "UIElements/Game.png",
                            |settings: &mut ImageLoaderSettings| {
                                // Make an exception for the splash image in case
                                // `ImagePlugin::default_nearest()` is used for pixel art.
                                settings.sampler = ImageSampler::nearest();
                            },
                        )),
                    )
                ]
            ),
            (widget::button(
                "Back to Title",
                quit_to_title,
                &asset_server
            ),),
        ],
    ));
}

fn quit_to_title(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn game_over(
    query: Query<(), (With<Player>, Added<Dead>)>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    match query.iter().next() {
        Some(_) => next_screen.set(Screen::GameOver),
        None => (),
    }
}

//! The main menu (seen on the title screen).

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use crate::{
    asset_tracking::ResourceHandles, audio::music, menus::Menu, screens::Screen, theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu);
}

fn spawn_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        widget::ui_root("Main Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::Main),
        music(asset_server.load("audio/music/titlescreen/Chrono_Intro.ogg")),
        #[cfg(not(target_family = "wasm"))]
        children![
            (
                Name::new("Title"),
                Node {
                    // margin: UiRect::all(Val::Auto),
                    width: Val::Percent(20.0),
                    ..default()
                },
                ImageNode::new(asset_server.load_with_settings(
                    // This should be an embedded asset for instant loading, but that is
                    // currently [broken on Windows Wasm builds](https://github.com/bevyengine/bevy/issues/14246).
                    "UIElements/Chrono_Curse.png",
                    |settings: &mut ImageLoaderSettings| {
                        // Make an exception for the splash image in case
                        // `ImagePlugin::default_nearest()` is used for pixel art.
                        settings.sampler = ImageSampler::nearest();
                    },
                )),
            ),
            widget::button("Play", enter_loading_or_gameplay_screen, &asset_server),
            widget::button("Settings", open_settings_menu, &asset_server),
            widget::button("Controls", open_credits_menu, &asset_server),
            widget::button("Exit", exit_app, &asset_server),
        ],
        #[cfg(target_family = "wasm")]
        children![
            widget::button("Play", enter_loading_or_gameplay_screen),
            widget::button("Settings", open_settings_menu),
            widget::button("Credits", open_credits_menu),
        ],
    ));
}

fn enter_loading_or_gameplay_screen(
    _: Trigger<Pointer<Click>>,
    resource_handles: Res<ResourceHandles>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if resource_handles.is_all_done() {
        next_screen.set(Screen::WorldGen);
    } else {
        next_screen.set(Screen::Loading);
    }
}

fn open_settings_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn open_credits_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Credits);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: Trigger<Pointer<Click>>, mut app_exit: EventWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}

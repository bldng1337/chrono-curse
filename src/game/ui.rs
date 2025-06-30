use avian2d::parry::query;
use bevy::{math::VectorSpace, prelude::*};

use std::borrow::Cow;

use bevy::{
    ecs::{spawn::SpawnWith, system::IntoObserverSystem},
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
    ui::Val::*,
};

use crate::{
    AppSystems, PausableSystems,
    game::{age::Aged, health::Health, player::Player},
    screens::Screen,
};

#[derive(Component)]
struct Pointer;
#[derive(Component)]
struct FillBar;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), setup);
    app.add_systems(
        Update,
        (update, update_age)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("UIRoot"),
        StateScoped(Screen::Gameplay),
        Node {
            position_type: PositionType::Absolute,
            width: Percent(100.0),
            height: Percent(100.0),
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            flex_direction: FlexDirection::Row,
            row_gap: Auto,
            ..default()
        },
        Pickable::IGNORE,
        children![
            (
                Pickable::IGNORE,
                Name::new("UIBar"),
                ImageNode::new(asset_server.load_with_settings(
                    "UIElements/healthbar_background.png",
                    |settings: &mut ImageLoaderSettings| {
                        settings.sampler = ImageSampler::nearest();
                    },
                )),
                Node {
                    width: Px(180.0),
                    height: Px(180.0),
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                children![(
                    Name::new("UIBarFill"),
                    FillBar,
                    Node {
                        width: Px(180.0),
                        height: Px(180.0),
                        overflow: Overflow::clip(),
                        align_items: AlignItems::Start,
                        justify_content: JustifyContent::Start,
                        align_content: AlignContent::Start,
                        justify_items: JustifyItems::Start,
                        ..default()
                    },
                    children![(
                        Node {
                            width: Px(180.0),
                            height: Px(180.0),
                            overflow: Overflow::clip(),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ImageNode {
                            image: asset_server.load_with_settings(
                                "UIElements/healthbar_fill.png",
                                |settings: &mut ImageLoaderSettings| {
                                    settings.sampler = ImageSampler::nearest();
                                },
                            ),
                            ..Default::default()
                        }
                    )]
                ),]
            ),
            (
                Pickable::IGNORE,
                Name::new("TimeBar"),
                ImageNode::new(asset_server.load_with_settings(
                    "UIElements/Uhr.png",
                    |settings: &mut ImageLoaderSettings| {
                        settings.sampler = ImageSampler::nearest();
                    },
                )),
                Node {
                    width: Px(180.0),
                    height: Px(180.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![(
                    Pickable::IGNORE,
                    Name::new("clock hand"),
                    Pointer,
                    ImageNode::new(asset_server.load_with_settings(
                        "UIElements/Zeiger_.png",
                        |settings: &mut ImageLoaderSettings| {
                            settings.sampler = ImageSampler::nearest();
                        },
                    )),
                    Node {
                        width: Px(180.0),
                        height: Px(180.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,

                        ..default()
                    },
                )]
            )
        ],
    ));
}

fn update(mut image: Single<&mut Node, With<FillBar>>, health: Single<&Health, With<Player>>) {
    let health = health.into_inner();
    let mut image = image.into_inner();
    let percent = health.get_percent();
    image.width = Px(180.0 * percent);
}

fn update_age(mut image: Single<&mut Transform, With<Pointer>>, aged: Single<&Aged, With<Player>>) {
    let aged = aged.into_inner();
    let mut image = image.into_inner();
    image.rotation = Quat::IDENTITY;
    image.rotate_z((aged.time as f32 / 50.0) * 3.1415);
}

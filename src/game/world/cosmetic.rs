use bevy::{
    ecs::{query, system::command},
    platform::collections::HashMap,
    prelude::*,
};
use bevy_ecs_ldtk::prelude::*;

use avian2d::prelude::*;
use bevy_light_2d::prelude::*;

use crate::{
    game::{
        age::Dead,
        enemies::Enemy,
        player::Player,
        ysort::{BACKGROUND_LAYER, ENTITY_LAYER, YSort},
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // app.add_systems(
    //     Update,
    //     spawn_wall_collision.run_if(in_state(Screen::WorldGen)),
    // );

    app.add_systems(OnEnter(Screen::Gameplay), (setup_sensor,setup_win));

    register_cosmetic_layer::<SimpleCosmeticBundleEntity>(app, "cosmetic");
    register_cosmetic_layer::<SimpleCosmeticBundleBackground>(app, "backgroundcosmetic");
    register_light_layer::<LightedBundleEntity>(app, "cosmetic");
    register_light_layer::<LightedBundleBackground>(app, "backgroundcosmetic");
    app.register_ldtk_entity_for_layer::<LightBundle>("cosmetic", "light");
    app.register_ldtk_entity_for_layer::<KillBundle>("functional", "death_rect");
    app.register_ldtk_entity_for_layer::<Gewinnbox>("functional", "rectregion");
}

fn setup_win(
    mut query2: Query<(&mut Transform, Entity), Added<GewinnDjinn>>,
    mut command: Commands,
) {
    for (mut transform, entity) in query2.iter_mut() {
        transform.scale = Vec3::ONE;
        command
            .entity(entity)
            .insert((Sensor, CollisionEventsEnabled))
            .observe(
                |trigger: Trigger<OnCollisionStart>,
                 player_query: Query<&Player>,
                 mut next_screen: ResMut<NextState<Screen>>| {
                    let other_entity = trigger.collider;
                    if player_query.contains(other_entity) {
                        next_screen.set(Screen::GameWin);
                    }
                },
            );
    }
}

fn setup_sensor(
    mut query: Query<(&mut Transform, Entity), Added<KillBill>>,

    mut command: Commands,
) {
    for (mut transform, entity) in query.iter_mut() {
        transform.scale = Vec3::ONE;
        command
            .entity(entity)
            .insert((Sensor, CollisionEventsEnabled))
            .observe(
                |trigger: Trigger<OnCollisionStart>,
                 player_query: Query<&Player>,
                 enemy_query: Query<&Enemy>,
                 mut commands: Commands| {
                    let other_entity = trigger.collider;
                    if player_query.contains(other_entity) || enemy_query.contains(other_entity) {
                        commands.entity(other_entity).insert(Dead);
                    }
                },
            );
    }
}

#[derive(Component, Clone, Default)]
struct GewinnDjinn;

#[derive(Component, Clone, Default)]
struct KillBill;

#[derive(Clone, Default, Bundle)]
struct Gewinnbox {
    friction: Friction,
    rigidbody: RigidBody,
    collider: Collider,
    marker: GewinnDjinn,
}

impl LdtkEntity for Gewinnbox {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        layer_instance: &LayerInstance,
        tileset: Option<&Handle<Image>>,
        tileset_definition: Option<&TilesetDefinition>,
        asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        Self {
            marker: Default::default(),
            friction: Friction::new(1.0),
            rigidbody: RigidBody::Static,
            collider: Collider::rectangle(
                (entity_instance.width) as f32,
                (entity_instance.height) as f32,
            ),
        }
    }
}

#[derive(Clone, Default, Bundle)]
struct KillBundle {
    friction: Friction,
    rigidbody: RigidBody,
    collider: Collider,
    marker: KillBill,
}

impl LdtkEntity for KillBundle {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        layer_instance: &LayerInstance,
        tileset: Option<&Handle<Image>>,
        tileset_definition: Option<&TilesetDefinition>,
        asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        KillBundle {
            marker: KillBill,
            friction: Friction::new(1.0),
            rigidbody: RigidBody::Static,
            collider: Collider::rectangle(
                (entity_instance.width) as f32,
                (entity_instance.height / 2) as f32,
            ),
        }
    }
}

fn register_cosmetic_layer<B>(app: &mut App, layer: &str)
where
    B: LdtkEntity + Bundle,
{
    for simplecosmetics in [
        "banner",
        "entrance",
        "window",
        "archway",
        "pillar",
        "statue_small",
        "standing_flag",
        "statue_big",
    ] {
        app.register_ldtk_entity_for_layer::<B>(layer, simplecosmetics);
    }
}

fn register_light_layer<B>(app: &mut App, layer: &str)
where
    B: LdtkEntity + Bundle,
{
    for lights in ["candles", "chandelier", "standing_light", "walllamp"] {
        app.register_ldtk_entity_for_layer::<B>(layer, lights);
    }
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
struct LightedBundleEntity {
    #[sprite_sheet]
    pub sprite_sheet: Sprite,
    #[ldtk_entity]
    pub light: LightBundle,
    #[with(construct_ysort_entity)]
    pub sort: YSort,
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
struct LightedBundleBackground {
    #[sprite_sheet]
    pub sprite_sheet: Sprite,
    #[ldtk_entity]
    pub light: LightBundle,
    #[with(construct_ysort_background)]
    pub sort: YSort,
}

fn construct_ysort_entity(entity_instance: &EntityInstance) -> YSort {
    YSort::new(ENTITY_LAYER, entity_instance.height as f32)
}

fn construct_ysort_background(entity_instance: &EntityInstance) -> YSort {
    YSort::new(BACKGROUND_LAYER, entity_instance.height as f32)
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
struct SimpleCosmeticBundleEntity {
    #[sprite_sheet]
    pub sprite_sheet: Sprite,
    #[with(construct_ysort_entity)]
    pub sort: YSort,
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
struct SimpleCosmeticBundleBackground {
    #[sprite_sheet]
    pub sprite_sheet: Sprite,
    #[with(construct_ysort_background)]
    pub sort: YSort,
}
//#[with(custom_constructor)]

#[derive(Clone, Default, Bundle)]
struct LightBundle {
    light: PointLight2d,
}

impl LdtkEntity for LightBundle {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        _layer_instance: &LayerInstance,
        tileset: Option<&Handle<Image>>,
        tileset_definition: Option<&TilesetDefinition>,
        asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        Self {
            light: PointLight2d {
                intensity: entity_instance
                    .get_float_field("intensity")
                    .expect("Expected range field on light")
                    .clone(),
                radius: entity_instance
                    .get_float_field("range")
                    .expect("Expected radius field on light")
                    .clone(),
                color: entity_instance
                    .get_color_field("color")
                    .expect("Expected radius field on light")
                    .clone(),
                ..Default::default()
            },
        }
    }
}

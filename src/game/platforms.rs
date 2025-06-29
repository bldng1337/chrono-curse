use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::{prelude::*, utils::ldtk_pixel_coords_to_translation_pivoted};

use crate::{AppSystems, asset_tracking::LoadResource, game::age::Timed, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlatformAssets>();
    app.add_systems(
        FixedUpdate,
        platform_update.run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        Update,
        platform_setup
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::WorldGen)),
    );
    app.register_ldtk_entity_for_layer::<PlatformBundle>("functional", "platform");
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlatformAssets {
    #[dependency]
    pub sprite: Handle<Image>,
    pub atlas: Handle<TextureAtlasLayout>,
}

impl FromWorld for PlatformAssets {
    fn from_world(world: &mut World) -> Self {
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let layout = TextureAtlasLayout::from_grid(UVec2::new(64, 64), 4, 1, None, None);
        let atlas = texture_atlas_layouts.add(layout);
        let assets = world.resource::<AssetServer>();
        Self {
            sprite: assets.load("sprites/tileset/paltform.png"),
            atlas,
        }
    }
}

#[derive(Clone, Default, Component)]
struct Platform {
    width: i32,
    speed: f32,
}

impl LdtkEntity for Platform {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        layer_instance: &LayerInstance,
        _: Option<&Handle<Image>>,
        _: Option<&TilesetDefinition>,
        _: &AssetServer,
        _: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        Self {
            width: entity_instance.width,
            speed: entity_instance
                .get_float_field("speed")
                .expect("Platform should have a speed field")
                .clone(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Default, Component, Reflect, Debug)]
struct Points {
    points: Vec<Vec2>,
    current: i32,
    mirror: bool,
    dir: bool,
}

impl Points {
    fn get_target(&self) -> Vec2 {
        self.points[self.current as usize]
    }

    fn update_target(&mut self, currpos: Vec2) {
        if self.get_target().distance_squared(currpos) < 30.0 {
            self.next_point();
        }
    }

    fn next_point(&mut self) {
        if self.dir {
            self.current += 1;
        } else {
            self.current -= 1;
        }
        if self.current >= self.points.len() as i32 || self.current < 0 {
            if self.mirror {
                self.dir = !self.dir;
                self.next_point();
            } else {
                self.current = 0;
            }
        }
    }
}

impl LdtkEntity for Points {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        layer_instance: &LayerInstance,
        _: Option<&Handle<Image>>,
        _: Option<&TilesetDefinition>,
        _: &AssetServer,
        _: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        println!("Spawning Platform");
        let mut points = Vec::new();
        points.push(ldtk_pixel_coords_to_translation_pivoted(
            entity_instance.px,
            layer_instance.c_hei * layer_instance.grid_size,
            IVec2::new(entity_instance.width, entity_instance.height),
            entity_instance.pivot,
        ));

        let ldtk_patrol_points = entity_instance
            .iter_points_field("points")
            .expect("points field should be correclty typed");

        for ldtk_point in ldtk_patrol_points {
            let pixel_coords = (ldtk_point.as_vec2() + Vec2::new(-1.0, 1.0))
                * Vec2::splat(layer_instance.grid_size as f32);

            points.push(ldtk_pixel_coords_to_translation_pivoted(
                pixel_coords.as_ivec2(),
                layer_instance.c_hei * layer_instance.grid_size,
                IVec2::new(entity_instance.width, entity_instance.height),
                entity_instance.pivot, //entity_instance.pivot,
            ));
        }

        Self {
            points,
            current: 0,
            mirror: entity_instance
                .get_bool_field("mirror")
                .expect("Platform should have mirror field")
                .clone(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
struct PlatformBundle {
    #[ldtk_entity]
    points: Points,
    #[ldtk_entity]
    platform: Platform,
}

fn platform_setup(
    mut platforms: Query<(Entity, &Platform, &mut Transform), Added<Platform>>,
    mut commands: Commands,
    sprite: Res<PlatformAssets>,
) {
    for (entity, platform, mut transform) in platforms.iter_mut() {
        transform.scale = Vec3::ONE; //this is scuffed
        transform.translation.z = 2.0;
        let width = platform.width / 64;
        let Ok(mut command) = commands.get_entity(entity) else {
            continue;
        };
        command
            .insert(Collider::rectangle(platform.width as f32, 32.0))
            .insert(RigidBody::Kinematic)
            .insert(Friction::new(1.0))
            .insert(Timed::default());

        let startoffset = -platform.width / 2 + 32;
        for i in 0..width {
            // Texture
            let xoffset = startoffset + i * 64;
            let index = match i {
                0 => 0,
                x if x == width - 1 => 3,
                x => 1 + x % 2, //TODO: Random between 1-2
            } as usize;
            let texture_atlas = TextureAtlas {
                layout: sprite.atlas.clone(),
                index,
            };
            let child = commands
                .spawn((
                    Transform::from_xyz(xoffset as f32, 0.0, 0.0),
                    Sprite {
                        image: sprite.sprite.clone(),
                        texture_atlas: Some(texture_atlas),
                        ..Default::default()
                    },
                ))
                .id(); //TODO: Maybe fix entity despawn if platform is invalid which shouldnt happen anyway
            let Ok(mut command) = commands.get_entity(entity) else {
                continue;
            };
            command.add_child(child);
        }
    }
}

fn platform_update(
    mut platforms: Query<(&mut Points, &mut LinearVelocity, &Transform, &Platform)>,
) {
    for (mut points, mut linvel, transform, platform) in platforms.iter_mut() {
        let currentpos = transform.translation.xy();
        points.update_target(currentpos);
        let nextpoint = points.get_target();
        let delta = nextpoint - currentpos;
        if delta.length() < 1.0 {
            continue;
        }
        let velocity = delta.normalize() * Vec2::splat(50.0 * platform.speed);
        linvel.0 = velocity;
    }
}

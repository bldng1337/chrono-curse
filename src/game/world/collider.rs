use std::collections::HashSet;

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_ecs_ldtk::prelude::*;

use avian2d::prelude::*;

use crate::{game::worldgen::LevelAssets, screens::Screen};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
enum Material {
    #[default]
    Stone,
    Wood,
    Spikes,
    Other,
}
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall {
    mat: Material,
}
impl LdtkIntCell for Wall {
    fn bundle_int_cell(int_grid_cell: IntGridCell, _layer_instance: &LayerInstance) -> Self {
        Self {
            mat: match int_grid_cell.value {
                1 => Material::Stone,
                2 => Material::Wood,
                3 => Material::Spikes,
                _ => Material::Other,
            },
        }
    }
}
#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    #[ldtk_int_cell]
    wall: Wall,
}

/// Spawns heron collisions for the walls of a level
///
/// You could just insert a ColliderBundle into the WallBundle,
/// but this spawns a different collider for EVERY wall tile.
/// This approach leads to bad performance.
///
/// Instead, by flagging the wall tiles and spawning the collisions later,
/// we can minimize the amount of colliding entities.
///
/// The algorithm used here is a nice compromise between simplicity, speed,
/// and a small number of rectangle colliders.
/// In basic terms, it will:
/// 1. consider where the walls are
/// 2. combine wall tiles into flat "plates" in each individual row
/// 3. combine the plates into rectangles across multiple rows wherever possible
/// 4. spawn colliders for each rectangle
pub fn spawn_wall_collision(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &ChildOf, &Wall), Added<Wall>>,
    parent_query: Query<&ChildOf, Without<Wall>>,
    level_query: Query<(Entity, &LevelIid)>,
    level_assets: Res<LevelAssets>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) {
    /// Represents a wide wall that is 1 tile tall
    /// Used to spawn wall collisions
    #[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: i32,
        right: i32,
        half_height: bool,
        left_half: bool,
        right_half: bool,
    }

    /// A simple rectangle type representing a wall of any size
    struct Rect {
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
    }

    // Consider where the walls are
    // storing them as GridCoords in a HashSet for quick, easy lookup
    //
    // The key of this map will be the entity of the level the wall belongs to.
    // This has two consequences in the resulting collision entities:
    // 1. it forces the walls to be split along level boundaries
    // 2. it lets us easily add the collision entities as children of the appropriate level entity
    let mut level_to_wall_locations: HashMap<Entity, HashMap<GridCoords, Material>> =
        HashMap::new();

    wall_query
        .iter()
        .for_each(|(&grid_coords, child_of, wall)| {
            // An intgrid tile's direct parent will be a layer entity, not the level entity
            // To get the level entity, you need the tile's grandparent.
            // This is where parent_query comes in.
            if let Ok(parent_child_of) = parent_query.get(child_of.parent()) {
                level_to_wall_locations
                    .entry(parent_child_of.parent())
                    .or_default()
                    .insert(grid_coords, wall.mat);
            }
        });

    if !wall_query.is_empty() {
        level_query.iter().for_each(|(level_entity, level_iid)| {
            if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
                let ldtk_project = ldtk_project_assets
                    .get(level_assets.worlddata.id())
                    .expect("Project should be loaded if level has spawned");

                let level = ldtk_project
                    .as_standalone()
                    .get_loaded_level_by_iid(&level_iid.to_string())
                    .expect("Spawned level should exist in LDtk project");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level.layer_instances()[0];

                // combine wall tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;
                    let mut half_size = None;
                    let mut lastmat = None;
                    let mut startmat = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right edge
                    for x in 0..width + 1 {
                        match (
                            plate_start,
                            half_size,
                            level_walls.get(&GridCoords { x, y }),
                            level_walls.contains_key(&GridCoords { x, y: y + 1 }),
                        ) {
                            //Start a Rect
                            (None, None, Some(currmat), above) => {
                                plate_start = Some(x);
                                lastmat = Some(currmat);
                                startmat = Some(currmat);
                                half_size = Some(!above);
                            }
                            (Some(_), Some(true), Some(currmat), false)
                            | (Some(_), Some(false), Some(currmat), true) => {
                                lastmat = Some(currmat);
                            }
                            (Some(s), Some(false), Some(currmat), false) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                    half_height: false,
                                    left_half: startmat == Some(&Material::Wood),
                                    right_half: startmat == Some(&Material::Wood),
                                });
                                plate_start = Some(x);
                                lastmat = Some(currmat);
                                startmat = Some(currmat);
                                half_size = Some(true);
                            }
                            (Some(s), Some(true), Some(currmat), true) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                    half_height: true,
                                    left_half: startmat == Some(&Material::Wood),
                                    right_half: lastmat == Some(&Material::Wood),
                                });
                                plate_start = Some(x);
                                lastmat = Some(currmat);
                                startmat = Some(currmat);
                                half_size = Some(false);
                            }
                            (Some(s), Some(halfsize), None, _) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                    half_height: halfsize,
                                    left_half: startmat == Some(&Material::Wood),
                                    right_half: lastmat == Some(&Material::Wood),
                                });
                                plate_start = None;
                                half_size = None;
                            }
                            _ => (),
                        }
                    }

                    plate_stack.push(row_plates);
                }

                // combine "plates" into rectangles across multiple rows
                let mut rect_builder: HashMap<Plate, Rect> = HashMap::new();
                let mut prev_row: Vec<Plate> = Vec::new();
                let mut wall_rects: Vec<Rect> = Vec::new();

                // an extra empty row so the algorithm "finishes" the rects that touch the top edge
                plate_stack.push(Vec::new());

                for (y, current_row) in plate_stack.into_iter().enumerate() {
                    for prev_plate in &prev_row {
                        if !current_row.contains(prev_plate) {
                            // remove the finished rect so that the same plate in the future starts a new rect
                            if let Some(rect) = rect_builder.remove(prev_plate) {
                                wall_rects.push(rect);
                            }
                        }
                    }
                    for plate in &current_row {
                        rect_builder
                            .entry(plate.clone())
                            .and_modify(|e| e.top += 1.0)
                            .or_insert(Rect {
                                bottom: y as f32,
                                top: if plate.half_height {
                                    y as f32 - 0.5
                                } else {
                                    y as f32
                                },
                                left: if plate.left_half {
                                    plate.left as f32 + 0.5
                                } else {
                                    plate.left as f32
                                },
                                right: if plate.right_half {
                                    plate.right as f32 - 0.5
                                } else {
                                    plate.right as f32
                                },
                            });
                    }
                    prev_row = current_row;
                }

                commands.entity(level_entity).with_children(|level| {
                    // Spawn colliders for every rectangle..
                    // Making the collider a child of the level serves two purposes:
                    // 1. Adjusts the transforms to be relative to the level for free
                    // 2. the colliders will be despawned automatically when levels unload
                    for wall_rect in wall_rects {
                        level
                            .spawn_empty()
                            .insert(Collider::rectangle(
                                (wall_rect.right as f32 - wall_rect.left as f32 + 1.)
                                    * grid_size as f32,
                                (wall_rect.top as f32 - wall_rect.bottom as f32 + 1.)
                                    * grid_size as f32,
                            ))
                            .insert(RigidBody::Static)
                            .insert(Friction::new(1.0))
                            .insert(Transform::from_xyz(
                                (wall_rect.left + wall_rect.right + 1.0) as f32 * grid_size as f32
                                    / 2.,
                                (wall_rect.bottom + wall_rect.top + 1.0) as f32 * grid_size as f32
                                    / 2.,
                                0.,
                            ))
                            .insert(GlobalTransform::default());
                    }
                });
            }
        });
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        spawn_wall_collision.run_if(in_state(Screen::WorldGen).or(in_state(Screen::Gameplay))),
    );
    app.register_ldtk_int_cell_for_layer::<WallBundle>("collider", 1);
    app.register_ldtk_int_cell_for_layer::<WallBundle>("collider", 2);
    app.register_ldtk_int_cell_for_layer::<WallBundle>("collider", 3);
}

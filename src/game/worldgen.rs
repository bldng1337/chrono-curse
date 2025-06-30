use bevy::{
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
};
use bevy_ecs_ldtk::{ldtk::Level, prelude::*};
use rand::prelude::*;

use crate::{AppSystems, asset_tracking::LoadResource, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
    app.init_resource::<WorldGen>();
    app.add_systems(OnEnter(Screen::WorldGen), init_world_gen);
    app.add_systems(OnEnter(Screen::Gameplay), cleanup);
    app.add_systems(
        Update,
        (world_gen, tick_timer)
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::WorldGen)),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    pub worlddata: Handle<LdtkProject>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            worlddata: assets.load("map/world.ldtk"),
        }
    }
}
#[derive(Component)]
struct Room {
    bb: Aabb2d,
}

#[derive(Resource, Default)]
pub struct WorldGen {
    rooms: Vec<RoomRef>,
    doors: Vec<Aabb2d>,
    time: Timer,
}

impl WorldGen {
    pub fn is_finished(&self) -> bool {
        self.doors.is_empty() && !self.rooms.is_empty() && self.time.finished()
    }
}

trait Sizeable {
    fn get_size(&self) -> Vec2;
}

impl Sizeable for Aabb2d {
    fn get_size(&self) -> Vec2 {
        (self.max - self.min).abs()
    }
}

struct RoomRef {
    bb: Aabb2d,
    levelid: String,
    difficulty: i32,
    doors: Vec<Aabb2d>,
    doorsizes: Vec<Vec2>,
}

impl RoomRef {
    fn has_door(&self, size: &Vec2) -> bool {
        self.doorsizes.contains(size)
    }

    fn get_translations(&self, door: &Aabb2d) -> impl Iterator<Item = (Vec2, &Aabb2d)> {
        self.doors
            .iter()
            .filter(|x| x.get_size() == door.get_size())
            .map(|x| (door.min - x.min, x))
    }

    fn spawn(&self, translation: Vec2, ldtk_handle: &Handle<LdtkProject>) -> impl Bundle {
        (
            Name::new("LdtkLevel"),
            LdtkWorldBundle {
                ldtk_handle: LdtkProjectHandle {
                    handle: ldtk_handle.clone(),
                },
                level_set: LevelSet::from_iids([self.levelid.clone()]),
                transform: Transform {
                    translation: Vec3::new(translation.x, translation.y, 0.0),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            Room {
                bb: self.bb.translated_by(translation),
            },
        )
    }
}

impl From<&Level> for RoomRef {
    fn from(value: &Level) -> Self {
        let width = value.px_wid;
        let height = value.px_hei;

        let bb = Aabb2d {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(width as f32, height as f32),
        };
        let layers = &value.layer_instances.as_ref().unwrap();
        let door_layer = &layers.iter().find(|x| x.identifier == "functional");
        let door_layer = door_layer.as_ref().unwrap();
        let doors: Vec<_> = door_layer
            .entity_instances
            .iter()
            .filter(|ent| ent.identifier == "door")
            .map(|ent| {
                let x = ent.px.x as f32;
                let y = (height - ent.px.y) as f32;
                return Aabb2d {
                    min: Vec2::new(x, y),
                    max: Vec2::new(x + ent.width as f32, y + ent.height as f32),
                };
            })
            // .filter(|x| {
            //     let min_inside = x.min.max(bb.min).min(bb.max) == x.min;
            //     let max_inside = x.max.max(bb.min).min(bb.max) == x.max;
            //     println!("ent {} {}  bb {} {} min {} max {}",x.min,x.max,bb.min,bb.max,min_inside,max_inside);
            //     min_inside != max_inside
            // })
            .collect();
        let doorsizes: Vec<_> = doors.iter().map(|door| door.get_size()).collect();
        Self {
            bb,
            doors,
            doorsizes,
            levelid: value.iid.clone(),
            difficulty: match value.field_instances.get(0).unwrap().value {
                FieldValue::Int(Some(num)) => num,
                _ => 1,
            },
        }
    }
}

fn cleanup(rooms: Query<(&Room, Entity)>, mut commands: Commands) {
    for (room, entity) in rooms.iter() {
        commands.entity(entity).remove::<Room>();
    }
}

fn world_gen(
    rooms: Query<&Room>,
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    mut worldgen: ResMut<WorldGen>,
) {
    if worldgen.doors.is_empty() {
        return;
    }
    let mut doors: Vec<Aabb2d> = Vec::new();
    let mut rng = thread_rng();
    for door in worldgen.doors.clone() {
        let dist = door.center().length_squared();
        worldgen.rooms.shuffle(&mut rng);
        'room: for roomref in &worldgen.rooms {
            let lowdiff_score = ((roomref.difficulty - 1) as f32) * 6000.0;
            let upperdiff_score = ((roomref.difficulty + 1) as f32) * 6000.0;
            if !((lowdiff_score * lowdiff_score)..(upperdiff_score * upperdiff_score))
                .contains(&dist)
            {
                continue;
            }
            if !roomref.has_door(&door.get_size()) {
                continue;
            }
            for (translation, currdoor) in roomref.get_translations(&door) {
                let bb = roomref.bb.translated_by(translation).shrink(Vec2::ONE);
                let mut isok = true;
                for room in rooms.iter() {
                    if bb.intersects(&room.bb) {
                        isok = false;
                        break;
                    }
                }
                if !isok {
                    continue;
                }
                for opendoor in roomref
                    .doors
                    .iter()
                    .filter(|x| **x != *currdoor)
                    .map(|x| x.translated_by(translation))
                {
                    doors.push(opendoor);
                }
                let translation = translation.clone();
                commands
                    .spawn(roomref.spawn(translation, &level_assets.worlddata))
                    .insert(StateScoped(Screen::Gameplay));
                break 'room;
            }
        }
    }
    let len = doors.len();
    worldgen.doors = doors;
}

fn tick_timer(mut world: ResMut<WorldGen>, time: Res<Time>) {
    if world.doors.is_empty() {
        world.time.tick(time.delta());
    } else {
        world.time.reset();
    }
}

fn init_world_gen(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    ldtkproj: Res<Assets<LdtkProject>>,
) {
    let ldtk: &Handle<LdtkProject> = &level_assets.worlddata;
    let proj: &LdtkProject = ldtkproj.get(ldtk.id()).unwrap();

    let rooms: Vec<RoomRef> = proj
        .data()
        .json_data()
        .levels
        .iter()
        .map(|x| x.into())
        .collect();
    // let mut rng = thread_rng();
    // rooms.shuffle(&mut rng);
    let first_room = &rooms[0];
    commands
        .spawn(first_room.spawn(Vec2::ZERO, ldtk))
        .insert(StateScoped(Screen::Gameplay));
    commands.insert_resource(WorldGen {
        doors: first_room.doors.clone(),
        rooms,
        time: Timer::from_seconds(0.3, TimerMode::Once),
    });
}

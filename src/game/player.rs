use avian2d::prelude::{
    Collider, ColliderAabb, LinearVelocity, LockedAxes, RigidBody, SpatialQuery,
};
use bevy::{
    core_pipeline::{
        bloom::Bloom,
        tonemapping::{DebandDither, Tonemapping},
    },
    prelude::*,
};
use bevy_ecs_ldtk::{LdtkEntity, app::LdtkEntityAppExt};
use bevy_light_2d::light::AmbientLight2d;
use bevy_tnua::{
    TnuaAction, TnuaAnimatingState,
    builtins::{TnuaBuiltinDash, TnuaBuiltinJumpState},
    control_helpers::TnuaSimpleAirActionsCounter,
    prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController},
};
use bevy_tnua_avian2d::TnuaAvian2dSensorShape;
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    AgedSystems, AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    audio::music,
    game::{
        age::{Age, Aged, Dead, Timed},
        animate::{AnimationConfig, Directional},
        enemies::Enemy,
        health::Health,
        ysort::{ENTITY_LAYER, YSort},
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // app.register_type::<LevelAssets>();
    app.load_resource::<PlayerAssets>();
    // app.init_resource::<WorldGen>();
    // app.add_systems(OnEnter(Screen::WorldGen), init_world_gen);
    app.add_systems(
        Update,
        handle_animating
            .in_set(AppSystems::Update)
            .in_set(AgedSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        Update,
        (drop, pickup, init_item)
            .in_set(AppSystems::Update)
            .in_set(AgedSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        Update,
        turn_book
            .in_set(AppSystems::Update)
            .in_set(AgedSystems)
            .run_if(in_state(Screen::Gameplay)),
    );

    app.add_systems(OnEnter(Screen::Gameplay), init_player);
    app.register_ldtk_entity::<PlayerSpawn>("player");
}

#[derive(Debug)]
pub enum AnimationState {
    Standing,
    Running(f32),
    Dashing,
    Jumping,
    Falling,
}

fn spawn_item(current_items: &Vec<Item>, assets: &PlayerAssets) -> Option<impl Bundle> {
    let mut items: Vec<_> = [
        "Immortal Flame",
        "Book of Fire",
        "Book of Current",
        "Duplex",
        "Solaces Cradle",
        "Doomsayer",
    ]
    .iter()
    .map(|a| a.to_string())
    .filter(|a| !current_items.iter().any(|b| b.name == *a))
    .collect();
    let mut stats = vec![
        "Quick Casting".to_string(),
        "Accelerate Magic".to_string(),
        "Basics of Magic".to_string(),
        "Intermediate Magic".to_string(),
        "Advanced Magic".to_string(),
    ];
    items.append(&mut stats);
    let mut elemental_tomes = vec!["Book of Fire".to_string(), "Book of Current".to_string()];
    if !current_items
        .iter()
        .any(|item| elemental_tomes.contains(&item.name))
    {
        items.append(&mut elemental_tomes);
    }
    let mut rng = thread_rng();
    items.shuffle(&mut rng);
    let size = 32.0;
    match items.get(0).map(|a| a.as_str()) {
        Some("Immortal Flame") => {
            return Some((
                Name::new("Immortal Flame"),
                Item {
                    name: "Immortal Flame".to_string(),
                    strength: 1.0,
                    speed: 0.0,
                },
                Sprite {
                    image: assets.sprite_book_red.clone(),
                    custom_size: Some(Vec2::splat(size)),
                    ..Default::default()
                },
            ));
        }
        Some("Book of Fire") => {
            return Some((
                Name::new("Book of Fire"),
                Item {
                    name: "Book of Fire".to_string(),
                    strength: 0.25,
                    speed: 0.0,
                },
                Sprite {
                    image: assets.sprite_book_red.clone(),
                    custom_size: Some(Vec2::splat(size)),
                    ..Default::default()
                },
            ));
        }
        Some("Book of Current") => {
            return Some((
                Name::new("Book of Current"),
                Item {
                    name: "Book of Current".to_string(),
                    strength: 0.0,
                    speed: 0.25,
                },
                Sprite {
                    image: assets.sprite_book_blue.clone(),
                    custom_size: Some(Vec2::splat(size)),
                    ..Default::default()
                },
            ));
        }
        Some("Duplex") => {
            return Some((
                Name::new("Duplex"),
                Item {
                    name: "Duplex".to_string(),
                    strength: 0.15,
                    speed: 0.15,
                },
                Sprite {
                    image: assets.sprite_book_blue.clone(),
                    custom_size: Some(Vec2::splat(size)),
                    ..Default::default()
                },
            ));
        }
        Some("Cradle of Solace") => {
            return Some((
                Name::new("Solaces Cradle"),
                Item {
                    name: "Solaces Cradle".to_string(),
                    strength: 0.0,
                    speed: 0.0,
                },
                Sprite {
                    image: assets.sprite_book_gold.clone(),
                    custom_size: Some(Vec2::splat(size)),
                    ..Default::default()
                },
            ));
        }
        Some("Quick Casting") => {
            return Some((
                Name::new("Quick Casting"),
                Item {
                    name: "Quick Casting".to_string(),
                    strength: 0.0,
                    speed: 0.1,
                },
                Sprite {
                    image: assets.sprite_book_gold.clone(),
                    custom_size: Some(Vec2::splat(size)),
                    ..Default::default()
                },
            ));
        }
        Some("Accelerate Magic") => {
            return Some((
                Name::new("Accelerate Magic"),
                Item {
                    name: "Accelerate Magic".to_string(),
                    strength: 0.0,
                    speed: 0.2,
                },
                Sprite {
                    image: assets.sprite_book_gold.clone(),
                    custom_size: Some(Vec2::splat(size)),
                    ..Default::default()
                },
            ));
        }
        Some("Basics of Magic") => {
            return Some((
                Name::new("Basics of Magic"),
                Item {
                    name: "Basics of Magic".to_string(),
                    strength: 0.1,
                    speed: 0.0,
                },
                Sprite {
                    image: assets.sprite_book_blue.clone(),
                    custom_size: Some(Vec2::splat(size)),
                    ..Default::default()
                },
            ));
        }
        Some("Intermediate Magic") => {
            return Some((
                Name::new("Intermediate Magic"),
                Item {
                    name: "Intermediate Magic".to_string(),
                    strength: 0.2,
                    speed: 0.0,
                },
                Sprite {
                    image: assets.sprite_book_blue.clone(),
                    custom_size: Some(Vec2::splat(size)),
                    ..Default::default()
                },
            ));
        }
        Some("Advanced Magic") => {
            return Some((
                Name::new("Advanced Magic"),
                Item {
                    name: "Advanced Magic".to_string(),
                    strength: 0.3,
                    speed: 0.0,
                },
                Sprite {
                    image: assets.sprite_book_blue.clone(),
                    custom_size: Some(Vec2::splat(size)),
                    ..Default::default()
                },
            ));
        }
        _ => {
            return None;
        }
    }
}

#[derive(Clone, Default, Component)]
pub struct Item {
    strength: f64,
    speed: f64,
    pub(crate) name: String,
}

#[derive(Clone, Component)]
pub struct SpellCap {
    pub strength: f64,
    pub speed: f64,
    pub items: Vec<Item>,
    pub timer: Timer,
}

impl Default for SpellCap {
    fn default() -> Self {
        Self {
            strength: 1.0,
            speed: 1.0,
            items: Default::default(),
            timer: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }
}

impl SpellCap {
    fn add_item(&mut self, item: Item) {
        self.items.push(item);
        self.strength = self
            .items
            .iter()
            .map(|a| a.strength)
            .reduce(|a, b| a + b)
            .unwrap_or_default()
            + 1.0;
        self.speed = self
            .items
            .iter()
            .map(|a| a.speed)
            .reduce(|a, b| a + b)
            .unwrap_or_default()
            + 1.0;
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    //Spells
    pub sprite_book_red: Handle<Image>,
    pub sprite_book_gold: Handle<Image>,
    pub sprite_book_blue: Handle<Image>,

    //Young
    #[dependency]
    pub ysprite_idle: Handle<Image>,
    pub yatlas_idle: Handle<TextureAtlasLayout>,

    #[dependency]
    pub ysprite_run: Handle<Image>,
    pub yatlas_run: Handle<TextureAtlasLayout>,

    #[dependency]
    pub ysprite_jump: Handle<Image>,
    pub yatlas_jump: Handle<TextureAtlasLayout>,

    //Ancient
    #[dependency]
    pub asprite_idle: Handle<Image>,
    pub aatlas_idle: Handle<TextureAtlasLayout>,

    #[dependency]
    pub asprite_run: Handle<Image>,
    pub aatlas_run: Handle<TextureAtlasLayout>,

    #[dependency]
    pub asprite_jump: Handle<Image>,
    pub aatlas_jump: Handle<TextureAtlasLayout>,

    //Old
    #[dependency]
    pub osprite_idle: Handle<Image>,
    pub oatlas_idle: Handle<TextureAtlasLayout>,

    #[dependency]
    pub osprite_run: Handle<Image>,
    pub oatlas_run: Handle<TextureAtlasLayout>,

    #[dependency]
    pub osprite_jump: Handle<Image>,
    pub oatlas_jump: Handle<TextureAtlasLayout>,

    #[dependency]
    pub sprite_book: Handle<Image>,
    pub atlas_book: Handle<TextureAtlasLayout>,

    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let layout = TextureAtlasLayout::from_grid(UVec2::new(704, 704), 3, 1, None, None);
        let atlas_idle = texture_atlas_layouts.add(layout);

        let layout = TextureAtlasLayout::from_grid(UVec2::new(320, 320), 6, 1, None, None);
        let atlas_run: Handle<TextureAtlasLayout> = texture_atlas_layouts.add(layout);

        let layout = TextureAtlasLayout::from_grid(UVec2::new(320, 320), 8, 1, None, None);
        let atlas_jump = texture_atlas_layouts.add(layout);

        let layout = TextureAtlasLayout::from_grid(UVec2::new(280, 280), 8, 1, None, None);
        let atlas_book = texture_atlas_layouts.add(layout);

        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/levelOST/combined_track.ogg"),

            ysprite_idle: assets.load("sprites/entities/player/young/idle.png"),
            ysprite_run: assets.load("sprites/entities/player/young/run.png"),
            ysprite_jump: assets.load("sprites/entities/player/young/jump.png"),

            osprite_idle: assets.load("sprites/entities/player/old/idle.png"),
            osprite_run: assets.load("sprites/entities/player/old/run.png"),
            osprite_jump: assets.load("sprites/entities/player/old/jump.png"),

            asprite_idle: assets.load("sprites/entities/player/ancient/idle.png"),
            asprite_run: assets.load("sprites/entities/player/ancient/run.png"),
            asprite_jump: assets.load("sprites/entities/player/ancient/jump.png"),

            sprite_book: assets.load("sprites/entities/player/magicbook.png"),

            sprite_book_red: assets.load("UIElements/Book_1.png"),
            sprite_book_gold: assets.load("UIElements/Book_2.png"),
            sprite_book_blue: assets.load("UIElements/Book_3.png"),

            aatlas_idle: atlas_idle.clone(),
            aatlas_run: atlas_run.clone(),
            aatlas_jump: atlas_jump.clone(),

            oatlas_idle: atlas_idle.clone(),
            oatlas_run: atlas_run.clone(),
            oatlas_jump: atlas_jump.clone(),

            yatlas_idle: atlas_idle,
            yatlas_run: atlas_run,
            yatlas_jump: atlas_jump,

            atlas_book,
        }
    }
}

#[derive(Clone, Default, Component)]
pub struct Player {
    pub dashtimer: Timer,
}

#[derive(Clone, Component)]
pub struct ItemText(Entity);

#[derive(Clone, Default, Component)]
pub struct Book;

fn init_item(items: Query<(Entity, &Item), Added<Item>>, mut commands: Commands) {
    for (entity, item) in items {
        commands.entity(entity).with_child((
            Transform::from_xyz(0.0, 22.0, 100.0),
            Text2d::new(item.name.clone()),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            ItemText(entity),
        ));
    }
}

#[derive(Clone, Default, Component)]
pub struct NoDrops;

fn pickup(
    mut player: Single<(Entity, &mut SpellCap), With<Player>>,
    items: Query<(Entity, &Item, &GlobalTransform, &Children)>,
    spatial_query: SpatialQuery,
    item_texts: Query<(Entity, &ItemText)>,
    mut commands: Commands,
) {
    let (player, mut inv) = player.into_inner();
    for (entity, item, transform, children) in items.iter() {
        let pos = transform.translation().xy();
        let size = 40.0;
        let aabb = ColliderAabb::from_min_max(pos - (size / 2.0), pos + (size / 2.0));
        let got_hit = spatial_query.aabb_intersections_with_aabb(aabb);
        if got_hit.contains(&player) {
            for (textent, text) in item_texts {
                if text.0 == entity {
                    commands.entity(textent).despawn();
                }
            }
            commands.entity(entity).despawn();
            inv.add_item(item.clone());
        }
    }
}

fn drop(
    spawn: Query<(&GlobalTransform, Entity), (Added<Dead>, Without<NoDrops>, With<Enemy>)>,
    mut commands: Commands,
    items: Single<&SpellCap>,
    playerassets: Res<PlayerAssets>,
) {
    let items = items.into_inner();
    for (transform, entity) in spawn.iter() {
        commands.entity(entity).insert(NoDrops);
        if let Some(item) = spawn_item(&items.items, &playerassets) {
            commands.spawn(item).insert((
                StateScoped(Screen::Gameplay),
                Transform::from_translation(transform.translation()),
                Timed::default(),
                RigidBody::Dynamic,
                YSort::new(ENTITY_LAYER, 32.0),
                Collider::circle(16.0),
                LockedAxes::ROTATION_LOCKED,
            ));
        }
    }
}

fn init_player(
    mut commands: Commands,
    spawn: Query<&Transform, With<PlayerSpawn>>,
    cameras: Query<Entity, With<Camera2d>>,
    playerassets: Res<PlayerAssets>,
) {
    for entity in cameras {
        commands.entity(entity).despawn();
    }
    let mut transform = spawn.iter().next().unwrap().clone();
    transform.translation.y += 32.0;
    transform.translation.z = 3.0;
    let texture_atlas = TextureAtlas {
        layout: playerassets.yatlas_idle.clone(),
        index: 0,
    };
    let texture_atlas_book = TextureAtlas {
        layout: playerassets.atlas_book.clone(),
        index: 0,
    };
    commands
        .spawn((
            StateScoped(Screen::Gameplay),
            //Player
            transform,
            Health::new(100.0),
            YSort::new(ENTITY_LAYER, 64.0),
            // The player character needs to be configured as a dynamic rigid body of the physics
            // engine.
            RigidBody::Dynamic,
            Collider::capsule(16.0, 32.0),
            TnuaAnimatingState::<AnimationState>::default(),
            // This is Tnua's interface component.
            TnuaController::default(),
            Player {
                dashtimer: Timer::from_seconds(1.5, TimerMode::Once),
            },
            // A sensor shape is not strictly necessary, but without it we'll get weird results.
            TnuaAvian2dSensorShape(Collider::rectangle(31.0, 0.0)),
            // Tnua can fix the rotation, but the character will still get rotated before it can do so.
            // By locking the rotation we can prevent this.
            LockedAxes::ROTATION_LOCKED,
            Name::new("Player"),
            Sprite {
                image: playerassets.ysprite_idle.clone(),
                texture_atlas: Some(texture_atlas.clone()),
                custom_size: Some(Vec2::new(100.0, 70.0)),
                ..Default::default()
            },
            AnimationConfig::new(
                0,
                2,
                8,
                true,
                true,
                Some(texture_atlas),
                playerassets.ysprite_idle.clone(),
            ),
            Directional {
                flipdir: true,
                ..Default::default()
            },
        ))
        .insert((
            Timed::default(),
            Aged::default(),
            SpellCap::default(),
            music(playerassets.music.clone()),
            TnuaSimpleAirActionsCounter::default(),
        ))
        .with_child((
            //Book
            Transform::from_xyz(35.0, 10.0, 1.0),
            Book,
            Sprite {
                image: playerassets.sprite_book.clone(),
                texture_atlas: Some(texture_atlas_book.clone()),
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..Default::default()
            },
            AnimationConfig::new(
                0,
                7,
                8,
                true,
                true,
                Some(texture_atlas_book),
                playerassets.sprite_book.clone(),
            ),
        ))
        .with_child((
            //Camera
            Camera2d,
            Camera {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            AmbientLight2d {
                brightness: 0.45,
                ..default()
            },
            Tonemapping::TonyMcMapface,
            Bloom::default(),
            DebandDither::Enabled,
        ));
}

#[derive(Clone, Default, Component, LdtkEntity)]
pub struct PlayerSpawn {}

fn turn_book(
    player_query: Query<(&LinearVelocity,), With<Player>>,
    mut book_query: Query<(&mut Transform, &mut Sprite), With<Book>>,
) {
    let Ok((vel,)) = player_query.single() else {
        return;
    };
    let Ok((mut transform, mut sprite)) = book_query.single_mut() else {
        return;
    };
    if vel.x.abs() < 100.0 {
        return;
    }
    let dir = vel.x.signum();
    transform.translation.x = 35.0 * dir;
    sprite.flip_x = dir > 0.0;
}

fn handle_animating(
    mut player_query: Query<
        (
            &TnuaController,
            &mut TnuaAnimatingState<AnimationState>,
            &mut Sprite,
            &mut AnimationConfig,
            &Aged,
        ),
        With<Player>,
    >,
    playerassets: Res<PlayerAssets>,
) {
    let Ok((controller, mut animating_state, mut sprite, mut animation, aged)) =
        player_query.single_mut()
    else {
        return;
    };
    let current_status_for_animating = match controller.action_name() {
        // Unless you provide the action names yourself, prefer matching against the `NAME` const
        // of the `TnuaAction` trait. Once `type_name` is stabilized as `const` Tnua will use it to
        // generate these names automatically, which may result in a change to the name.
        Some(TnuaBuiltinJump::NAME) => {
            // In case of jump, we want to cast it so that we can get the concrete jump state.
            let (_, jump_state) = controller
                .concrete_action::<TnuaBuiltinJump>()
                .expect("action name mismatch");
            // Depending on the state of the jump, we need to decide if we want to play the jump
            // animation or the fall animation.
            match jump_state {
                TnuaBuiltinJumpState::NoJump => return,
                TnuaBuiltinJumpState::StartingJump { .. } => AnimationState::Jumping,
                TnuaBuiltinJumpState::SlowDownTooFastSlopeJump { .. } => AnimationState::Jumping,
                TnuaBuiltinJumpState::MaintainingJump { .. } => AnimationState::Jumping,
                TnuaBuiltinJumpState::StoppedMaintainingJump => AnimationState::Jumping,
                TnuaBuiltinJumpState::FallSection => AnimationState::Falling,
            }
        }
        Some(TnuaBuiltinDash::NAME) => AnimationState::Dashing,
        // Tnua should only have the `action_name` of the actions you feed to it. If it has
        // anything else - consider it a bug.
        Some(other) => panic!("Unknown action {other}"),
        // No action name means that no action is currently being performed - which means the
        // animation should be decided by the basis.
        None => {
            // If there is no action going on, we'll base the animation on the state of the
            // basis.
            let Some((_, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>() else {
                // Since we only use the walk basis in this example, if we can't get get this
                // basis' state it probably means the system ran before any basis was set, so we
                // just stkip this frame.
                return;
            };
            if basis_state.standing_on_entity().is_none() {
                // The walk basis keeps track of what the character is standing on. If it doesn't
                // stand on anything, `standing_on_entity` will be empty - which means the
                // character has walked off a cliff and needs to fall.
                AnimationState::Falling
            } else {
                let speed = basis_state.running_velocity.length();
                if 0.01 < speed {
                    AnimationState::Running(0.1 * speed)
                } else {
                    AnimationState::Standing
                }
            }
        }
    };

    let animating_directive = animating_state.update_by_discriminant(current_status_for_animating);
    let age = aged.to_age();
    match animating_directive {
        bevy_tnua::TnuaAnimatingStateDirective::Maintain { state } => {}
        bevy_tnua::TnuaAnimatingStateDirective::Alter { old_state, state } => match state {
            AnimationState::Standing => {
                let (layout, sprite) = match age {
                    Age::Young => (
                        playerassets.yatlas_idle.clone(),
                        playerassets.ysprite_idle.clone(),
                    ),
                    Age::Old => (
                        playerassets.oatlas_idle.clone(),
                        playerassets.osprite_idle.clone(),
                    ),
                    Age::Ancient => (
                        playerassets.aatlas_idle.clone(),
                        playerassets.asprite_idle.clone(),
                    ),
                };
                animation.update_sprite(
                    Some(TextureAtlas {
                        layout: layout,
                        index: 0,
                    }),
                    sprite,
                );
                animation.set_frames(0, 2);
                animation.play();
            }
            AnimationState::Running(_) => {
                let (layout, sprite) = match age {
                    Age::Young => (
                        playerassets.yatlas_run.clone(),
                        playerassets.ysprite_run.clone(),
                    ),
                    Age::Old => (
                        playerassets.oatlas_run.clone(),
                        playerassets.osprite_run.clone(),
                    ),
                    Age::Ancient => (
                        playerassets.aatlas_run.clone(),
                        playerassets.asprite_run.clone(),
                    ),
                };
                animation.update_sprite(
                    Some(TextureAtlas {
                        layout: layout,
                        index: 1,
                    }),
                    sprite,
                );
                animation.set_frames(1, 5);
                animation.play();
            }
            AnimationState::Jumping => {
                let (layout, sprite) = match age {
                    Age::Young => (
                        playerassets.yatlas_jump.clone(),
                        playerassets.ysprite_jump.clone(),
                    ),
                    Age::Old => (
                        playerassets.oatlas_jump.clone(),
                        playerassets.osprite_jump.clone(),
                    ),
                    Age::Ancient => (
                        playerassets.aatlas_jump.clone(),
                        playerassets.asprite_jump.clone(),
                    ),
                };
                animation.update_sprite(
                    Some(TextureAtlas {
                        layout: layout,
                        index: 0,
                    }),
                    sprite,
                );
                animation.set_frames(0, 5);
                animation.play();
            }
            AnimationState::Falling => {
                let (layout, sprite) = match age {
                    Age::Young => (
                        playerassets.yatlas_jump.clone(),
                        playerassets.ysprite_jump.clone(),
                    ),
                    Age::Old => (
                        playerassets.oatlas_jump.clone(),
                        playerassets.osprite_jump.clone(),
                    ),
                    Age::Ancient => (
                        playerassets.aatlas_jump.clone(),
                        playerassets.asprite_jump.clone(),
                    ),
                };
                animation.update_sprite(
                    Some(TextureAtlas {
                        layout: layout,
                        index: 4,
                    }),
                    sprite,
                );
                animation.set_frames(4, 4);
                animation.play();
            }
            AnimationState::Dashing => {
                let (layout, sprite) = match age {
                    Age::Young => (
                        playerassets.yatlas_idle.clone(),
                        playerassets.ysprite_idle.clone(),
                    ),
                    Age::Old => (
                        playerassets.oatlas_idle.clone(),
                        playerassets.osprite_idle.clone(),
                    ),
                    Age::Ancient => (
                        playerassets.aatlas_idle.clone(),
                        playerassets.asprite_idle.clone(),
                    ),
                };
                animation.update_sprite(
                    Some(TextureAtlas {
                        layout: layout,
                        index: 0,
                    }),
                    sprite,
                );
                animation.set_frames(0, 0);
                animation.play();
            }
        },
    }
}

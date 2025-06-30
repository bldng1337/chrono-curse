use crate::game::player::Player;
use crate::game::world::collider::WallCollider;
use crate::{
    AgedSystems, AppSystems, PausableSystems,
    game::{age::Dead, enemies::Enemy, health::Health},
    screens::Screen,
};
use avian2d::{
    parry::query,
    prelude::{Collider, ColliderAabb, SimpleCollider, SpatialQuery, SpatialQueryFilter},
};
use bevy::{ecs::system::command, platform::collections::HashSet, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        update
            .in_set(AgedSystems)
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
}
#[derive(PartialEq)]
pub enum ProjectileTarget {
    Player,
    Enemies,
}

#[derive(Component)]
pub struct Projectile {
    pub target: ProjectileTarget,
    pub size: Vec2,
    pub dmg: f32,
    pub dir: Vec2,
}

fn update(
    mut query: Query<(&Projectile, &mut Transform, &GlobalTransform, Entity), Without<Dead>>,
    mut query_entity: Query<&mut Health>,

    enemies: Query<Entity, With<Enemy>>,
    walls: Query<Entity, With<WallCollider>>,
    player: Single<Entity, With<Player>>,

    spatial_query: SpatialQuery,
    mut command: Commands,
) {
    let player = player.into_inner();
    for (proj, mut transform, global, entity) in query.iter_mut() {
        transform.translation.x += proj.dir.x * 0.007;
        transform.translation.y += proj.dir.y * 0.007;
        let pos = global.translation().xy();
        let aabb = ColliderAabb::from_min_max(pos - (proj.size / 2.0), pos + (proj.size / 2.0));
        let got_hit = spatial_query.aabb_intersections_with_aabb(aabb);
        let mut hit = false;
        for collision in got_hit {
            if walls.contains(collision) {
                hit = true;
                continue;
            }
            if collision == player && proj.target == ProjectileTarget::Player {
                hit = true;
                if let Ok(mut health) = query_entity.get_mut(collision) {
                    health.damage(proj.dmg);
                }
            }
            if enemies.contains(collision) && proj.target == ProjectileTarget::Enemies {
                hit = true;
                if let Ok(mut health) = query_entity.get_mut(collision) {
                    health.damage(proj.dmg);
                }
            }
        }
        if hit {
            command.entity(entity).insert(Dead);
        }
    }
}

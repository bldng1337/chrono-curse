use avian2d::{
    parry::query,
    prelude::{Collider, ColliderAabb, SimpleCollider, SpatialQuery, SpatialQueryFilter},
};
use bevy::{ecs::system::command, platform::collections::HashSet, prelude::*};

use crate::{AppSystems, PausableSystems, game::health::Health, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        update
            .in_set(PausableSystems)
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Component)]
pub struct Projectile {
    pub owner: Entity,
    pub target: Option<Entity>,
    pub size: Vec2,
    pub dmg: f32,
    pub dir: Vec2,
}

fn update(
    mut query: Query<(&Projectile, &mut Transform, &GlobalTransform, Entity)>,
    mut query_entity: Query<&mut Health>,
    spatial_query: SpatialQuery,
    mut command: Commands,
) {
    for (proj, mut transform, global, entity) in query.iter_mut() {
        transform.translation.x += proj.dir.x * 0.007;
        transform.translation.y += proj.dir.y * 0.007;
        let pos = global.translation().xy();
        let aabb = ColliderAabb::from_min_max(pos - (proj.size / 2.0), pos + (proj.size / 2.0));
        let got_hit = spatial_query.aabb_intersections_with_aabb(aabb);
        let mut hit = false;
        for collision in got_hit {
            if proj.owner == collision {
                continue;
            }
            if let Some(entity) = proj.target {
                if entity != collision {
                    continue;
                }
            }
            hit = true;
            if let Ok(mut health) = query_entity.get_mut(collision) {
                health.damage(proj.dmg);
            }
        }
        if hit {
            if let Ok(mut entcommand) = command.get_entity(entity) {
                entcommand.despawn();
            }
        }
    }
}

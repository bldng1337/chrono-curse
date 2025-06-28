use bevy::prelude::*;

use crate::{AppSystems, screens::Screen};

pub const ENTITY_LAYER: i32=20;
pub const BACKGROUND_LAYER: i32=10;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        y_sort
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Component,Clone,Default)]
pub struct YSort {
    zlayer: f32,
    height: f32,
}

impl YSort {
    pub fn new(zlayer: i32, height: f32) -> Self {
        Self {
            zlayer: zlayer as f32,
            height,
        }
    }
}

fn y_sort(mut q: Query<(&mut Transform, &YSort, &GlobalTransform)>) {
    for (mut tf, ysort, gtransform) in q.iter_mut() {
        let basepoint = tf.translation.y - (ysort.height / 2.0);
        let base=(gtransform.translation().z*gtransform.scale().z)-(tf.translation.z*tf.scale.z);
        let zlayer=ysort.zlayer - (1.0f32 / (1.0f32 + (2.0f32.powf(-0.01 * basepoint))));
        tf.translation.z = zlayer-base;
    }
}

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_ecs_ldtk::prelude::*;

use avian2d::prelude::*;
use bevy_light_2d::prelude::*;

pub(super) fn plugin(app: &mut App) {
    // app.add_systems(
    //     Update,
    //     spawn_wall_collision.run_if(in_state(Screen::WorldGen)),
    // );
    for layer in ["backgroundcosmetic", "cosmetic"] {
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
            app.register_ldtk_entity_for_layer::<SimpleCosmeticBundle>(layer, simplecosmetics);
        }
        for lights in ["candles","chandelier","standing_light","walllamp"] {
            app.register_ldtk_entity_for_layer::<LightedBundle>(layer, lights);
        }
    }
    app.register_ldtk_entity_for_layer::<LightBundle>("cosmetic", "light");
    // app.register_ldtk_int_cell_for_layer::<WallBundle>("collider", 2);
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
struct LightedBundle {
    #[sprite_sheet]
    pub sprite_sheet: Sprite,
    #[ldtk_entity]
    pub light: LightBundle,
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
struct SimpleCosmeticBundle {
    #[sprite_sheet]
    pub sprite_sheet: Sprite,
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

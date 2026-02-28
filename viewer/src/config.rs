use bevy::prelude::Resource;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Resource)]
pub struct ViewerConfig {
    pub space_usda_path: String,
    pub placement_region_usda_path: String,
    pub regions_type_path: String,
    #[serde(default = "default_usda_scale")]
    pub usda_scale: f32,
}

impl Default for ViewerConfig {
    fn default() -> Self {
        ViewerConfig {
            space_usda_path: String::new(),
            placement_region_usda_path: String::new(),
            regions_type_path: String::new(),
            usda_scale: default_usda_scale(),
        }
    }
}

fn default_usda_scale() -> f32 {
    1.0
}

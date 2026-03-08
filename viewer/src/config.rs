use bevy::prelude::Resource;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Resource)]
pub struct ViewerConfig {
    pub space_usda_path: String,
    pub placement_region_usda_dir: String,
    pub regions_type_path: String,
}

impl Default for ViewerConfig {
    fn default() -> Self {
        ViewerConfig {
            space_usda_path: String::new(),
            placement_region_usda_dir: String::new(),
            regions_type_path: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SceneFileConfig {
    pub space_usda_path: String,
    pub placement_region_usda_dir: String,
    pub regions_type_path: String,
}

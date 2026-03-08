use std::path::Path;

use assets_import::{load_placement_regions_from_dir, load_regions_type_registry, load_space_model_from_usda};
use geometry_core::models::placement_region::PlacementRegion;
use geometry_core::models::placement_region_instance::{PlacementRegionInstance, PlacementTransform};
use geometry_core::models::space::Space;

use crate::config::SceneConfig;

pub struct SceneData {
    pub space: Space,
    pub placements: Vec<PlacementRegion>,
    pub instances: Vec<PlacementRegionInstance>,
}

pub fn load_scene_data(config: &SceneConfig) -> Result<SceneData, String> {
    let regions_type_ids = load_regions_type_registry(&config.regions_type_path)?;
    let space = load_space_model_from_usda(
        &config.space_usda_path,
        &regions_type_ids,
        1.0,
    )?;
    let placements = load_placement_regions_from_dir(
        Path::new(&config.placement_region_usda_dir),
        &regions_type_ids,
        1.0,
    )?;

    let instances = placements
        .iter()
        .enumerate()
        .map(|(idx, _)| PlacementRegionInstance {
            region_index: idx,
            transform: PlacementTransform::identity(),
        })
        .collect();

    Ok(SceneData {
        space,
        placements,
        instances,
    })
}

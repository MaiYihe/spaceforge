use bevy::log::{error, info};

use crate::config::SceneFileConfig;
use assets_import::{
    load_placement_regions_from_dir, load_regions_type_registry, load_space_model_from_usda,
};

use super::{IndexedTransform, ScenePayload, SceneTransforms};

pub(super) fn load_scene_from_json(path: &str) -> Option<ScenePayload> {
    let data = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(err) => {
            error!("Failed to read scene config {}: {}", path, err);
            return None;
        }
    };
    let mut config: SceneFileConfig = match serde_json::from_str(&data) {
        Ok(config) => config,
        Err(err) => {
            error!("Failed to parse scene config {}: {}", path, err);
            return None;
        }
    };

    let base = std::path::Path::new(path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    config.space_usda_path = resolve_path(base, &config.space_usda_path);
    config.placement_region_usda_dir = resolve_path(base, &config.placement_region_usda_dir);
    config.regions_type_path = resolve_path(base, &config.regions_type_path);

    info!(
        "Scene config resolved: space={} placement_dir={} regions_type={}",
        config.space_usda_path,
        config.placement_region_usda_dir,
        config.regions_type_path
    );

    let regions_type_ids = match load_regions_type_registry(&config.regions_type_path) {
        Ok(registry) => registry,
        Err(err) => {
            error!("Failed to load regions type registry: {}", err);
            return None;
        }
    };
    let space = match load_space_model_from_usda(
        &config.space_usda_path,
        &regions_type_ids,
        1.0,
    ) {
        Ok(space) => space,
        Err(err) => {
            error!("Failed to load Space from {}: {}", config.space_usda_path, err);
            return None;
        }
    };
    let placements = match load_placement_regions_from_dir(
        std::path::Path::new(&config.placement_region_usda_dir),
        &regions_type_ids,
        1.0,
    ) {
        Ok(placements) => placements,
        Err(err) => {
            error!(
                "Failed to load PlacementRegions from {}: {}",
                config.placement_region_usda_dir, err
            );
            return None;
        }
    };

    Some(ScenePayload { space, placements })
}

fn resolve_path(base: &std::path::Path, raw: &str) -> String {
    let p = std::path::Path::new(raw);
    if p.is_absolute() {
        return raw.to_string();
    }
    base.join(p).to_string_lossy().into_owned()
}

pub(super) fn load_transforms_resource() -> SceneTransforms {
    let path = std::env::var("SCENE_TRANSFORMS")
        .unwrap_or_else(|_| "/tmp/spaceforge/transforms.json".into());
    load_transforms_from_path(&path)
}

pub(super) fn load_transforms_from_path(path: &str) -> SceneTransforms {
    let data = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(_) => return SceneTransforms::default(),
    };

    #[derive(serde::Deserialize)]
    struct TransformFile {
        #[allow(dead_code)]
        version: u32,
        space_meshes: Vec<TransformEntry>,
        placements: Vec<TransformEntry>,
    }

    #[derive(serde::Deserialize)]
    struct TransformEntry {
        index: usize,
        matrix: [[f32; 4]; 4],
    }

    let parsed: TransformFile = match serde_json::from_str(&data) {
        Ok(p) => p,
        Err(_) => return SceneTransforms::default(),
    };

    SceneTransforms {
        space_meshes: parsed
            .space_meshes
            .into_iter()
            .map(|e| IndexedTransform {
                index: e.index,
                matrix: e.matrix,
            })
            .collect(),
        placements: parsed
            .placements
            .into_iter()
            .map(|e| IndexedTransform {
                index: e.index,
                matrix: e.matrix,
            })
            .collect(),
    }
}

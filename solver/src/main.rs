mod config;
mod export;
mod logging;

use config::load_scene_config;
use export::{
    export_debug_boundary_json, export_debug_points_json, export_scene_json, export_transforms_json,
};
use logging::init_logging;
use geometry_core::geometry_ops::{convex_hull_xz, sample_points_uv};
use utils::time_ms;

fn main() {
    init_logging();
    run_backend();
}

fn run_backend() {
    let config_path = std::env::var("ASSET_IMPORT_CONFIG")
        .unwrap_or_else(|_| "assets/config/asset_import.toml".to_string());
    match load_scene_config(&config_path) {
        Ok(config) => {
            if let Err(err) = export_scene_json(&config) {
                log::error!("Failed to export scene.json: {err}");
            } else {
                log::info!("Exported scene.json");
            }
            if let Err(err) = export_transforms_json() {
                log::error!("Failed to export transforms.json: {err}");
            } else {
                log::info!("Exported transforms.json");
            }

            if let Err(err) = export_debug_points(&config) {
                log::error!("Failed to export debug_points.json: {err}");
            } else {
                log::info!("Exported debug_points.json");
            }
        }
        Err(err) => {
            log::error!("Failed to load backend config: {err}");
        }
    }
    // TODO: wire geometry_core search/layout execution here.
    println!("Running backend-only mode (no viewer).");
}

fn export_debug_points(config: &config::SceneConfig) -> Result<(), String> {
    let regions_type_ids = assets_import::load_regions_type_registry(&config.regions_type_path)?;
    let space = assets_import::load_space_model_from_usda(
        &config.space_usda_path,
        &regions_type_ids,
        config.usda_scale,
    )?;
    let placements = assets_import::load_placement_regions_from_dir(
        std::path::Path::new(&config.placement_region_usda_dir),
        &regions_type_ids,
        config.usda_scale,
    )?;

    let mesh = space
        .meshes
        .get(0)
        .ok_or_else(|| "Space has no meshes (index 0 missing)".to_string())?;
    let sampled = time_ms("sample_points_uv", || sample_points_uv(mesh, 100.0));
    log::info!("sample_points_uv points={}", sampled.len());
    export_debug_points_json(&sampled)?;

    if let Some(first) = placements.first() {
        let hull = time_ms("convex_hull_xz", || {
            convex_hull_xz(&first.regions.forbidden_region.mesh)
        });
        log::info!("convex_hull_xz points={}", hull.len());
        export_debug_boundary_json(&hull)?;
    } else {
        log::info!("convex_hull_xz skipped (no PlacementRegions)");
    }

    Ok(())
}

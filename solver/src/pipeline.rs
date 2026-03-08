use crate::config::load_scene_config;
use crate::export::{
    export_debug_boundary_json, export_debug_forbidden_points_json, export_debug_points_json,
    export_scene_json, export_space_boundaries_json, export_transforms_json, export_nfp_debug_json,
        export_ifp_debug_json,
};
use crate::steps::run::run_steps;

pub fn run_backend(config_path: &str) {
    match load_scene_config(config_path) {
        Ok(config) => {
            if let Err(err) = export_scene_json(&config) {
                log::error!("Failed to export scene.json: {err}");
            } else {
                log::info!("Exported scene.json");
            }
            match run_steps(&config) {
                Ok(outputs) => {
                    if let Err(err) = export_transforms_json(&outputs.transforms) {
                        log::error!("Failed to export transforms.json: {err}");
                    } else {
                        log::info!("Exported transforms.json");
                    }

                    if let Err(err) = export_debug_points_json(&outputs.space_points) {
                        log::error!("Failed to export debug_points.json: {err}");
                    } else {
                        log::info!("Exported debug_points.json");
                    }
                    if let Err(err) = export_space_boundaries_json(&outputs.space_boundaries) {
                        log::error!("Failed to export debug_space_boundary.json: {err}");
                    } else {
                        log::info!("Exported debug_space_boundary.json");
                    }

                    if let Err(err) = export_debug_forbidden_points_json(&outputs.forbidden_points)
                    {
                        log::error!("Failed to export debug_forbidden_points.json: {err}");
                    } else {
                        log::info!("Exported debug_forbidden_points.json");
                    }

                    if let Err(err) = export_debug_boundary_json(&outputs.forbidden_hull) {
                        log::error!("Failed to export debug_boundary.json: {err}");
                    } else {
                        log::info!("Exported debug_boundary.json");
                    }

                    if let Err(err) = export_nfp_debug_json(&outputs.nfp_rect, &outputs.nfp_result) {
                        log::error!("Failed to export debug_nfp.json: {err}");
                    } else {
                        log::info!("Exported debug_nfp.json");
                    }

                    if let Err(err) = export_ifp_debug_json(
                        &outputs.ifp_result,
                        &outputs.ifp_sample_points,
                        &outputs.ifp_instance_loops,
                    ) {
                        log::error!("Failed to export debug_ifp.json: {err}");
                    } else {
                        log::info!("Exported debug_ifp.json");
                    }
                }
                Err(err) => {
                    log::error!("Failed to run steps: {err}");
                }
            }
        }
        Err(err) => {
            log::error!("Failed to load backend config: {err}");
        }
    }
}

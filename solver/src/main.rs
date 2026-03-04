mod config;
mod export;
mod logging;

use config::load_scene_config;
use export::{export_scene_json, export_transforms_json};
use logging::init_logging;

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
        }
        Err(err) => {
            log::error!("Failed to load backend config: {err}");
        }
    }
    // TODO: wire geometry_core search/layout execution here.
    println!("Running backend-only mode (no viewer).");
}

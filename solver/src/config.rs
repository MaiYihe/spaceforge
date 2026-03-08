use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SceneConfig {
    pub space_usda_path: String,
    pub placement_region_usda_dir: String,
    pub regions_type_path: String,
}

pub fn load_scene_config(path: &str) -> Result<SceneConfig, String> {
    let data = std::fs::read_to_string(path)
        .map_err(|err| format!("Failed to read config {}: {err}", path))?;
    let mut config: SceneConfig = toml::from_str(&data)
        .map_err(|err| format!("Failed to parse config {}: {err}", path))?;
    let base = std::path::Path::new(path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    config.space_usda_path = resolve_path(base, &config.space_usda_path);
    config.placement_region_usda_dir =
        resolve_path(base, &config.placement_region_usda_dir);
    config.regions_type_path = resolve_path(base, &config.regions_type_path);
    Ok(config)
}

fn resolve_path(base: &std::path::Path, raw: &str) -> String {
    let p = std::path::Path::new(raw);
    if p.is_absolute() {
        return raw.to_string();
    }
    let joined = base.join(p);
    match std::fs::canonicalize(&joined) {
        Ok(abs) => abs.to_string_lossy().into_owned(),
        Err(_) => joined.to_string_lossy().into_owned(),
    }
}

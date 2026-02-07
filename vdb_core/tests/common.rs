use std::fs;
use std::path::{Path, PathBuf};

pub fn load_first_obj_path() -> Result<String, String> {
    let config_path = std::env::var("VIEWER_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("app/config/viewer.toml"));

    let data = fs::read_to_string(&config_path)
        .map_err(|e| format!("read config failed: {}", e))?;
    let value: toml::Value = toml::from_str(&data)
        .map_err(|e| format!("parse config failed: {}", e))?;

    let placements = value
        .get("placements")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "config missing placements array".to_string())?;

    let first = placements
        .get(0)
        .and_then(|v| v.as_table())
        .ok_or_else(|| "config has no first placement".to_string())?;

    let obj_path = first
        .get("obj_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "placement.obj_path missing".to_string())?;

    let path = Path::new(obj_path);
    if path.is_absolute() {
        return Ok(path.to_string_lossy().into_owned());
    }

    let base = config_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let joined = base.join(path);
    Ok(joined.to_string_lossy().into_owned())
}

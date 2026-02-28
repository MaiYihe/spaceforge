use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn resolve_script_path(default_rel: &str) -> Result<PathBuf, String> {
    let rel_path = if let Ok(root) = env::var("SPACEFORGE_ROOT") {
        PathBuf::from(root).join(default_rel)
    } else {
        PathBuf::from(default_rel)
    };

    if rel_path.exists() {
        return Ok(rel_path);
    }

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let candidate = PathBuf::from(manifest_dir)
            .parent()
            .map(|p| p.join(default_rel));
        if let Some(path) = candidate {
            if path.exists() {
                return Ok(path);
            }
        }
    }

    Err(format!(
        "usd parser script not found: {}",
        rel_path.display()
    ))
}

pub fn run_python(script_path: &Path, usda_path: &str) -> Result<String, String> {
    ensure_usda(usda_path)?;

    let output = Command::new("python3")
        .arg(script_path)
        .arg(usda_path)
        .output()
        .map_err(|e| format!("failed to run python3: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "usd parser failed ({}) with: {}",
            script_path.display(),
            stderr.trim()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn ensure_usda(path: &str) -> Result<(), String> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .ok_or_else(|| format!("path has no extension: {path}"))?;
    if ext != "usda" {
        return Err(format!("unsupported format .{ext}; only .usda is supported"));
    }
    Ok(())
}

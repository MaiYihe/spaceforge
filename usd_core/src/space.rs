use serde::Deserialize;

use crate::common::{resolve_script_path, run_python};
use crate::types::{SpaceUsd, UsdMesh, UsdMeshData};

#[derive(Debug, Deserialize)]
struct UsdMeshJson {
    path: String,
    points: Vec<[f32; 3]>,
    indices: Vec<u32>,
    counts: Vec<u32>,
    #[serde(rename = "regionsTypeMask")]
    regions_type_mask: Option<Vec<String>>,
}

pub fn load_space_usda(path: &str) -> Result<Vec<SpaceUsd>, String> {
    let script = resolve_script_path("app/assets/models/input_space/parse_space_usda.py")?;
    let stdout = run_python(&script, path)?;

    let meshes: Vec<UsdMeshJson> =
        serde_json::from_str(&stdout).map_err(|e| format!("parse usd json failed: {e}"))?;

    Ok(meshes
        .into_iter()
        .map(|mesh| SpaceUsd {
            mesh: UsdMesh {
                path: mesh.path,
                mesh: UsdMeshData {
                    points: mesh.points,
                    indices: mesh.indices,
                    counts: mesh.counts,
                },
            },
            regions_type_mask: mesh.regions_type_mask,
        })
        .collect())
}

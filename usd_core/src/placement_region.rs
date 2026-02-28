//! Placement-region USD parsing helpers.
use serde::Deserialize;

use crate::common::{resolve_script_path, run_python};
use crate::types::{PlacementRegionUsd, UsdMesh, UsdMeshData};
#[derive(Debug, Deserialize)]
struct PlacementRegionMeshJson {
    path: String,
    points: Vec<[f32; 3]>,
    indices: Vec<u32>,
    counts: Vec<u32>,
}

#[derive(Debug, Deserialize)]
struct PlacementRegionUsdJson {
    #[serde(rename = "regionsTypeName")]
    regions_type_name: String,
    name: String,
    count: Option<i32>,
    #[serde(rename = "heightRange")]
    height_range: Option<[f32; 2]>,
    #[serde(rename = "restrictedRegion")]
    restricted_region: Option<PlacementRegionMeshJson>,
    #[serde(rename = "forbiddenRegion")]
    forbidden_region: Option<PlacementRegionMeshJson>,
    #[serde(rename = "footprint2d")]
    footprint_2d: Option<PlacementRegionMeshJson>,
}

pub fn load_placement_region_usda(path: &str) -> Result<PlacementRegionUsd, String> {
    let script = resolve_script_path(
        "app/assets/models/input_placement_region/parse_placement_region_usda.py",
    )?;
    let stdout = run_python(&script, path)?;

    let region: PlacementRegionUsdJson =
        serde_json::from_str(&stdout).map_err(|e| format!("parse usd json failed: {e}"))?;

    Ok(PlacementRegionUsd {
        regions_type_name: region.regions_type_name,
        name: region.name,
        count: region.count,
        height_range: region.height_range,
        restricted_region: region.restricted_region.map(placement_mesh_from_json),
        forbidden_region: region.forbidden_region.map(placement_mesh_from_json),
        footprint_2d: region.footprint_2d.map(placement_mesh_from_json),
    })
}

fn placement_mesh_from_json(mesh: PlacementRegionMeshJson) -> UsdMesh {
    UsdMesh {
        path: mesh.path,
        mesh: UsdMeshData {
            points: mesh.points,
            indices: mesh.indices,
            counts: mesh.counts,
        },
    }
}

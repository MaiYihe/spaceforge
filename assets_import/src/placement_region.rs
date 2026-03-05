use usd_core::UsdMesh;
use geometry_core::models::placement_region::{
    HeightRange, Mesh as RegionMesh, PlacementRegion, PlacementSemantics, Region, Regions, Visual,
};
use log::info;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Once};
use utils::time_ms;
use types::RegionsType;
use usd_core::load_placement_region_usda;
use vdb_core::VdbGrid;

pub fn load_placement_region_model_from_usda(
    path: &str,
    regions_type_ids: &HashMap<String, RegionsType>,
    scale: f32,
) -> Result<PlacementRegion, String> {
    info!("assets_import: loading PlacementRegion from {}", path);
    let region = load_placement_region_usda(path)?;
    let unit_scale = unit_scale_factor(region.unit.as_deref())?;
    let scale = scale * unit_scale;

    let regions_type_name = &region.regions_type_name;
    let regions_type = *regions_type_ids
        .get(regions_type_name)
        .ok_or_else(|| format!("regions type '{regions_type_name}' not found in registry"))?;

    let restricted_mesh = region
        .restricted_region
        .as_ref()
        .ok_or_else(|| "placement region has no restricted_region mesh".to_string())?;
    let forbidden_mesh = region
        .forbidden_region
        .as_ref()
        .ok_or_else(|| "placement region has no forbidden_region mesh".to_string())?;

    let restricted_local = to_mesh_data_from_placement(restricted_mesh, scale)?;
    let forbidden_local = to_mesh_data_from_placement(forbidden_mesh, scale)?;

    let restricted_sdf = build_sdf(&restricted_local.positions, &restricted_local.indices)?;
    log_sdf_voxels("restricted_region", &restricted_sdf);
    let forbidden_sdf = build_sdf(&forbidden_local.positions, &forbidden_local.indices)?;
    log_sdf_voxels("forbidden_region", &forbidden_sdf);

    let footprint_mesh = if let Some(mesh) = region.footprint_2d.as_ref() {
        let local = to_mesh_data_from_placement(mesh, scale)?;
        Some(RegionMesh {
            positions: local.positions,
            indices: local.indices,
        })
    } else {
        None
    };

    let (min_y, max_y) = region
        .height_range
        .map(|range| (range[0], range[1]))
        .unwrap_or((0.0, 0.0));

    let region = PlacementRegion {
        regions: Regions {
            forbidden_region: Region {
                mesh: RegionMesh {
                    positions: forbidden_local.positions,
                    indices: forbidden_local.indices,
                },
                sdf: Some(forbidden_sdf),
            },
            restricted_region: Region {
                mesh: RegionMesh {
                    positions: restricted_local.positions,
                    indices: restricted_local.indices,
                },
                sdf: Some(restricted_sdf),
            },
        },
        semantics: PlacementSemantics {
            regions_type,
            count: region.count.unwrap_or(1),
        },
        visual: Visual {
            footprint_2d: footprint_mesh.unwrap_or_default(),
            height_range: HeightRange { min_y, max_y },
        },
    };
    Ok(region)
}

fn build_sdf(positions: &[[f32; 3]], indices: &[u32]) -> Result<geometry_core::models::placement_region::SdfGrid, String> {
    const VOXEL_SIZE_MM: f32 = 20.0;
    ensure_vdb_init();
    info!(
        "assets_import: building sdf voxels (positions={}, indices={})",
        positions.len(),
        indices.len()
    );
    let grid = time_ms("assets_import: sdf build", || {
        VdbGrid::from_mesh(positions, indices, VOXEL_SIZE_MM, 1.0)
    })?;
    Ok(geometry_core::models::placement_region::SdfGrid {
        grid: Arc::new(grid),
        voxel_size: VOXEL_SIZE_MM,
    })
}

fn ensure_vdb_init() {
    static INIT: Once = Once::new();
    INIT.call_once(VdbGrid::init);
}

fn log_sdf_voxels(label: &str, sdf: &geometry_core::models::placement_region::SdfGrid) {
    match sdf.grid.active_voxel_coords() {
        Ok(coords) => {
            info!("assets_import: {label} voxel count={}", coords.len());
        }
        Err(err) => {
            info!("assets_import: {label} voxel count unavailable: {err}");
        }
    }
}

pub fn load_placement_regions_from_dir(
    dir: &Path,
    regions_type_ids: &HashMap<String, RegionsType>,
    scale: f32,
) -> Result<Vec<PlacementRegion>, String> {
    info!(
        "assets_import: loading PlacementRegions from dir {}",
        dir.display()
    );
    let mut entries = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(path) = stack.pop() {
        let read_dir = fs::read_dir(&path).map_err(|e| {
            format!(
                "read placement region dir failed ({}): {e}",
                path.display()
            )
        })?;

        for entry in read_dir {
            let entry = entry.map_err(|e| {
                format!(
                    "read placement region dir entry failed ({}): {e}",
                    path.display()
                )
            })?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }
            if entry_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("usda"))
                .unwrap_or(false)
            {
                entries.push(entry_path);
            }
        }
    }

    entries.sort();

    if entries.is_empty() {
        return Err(format!(
            "no .usda files found in placement region dir: {}",
            dir.display()
        ));
    }

    let mut out = Vec::with_capacity(entries.len());
    let mut named = Vec::with_capacity(entries.len());
    for path in entries {
        let path_str = path
            .to_str()
            .ok_or_else(|| format!("invalid path in placement region dir: {}", path.display()))?;
        let region = load_placement_region_model_from_usda(path_str, regions_type_ids, scale)
            .map_err(|e| format!("failed to load placement region usda '{}': {e}", path.display()))?;
        let name = path
            .strip_prefix(dir)
            .ok()
            .and_then(|p| p.to_str())
            .unwrap_or(path_str)
            .to_string();
        out.push(region);
        let idx = out.len() - 1;
        named.push((name, idx));
    }

    log_placement_json(&named, &out);
    Ok(out)
}

fn log_placement_json(placements: &[(String, usize)], regions: &[PlacementRegion]) {
    let payload = placements
        .iter()
        .enumerate()
        .map(|(idx, (name, region_idx))| {
            let placement = &regions[*region_idx];
            let restricted = &placement.regions.restricted_region.mesh;
            let forbidden = &placement.regions.forbidden_region.mesh;
            let footprint = &placement.visual.footprint_2d;
            let restricted_voxels = placement
                .regions
                .restricted_region
                .sdf
                .as_ref()
                .and_then(|sdf| sdf.grid.active_voxel_coords().ok())
                .map(|v| v.len());
            let forbidden_voxels = placement
                .regions
                .forbidden_region
                .sdf
                .as_ref()
                .and_then(|sdf| sdf.grid.active_voxel_coords().ok())
                .map(|v| v.len());
            json!({
                "id": idx,
                "file": name,
                "regions": {
                    "restricted_region": {
                        "mesh": {
                            "vertices": restricted.positions.len(),
                            "indices": restricted.indices.len()
                        },
                        "sdf": {
                            "voxel_count": restricted_voxels
                        }
                    },
                    "forbidden_region": {
                        "mesh": {
                            "vertices": forbidden.positions.len(),
                            "indices": forbidden.indices.len()
                        },
                        "sdf": {
                            "voxel_count": forbidden_voxels
                        }
                    }
                },
                "semantics": {
                    "regions_type": placement.semantics.regions_type,
                    "count": placement.semantics.count
                },
                "visual": {
                    "footprint_2d": {
                        "mesh": {
                            "vertices": footprint.positions.len(),
                            "indices": footprint.indices.len()
                        }
                    },
                    "height_range": {
                        "min_y": placement.visual.height_range.min_y,
                        "max_y": placement.visual.height_range.max_y
                    }
                }
            })
        })
        .collect::<Vec<_>>();
    if let Ok(text) = serde_json::to_string_pretty(&payload) {
        info!("PlacementRegions JSON:\n{}", text);
    }
}

fn unit_scale_factor(unit: Option<&str>) -> Result<f32, String> {
    let unit = match unit {
        Some(u) => u.trim().to_ascii_lowercase(),
        None => return Ok(1.0),
    };
    match unit.as_str() {
        "mm" => Ok(1.0),
        "cm" => Ok(10.0),
        "m" | "meter" | "meters" => Ok(1000.0),
        other => Err(format!("unsupported unit '{other}'")),
    }
}

fn to_mesh_data_from_placement(
    mesh: &UsdMesh,
    scale: f32,
) -> Result<crate::usda_common::MeshData, String> {
    let positions: Vec<[f32; 3]> = mesh
        .mesh
        .points
        .iter()
        .map(|p| [p[0] * scale, p[1] * scale, p[2] * scale])
        .collect();

    let mut indices = Vec::<u32>::new();
    let mut cursor = 0usize;

    for &count in &mesh.mesh.counts {
        let count = count as usize;
        if count < 3 {
            cursor = cursor.saturating_add(count);
            continue;
        }
        if cursor + count > mesh.mesh.indices.len() {
            return Err(format!(
                "mesh '{}' has inconsistent face counts/indices",
                mesh.path
            ));
        }

        let base = mesh.mesh.indices[cursor] as usize;
        for i in 1..count - 1 {
            let i1 = mesh.mesh.indices[cursor + i] as usize;
            let i2 = mesh.mesh.indices[cursor + i + 1] as usize;
            if base >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
                return Err(format!("mesh '{}' has out-of-range face index", mesh.path));
            }
            indices.push(base as u32);
            indices.push(i1 as u32);
            indices.push(i2 as u32);
        }

        cursor += count;
    }

    if positions.is_empty() || indices.is_empty() {
        return Err(format!("mesh '{}' has no triangles", mesh.path));
    }

    Ok(crate::usda_common::MeshData { positions, indices })
}

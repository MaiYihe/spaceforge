use usd_core::UsdMesh;
use geometry_core::models::placement_region::{
    HeightRange, Mesh as RegionMesh, PlacementRegion, PlacementSemantics, Region, Regions, Visual,
};
use std::collections::HashMap;
use types::RegionsType;
use usd_core::load_placement_region_usda;

pub fn load_placement_region_model_from_usda(
    path: &str,
    regions_type_ids: &HashMap<String, RegionsType>,
    scale: f32,
) -> Result<PlacementRegion, String> {
    let region = load_placement_region_usda(path)?;

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

    Ok(PlacementRegion {
        regions: Regions {
            forbidden_region: Region {
                mesh: RegionMesh {
                    positions: forbidden_local.positions,
                    indices: forbidden_local.indices,
                },
                sdf: None,
            },
            restricted_region: Region {
                mesh: RegionMesh {
                    positions: restricted_local.positions,
                    indices: restricted_local.indices,
                },
                sdf: None,
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
    })
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

use crate::usda_common::{
    ensure_usda, extract_first_xform_name, parse_usda_meshes, to_mesh_data,
};
use geometry_core::models::placement_region::{
    HeightRange, Mesh as RegionMesh, PlacementRegion, PlacementSemantics, Region, Regions, Visual,
};
use std::collections::HashMap;
use std::fs;
use types::RegionsTypeId;

pub fn load_placement_region_model_from_usda(
    path: &str,
    regions_type_ids: &HashMap<String, RegionsTypeId>,
    scale: f32,
) -> Result<PlacementRegion, String> {
    ensure_usda(path)?;
    let data = fs::read_to_string(path).map_err(|e| format!("read usda failed: {e}"))?;
    let meshes = parse_usda_meshes(&data)?;

    let regions_type_name = extract_first_xform_name(&data)
        .ok_or_else(|| "cannot find root xform name from usda".to_string())?;
    let regions_type_id = *regions_type_ids
        .get(&regions_type_name)
        .ok_or_else(|| format!("regions type '{regions_type_name}' not found in registry"))?;

    let mut forbidden = Vec::<Region>::new();
    let mut restricted = Vec::<Region>::new();
    let mut all_y = Vec::<f32>::new();

    for mesh in meshes {
        let local = to_mesh_data(&mesh, scale)?;
        for p in &local.positions {
            all_y.push(p[1]);
        }

        let region = Region {
            mesh: RegionMesh {
                positions: local.positions,
                indices: local.indices,
            },
            sdf: None,
        };

        let lname = mesh.name.to_ascii_lowercase();
        if lname.contains("forbidden") {
            forbidden.push(region);
        } else if lname.contains("restricted") {
            restricted.push(region);
        }
    }

    if forbidden.is_empty() && restricted.is_empty() {
        return Err("usda has no forbidden/restricted region meshes".to_string());
    }

    let (min_y, max_y) = if all_y.is_empty() {
        (0.0, 0.0)
    } else {
        let mut min_y = all_y[0];
        let mut max_y = all_y[0];
        for y in &all_y[1..] {
            min_y = min_y.min(*y);
            max_y = max_y.max(*y);
        }
        (min_y, max_y)
    };

    Ok(PlacementRegion {
        region: Regions {
            forbidden,
            restricted,
        },
        semantics: PlacementSemantics { regions_type_id },
        visual: Visual {
            footprint_2d: RegionMesh::default(),
            height_range: HeightRange { min_y, max_y },
        },
    })
}

use crate::usda_common::{mask_from_names, to_mesh_data_from_usd};
use geometry_core::geometry_ops::mesh_boundary_loop;
use geometry_core::models::space::{Mesh as SpaceMesh, Space, SpaceSurface, SurfaceMeta};
use log::info;
use serde_json::json;
use std::collections::HashMap;
use types::{RegionsType, RegionsTypeMask};
use usd_core::load_space_usda;

pub fn load_space_model_from_usda(
    path: &str,
    regions_type_ids: &HashMap<String, RegionsType>,
    scale: f32,
) -> Result<Space, String> {
    info!("assets_import: loading Space from {}", path);
    let scene = load_space_usda(path)?;
    let unit_scale = unit_scale_factor(scene.unit.as_deref())?;
    let scale = scale * unit_scale;

    let mut surfaces = Vec::<SpaceSurface>::new();
    let mut metas = Vec::<SurfaceMeta>::new();

    for usd_mesh in scene.meshes {
        let local = to_mesh_data_from_usd(&usd_mesh.mesh, scale)?;
        let mesh = SpaceMesh {
            positions: local.positions,
            indices: local.indices,
        };
        let boundary = mesh_boundary_loop(&mesh);
        surfaces.push(SpaceSurface { mesh, boundary });

        let mask = if usd_mesh
            .regions_type_mask
            .as_ref()
            .map_or(true, |m| m.is_empty())
        {
            RegionsTypeMask::NONE
        } else {
            mask_from_names(usd_mesh.regions_type_mask.as_ref().unwrap(), regions_type_ids)?
        };
        metas.push(SurfaceMeta {
            regions_type_mask: mask,
        });
    }

    let space = Space {
        surfaces,
        surface_metas: metas,
    };
    log_space_json(&space);
    Ok(space)
}

fn log_space_json(space: &Space) {
    let meshes = space
        .surface_metas
        .iter()
        .enumerate()
        .map(|(idx, meta)| {
            json!({
                "mesh_id": idx,
                "mesh": "Mesh",
                "boundary_vertices": space.surfaces.get(idx).map(|s| s.boundary.len()).unwrap_or(0),
                "regions_type_mask": meta.regions_type_mask.bits()
            })
        })
        .collect::<Vec<_>>();
    let payload = json!({ "meshes": meshes });
    if let Ok(text) = serde_json::to_string_pretty(&payload) {
        info!("Space JSON:\n{}", text);
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

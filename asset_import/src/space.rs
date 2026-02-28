use crate::usda_common::{mask_from_names, to_mesh_data_from_usd};
use geometry_core::models::space::{Mesh as SpaceMesh, Space, SurfaceMeta};
use std::collections::HashMap;
use types::{RegionsType, RegionsTypeMask};
use usd_core::load_space_usda;

pub fn load_space_model_from_usda(
    path: &str,
    regions_type_ids: &HashMap<String, RegionsType>,
    scale: f32,
) -> Result<Space, String> {
    let meshes = load_space_usda(path)?;

    let mut space_meshes = Vec::<SpaceMesh>::new();
    let mut metas = Vec::<SurfaceMeta>::new();

    for mesh in meshes {
        let local = to_mesh_data_from_usd(&mesh.mesh, scale)?;
        space_meshes.push(SpaceMesh {
            positions: local.positions,
            indices: local.indices,
        });

        let mask = if mesh.regions_type_mask.as_ref().map_or(true, |m| m.is_empty()) {
            RegionsTypeMask::NONE
        } else {
            mask_from_names(mesh.regions_type_mask.as_ref().unwrap(), regions_type_ids)?
        };
        metas.push(SurfaceMeta {
            regions_type_mask: mask,
        });
    }

    Ok(Space {
        meshes: space_meshes,
        surface_metas: metas,
    })
}

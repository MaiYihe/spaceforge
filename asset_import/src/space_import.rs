use crate::usda_common::{
    ensure_usda, mask_from_names, parse_usda_meshes, to_mesh_data,
};
use geometry_core::models::space::{Mesh as SpaceMesh, Space, SurfaceMeta};
use std::collections::HashMap;
use std::fs;
use types::{RegionsTypeId, RegionsTypeMask};

pub fn load_space_model_from_usda(
    path: &str,
    regions_type_ids: &HashMap<String, RegionsTypeId>,
    scale: f32,
) -> Result<Space, String> {
    ensure_usda(path)?;
    let data = fs::read_to_string(path).map_err(|e| format!("read usda failed: {e}"))?;
    let meshes = parse_usda_meshes(&data)?;

    let mut space_meshes = Vec::<SpaceMesh>::new();
    let mut metas = Vec::<SurfaceMeta>::new();

    for mesh in meshes {
        let local = to_mesh_data(&mesh, scale)?;
        space_meshes.push(SpaceMesh {
            positions: local.positions,
            indices: local.indices,
        });

        let mask = if mesh.regions_type_names.is_empty() {
            RegionsTypeMask::ALL
        } else {
            mask_from_names(&mesh.regions_type_names, regions_type_ids)?
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

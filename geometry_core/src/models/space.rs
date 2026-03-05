use types::RegionsTypeMask;

#[derive(Debug, Clone)]
pub struct SurfaceMeta {
    pub regions_type_mask: RegionsTypeMask,
}

#[derive(Debug, Clone)]
pub struct Space {
    pub meshes: Vec<Mesh>,
    /// 每个 surface 对应一个 metadata
    pub surface_metas: Vec<SurfaceMeta>,
}

pub type Mesh = crate::models::mesh::Mesh;

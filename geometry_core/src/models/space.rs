use types::RegionsTypeMask;

#[derive(Debug, Clone)]
pub struct SurfaceMeta {
    pub regions_type_mask: RegionsTypeMask,
}

#[derive(Debug, Clone)]
pub struct Space {
    pub surfaces: Vec<SpaceSurface>,
    /// 每个 surface 对应一个 metadata
    pub surface_metas: Vec<SurfaceMeta>,
}

#[derive(Debug, Clone)]
pub struct SpaceSurface {
    pub mesh: Mesh,
    /// Boundary loop (topological edges that appear once), ordered as a polyline.
    pub boundary: Vec<[f32; 3]>,
}

pub type Mesh = crate::models::mesh::Mesh;

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

#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

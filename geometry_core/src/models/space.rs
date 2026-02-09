use category_types::{CategoryMask, SurfaceId};

#[derive(Debug, Clone)]
pub struct SurfaceMeta {
    pub kind: SurfaceId,
    pub allowed: CategoryMask,
}

#[derive(Debug, Clone)]
pub struct Space {
    pub mesh: Mesh,
    /// 每个 surface 对应一个 metadata
    pub surfaces: Vec<SurfaceMeta>,
}

// ---------- Placeholder geometry type ----------
#[derive(Debug, Clone)]
pub struct Mesh;

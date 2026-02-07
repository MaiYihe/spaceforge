use furniture_types::FurnitureMask;

#[derive(Debug, Clone, Copy)]
pub enum SurfaceKind {
    Floor,
    Wall,
    Ceiling,
}

#[derive(Debug, Clone)]
pub struct SurfaceMeta {
    pub kind: SurfaceKind,
    pub allowed: FurnitureMask,
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

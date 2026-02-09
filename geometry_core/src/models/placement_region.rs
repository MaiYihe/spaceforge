use category_types::{CategoryId, SurfaceId};

#[derive(Debug, Clone)]
pub struct PlacementRegion {
    /// 几何区域
    pub regions: Vec<Region>,
    /// 部署语义（规则层）
    pub semantics: PlacementSemantics,
    /// 可视化壳（纯 display）
    pub visual: Visual,
}

#[derive(Debug, Clone)]
pub struct Region {
    pub class: RegionClass,
    pub mesh: Mesh,
    pub sdf: Option<SdfGrid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionClass {
    Forbidden,
    Restricted,
}

#[derive(Debug, Clone)]
pub struct PlacementSemantics {
    pub attach: SurfaceId,
    pub category_id: CategoryId,
}

#[derive(Debug, Clone)]
pub struct Visual {
    pub footprint_2d: Shape2D,
    pub height_range: HeightRange,
}

// ---------- Placeholder geometry types ----------
#[derive(Debug, Clone)]
pub struct Mesh;

#[derive(Debug, Clone)]
pub struct SdfGrid;

#[derive(Debug, Clone)]
pub struct Shape2D;

#[derive(Debug, Clone)]
pub struct HeightRange;

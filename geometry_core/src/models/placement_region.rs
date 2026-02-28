use types::RegionsType;

#[derive(Debug, Clone)]
pub struct PlacementRegion {
    /// 几何区域
    pub regions: Regions,
    /// 部署语义（规则层）
    pub semantics: PlacementSemantics,
    /// 可视化壳（纯 display）
    pub visual: Visual,
}

#[derive(Debug, Clone)]
pub struct Regions {
    pub forbidden_region: Region,
    pub restricted_region: Region,
}

#[derive(Debug, Clone)]
pub struct Region {
    pub mesh: Mesh,
    pub sdf: Option<SdfGrid>,
}

#[derive(Debug, Clone)]
pub struct PlacementSemantics {
    pub regions_type: RegionsType,
    pub count: i32,
}

#[derive(Debug, Clone)]
pub struct Visual {
    pub footprint_2d: Mesh,
    pub height_range: HeightRange,
}

#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct SdfGrid;

#[derive(Debug, Clone, Default)]
pub struct HeightRange {
    pub min_y: f32,
    pub max_y: f32,
}

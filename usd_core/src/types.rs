#[derive(Debug, Clone)]
pub struct UsdMeshData {
    pub points: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub counts: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct UsdMesh {
    pub path: String,
    pub mesh: UsdMeshData,
}

#[derive(Debug, Clone)]
pub struct SpaceUsd {
    pub mesh: UsdMesh,
    pub regions_type_mask: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct PlacementRegionUsd {
    pub regions_type_name: String,
    pub name: String,
    pub count: Option<i32>,
    pub height_range: Option<[f32; 2]>,
    pub restricted_region: Option<UsdMesh>,
    pub forbidden_region: Option<UsdMesh>,
    pub footprint_2d: Option<UsdMesh>,
}

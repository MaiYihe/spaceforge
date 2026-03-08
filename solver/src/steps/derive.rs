use geometry_core::geometry_ops::sample_points_uv;
use geometry_core::models::placement_region::PlacementRegion;
use geometry_core::models::placement_region_instance::PlacementRegionInstance;
use geometry_core::models::placement_region_instance::PlacementRegionDerived;

use crate::placement_region_cache::PlacementRegionCache;
use crate::steps::transform::transform_key;
use utils::time_ms;

pub fn derive_space_points(space: &geometry_core::models::space::Space) -> Result<Vec<[f32; 3]>, String> {
    let mesh = &space
        .surfaces
        .get(0)
        .ok_or_else(|| "Space has no surfaces (index 0 missing)".to_string())?
        .mesh;
    let sampled = time_ms("sample_points_uv", || sample_points_uv(mesh, 100.0));
    log::info!("sample_points_uv points={}", sampled.len());
    Ok(sampled)
}

pub fn derive_space_boundaries(
    space: &geometry_core::models::space::Space,
) -> Vec<Vec<[f32; 3]>> {
    space
        .surfaces
        .iter()
        .map(|surface| surface.boundary.clone())
        .collect()
}

pub fn derive_forbidden_region(
    placements: &[PlacementRegion],
    instances: &[PlacementRegionInstance],
) -> Option<PlacementRegionDerived> {
    let (first, instance) = placements.first().zip(instances.first())?;
    let mut cache = PlacementRegionCache::new();
    let key = transform_key(&instance.transform);
    let derived = time_ms("placement_region_derived", || {
        cache.get_or_compute(0, first, instance.transform, key, 2000, 42)
    });
    log::info!("forbidden samples points={}", derived.samples.len());
    log::info!("convex_hull_xz points={}", derived.hull.len());
    Some(derived.clone())
}

use std::collections::HashMap;

use geometry_core::geometry_ops::{convex_hull_xz_points, sample_points_area};
use geometry_core::models::placement_region::PlacementRegion;
use geometry_core::models::placement_region_instance::{
    PlacementRegionDerived, PlacementTransform,
};

#[derive(Default)]
pub struct PlacementRegionCache {
    derived: HashMap<(usize, u64), PlacementRegionDerived>,
}

impl PlacementRegionCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_compute(
        &mut self,
        region_id: usize,
        region: &PlacementRegion,
        transform: PlacementTransform,
        transform_key: u64,
        sample_count: usize,
        seed: u64,
    ) -> &PlacementRegionDerived {
        self.derived
            .entry((region_id, transform_key))
            .or_insert_with(|| {
                let mut samples = sample_points_area(
                    &region.regions.forbidden_region.mesh,
                    sample_count,
                    seed,
                );
                for p in &mut samples {
                    *p = transform.apply_to_point(*p);
                }
                let hull = convex_hull_xz_points(&samples);
                PlacementRegionDerived { samples, hull }
            })
    }
}

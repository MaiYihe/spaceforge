use geometry_core::models::placement_region_instance::{
    PlacementRegionInstance, PlacementTransform,
};

pub fn build_default_transforms(count: usize) -> Vec<PlacementTransform> {
    let mut transforms = vec![PlacementTransform::identity(); count];
    if let Some(first) = transforms.first_mut() {
        let deg = 20.0f32;
        let rad = deg.to_radians();
        let c = rad.cos();
        let s = rad.sin();
        // Y-axis rotation
        first.matrix = [
            [c, 0.0, -s, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [s, 0.0, c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
    }
    transforms
}

pub fn attach_transforms(instances: &mut [PlacementRegionInstance]) {
    let transforms = build_default_transforms(instances.len());
    for (instance, transform) in instances.iter_mut().zip(transforms.into_iter()) {
        instance.transform = transform;
    }
}

pub fn transform_key(transform: &PlacementTransform) -> u64 {
    let mut acc: u64 = 1469598103934665603;
    for row in &transform.matrix {
        for v in row {
            acc ^= v.to_bits() as u64;
            acc = acc.wrapping_mul(1099511628211);
        }
    }
    acc
}

use bevy::prelude::Vec3;

use geometry_core::models::space::Space;

use super::IndexedTransform;

pub(super) fn compute_bounds(space: &Space) -> (Vec3, Vec3) {
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);

    for surface in &space.surfaces {
        for p in &surface.mesh.positions {
            min.x = min.x.min(p[0]);
            min.y = min.y.min(p[1]);
            min.z = min.z.min(p[2]);
            max.x = max.x.max(p[0]);
            max.y = max.y.max(p[1]);
            max.z = max.z.max(p[2]);
        }
    }

    if !min.x.is_finite() || !max.x.is_finite() {
        (Vec3::ZERO, Vec3::ZERO)
    } else {
        (min, max)
    }
}

pub(super) fn find_transform(
    transforms: &[IndexedTransform],
    index: usize,
) -> Option<&[[f32; 4]; 4]> {
    transforms
        .iter()
        .find(|t| t.index == index)
        .map(|t| &t.matrix)
}

pub(super) fn apply_optional_transform(
    mut positions: Vec<[f32; 3]>,
    transform: Option<&[[f32; 4]; 4]>,
) -> Vec<[f32; 3]> {
    if let Some(m) = transform {
        apply_transform_positions(&mut positions, m);
    }
    positions
}

pub(super) fn apply_transform_positions(positions: &mut [[f32; 3]], m: &[[f32; 4]; 4]) {
    for p in positions.iter_mut() {
        apply_transform_point(p, m);
    }
}

pub(super) fn apply_transform_point(p: &mut [f32; 3], m: &[[f32; 4]; 4]) {
    let x = p[0];
    let y = p[1];
    let z = p[2];
    p[0] = x * m[0][0] + y * m[1][0] + z * m[2][0] + m[3][0];
    p[1] = x * m[0][1] + y * m[1][1] + z * m[2][1] + m[3][1];
    p[2] = x * m[0][2] + y * m[1][2] + z * m[2][2] + m[3][2];
}

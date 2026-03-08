use bevy::prelude::*;

pub fn draw_gizmos(
    mut gizmos: Gizmos,
    boundary: Res<crate::scene::DebugBoundaryPoints>,
    space_boundaries: Res<crate::scene::DebugSpaceBoundaries>,
    nfp: Res<crate::scene::DebugNfpData>,
    ifp: Res<crate::scene::DebugIfpData>,
) {
    let grid_size = 20;
    let step = 500.0;
    for i in -grid_size..=grid_size {
        let p = i as f32 * step;
        gizmos.line(
            Vec3::new(p, 0.0, -grid_size as f32 * step),
            Vec3::new(p, 0.0, grid_size as f32 * step),
            Color::GRAY,
        );
        gizmos.line(
            Vec3::new(-grid_size as f32 * step, 0.0, p),
            Vec3::new(grid_size as f32 * step, 0.0, p),
            Color::GRAY,
        );
    }

    gizmos.line(Vec3::ZERO, Vec3::new(1000.0, 0.0, 0.0), Color::RED);
    gizmos.line(Vec3::ZERO, Vec3::new(0.0, 1000.0, 0.0), Color::GREEN);
    gizmos.line(Vec3::ZERO, Vec3::new(0.0, 0.0, 1000.0), Color::BLUE);

    if boundary.points.len() >= 2 {
        let mut prev = boundary.points[0] + Vec3::new(0.0, boundary.y_offset, 0.0);
        for p in boundary.points.iter().skip(1) {
            let curr = *p + Vec3::new(0.0, boundary.y_offset, 0.0);
            gizmos.line(prev, curr, boundary.color);
            prev = curr;
        }
        let first = boundary.points[0] + Vec3::new(0.0, boundary.y_offset, 0.0);
        gizmos.line(prev, first, boundary.color);
    }

    for loop_points in &space_boundaries.loops {
        if loop_points.len() < 2 {
            continue;
        }
        let mut prev = loop_points[0] + Vec3::new(0.0, space_boundaries.y_offset, 0.0);
        for p in loop_points.iter().skip(1) {
            let curr = *p + Vec3::new(0.0, space_boundaries.y_offset, 0.0);
            gizmos.line(prev, curr, space_boundaries.color);
            prev = curr;
        }
        let first = loop_points[0] + Vec3::new(0.0, space_boundaries.y_offset, 0.0);
        gizmos.line(prev, first, space_boundaries.color);
    }

    for loop_points in &nfp.rect_loops {
        if loop_points.len() < 2 {
            continue;
        }
        let mut prev = loop_points[0];
        for p in loop_points.iter().skip(1) {
            gizmos.line(prev, *p, Color::YELLOW);
            prev = *p;
        }
        gizmos.line(prev, loop_points[0], Color::YELLOW);
    }

    for loop_points in &nfp.nfp_loops {
        if loop_points.len() < 2 {
            continue;
        }
        let mut prev = loop_points[0];
        for p in loop_points.iter().skip(1) {
            gizmos.line(prev, *p, Color::ORANGE);
            prev = *p;
        }
        gizmos.line(prev, loop_points[0], Color::ORANGE);
    }

    for loop_points in &ifp.loops {
        if loop_points.len() < 2 {
            continue;
        }
        let mut prev = loop_points[0];
        for p in loop_points.iter().skip(1) {
            gizmos.line(prev, *p, Color::GREEN);
            prev = *p;
        }
        gizmos.line(prev, loop_points[0], Color::GREEN);
    }

    for loop_points in &ifp.instance_loops {
        if loop_points.len() < 2 {
            continue;
        }
        let mut prev = loop_points[0];
        for p in loop_points.iter().skip(1) {
            gizmos.line(prev, *p, Color::PINK);
            prev = *p;
        }
        gizmos.line(prev, loop_points[0], Color::PINK);
    }
}

pub fn reset_camera_hotkey(
    keys: Res<ButtonInput<KeyCode>>,
    scene: Res<crate::scene::SceneInfo>,
    mut cam: ResMut<crate::camera::OrbitCamera>,
) {
    if (keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight))
        && (keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight))
        && keys.just_pressed(KeyCode::KeyE)
    {
        cam.target = scene.center;
        cam.distance = scene.focus_distance;
    }
}

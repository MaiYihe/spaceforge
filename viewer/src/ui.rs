use bevy::prelude::*;

pub fn draw_gizmos(mut gizmos: Gizmos) {
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

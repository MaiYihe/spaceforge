use bevy::prelude::*;

#[derive(Resource)]
pub struct SceneInfo {
    pub center: Vec3,
    pub focus_distance: f32,
    pub size: Vec3,
}

#[derive(Resource)]
pub struct BBoxEntity(pub Entity);

#[derive(Resource)]
pub struct BBoxTextEntity(pub Entity);

pub fn toggle_bbox(
    keys: Res<ButtonInput<KeyCode>>,
    mut vis_q: Query<&mut Visibility>,
    bbox: Res<BBoxEntity>,
    scene: Res<SceneInfo>,
    mut cam: ResMut<crate::camera::OrbitCamera>,
) {
    if keys.just_pressed(KeyCode::KeyB) {
        if let Ok(mut vis) = vis_q.get_mut(bbox.0) {
            *vis = if *vis == Visibility::Visible {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        }
    }

    if (keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight))
        && (keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight))
        && keys.just_pressed(KeyCode::KeyE)
    {
        cam.target = scene.center;
        cam.distance = scene.focus_distance;
    }
}

pub fn update_bbox_text(
    scene: Res<SceneInfo>,
    bbox: Res<BBoxEntity>,
    text_entity: Res<BBoxTextEntity>,
    mut set: ParamSet<(
        Query<&Visibility>,
        Query<(&mut Text, &mut Visibility)>,
    )>,
) {
    let bbox_visible = set
        .p0()
        .get(bbox.0)
        .map(|v| *v == Visibility::Visible)
        .unwrap_or(false);

    if let Ok((mut text, mut vis)) = set.p1().get_mut(text_entity.0) {
        if bbox_visible {
            text.sections[0].value = format!(
                "OBJ size (mm): x={:.1} y={:.1} z={:.1}",
                scene.size.x, scene.size.y, scene.size.z
            );
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

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

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::CursorMoved;

#[derive(Resource)]
pub struct OrbitCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: Vec3,
}

#[derive(Resource, Default)]
pub struct MouseState {
    pub last_pos: Option<Vec2>,
}

pub fn camera_input(
    mut cam: ResMut<OrbitCamera>,
    mut mouse_state: ResMut<MouseState>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut cursor_events: EventReader<CursorMoved>,
    mut wheel_events: EventReader<MouseWheel>,
) {
    let mut delta = Vec2::ZERO;
    for ev in cursor_events.read() {
        if let Some(last) = mouse_state.last_pos {
            delta += ev.position - last;
        }
        mouse_state.last_pos = Some(ev.position);
    }

    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if delta != Vec2::ZERO && buttons.pressed(MouseButton::Right) && shift {
        let forward = Vec3::new(
            cam.yaw.cos() * cam.pitch.cos(),
            cam.pitch.sin(),
            cam.yaw.sin() * cam.pitch.cos(),
        )
        .normalize();
        let right = forward.cross(Vec3::Y).normalize_or_zero();
        let up = right.cross(forward).normalize_or_zero();
        let pan_speed = cam.distance * 0.002;
        cam.target += (right * delta.x + up * delta.y) * pan_speed;
    } else if delta != Vec2::ZERO && buttons.pressed(MouseButton::Right) {
        cam.yaw += delta.x * 0.01;
        cam.pitch += delta.y * 0.01;
        cam.pitch = cam.pitch.clamp(-1.55, 1.55);
    }

    for ev in wheel_events.read() {
        cam.distance -= ev.y * 50.0;
        cam.distance = cam.distance.clamp(100.0, 200_000.0);
    }
}

#[allow(clippy::type_complexity)]
pub fn camera_sync_system(mut q: Query<&mut Transform, With<Camera3d>>, cam: Res<OrbitCamera>) {
    if let Ok(mut t) = q.get_single_mut() {
        let dir = Vec3::new(
            cam.yaw.cos() * cam.pitch.cos(),
            cam.pitch.sin(),
            cam.yaw.sin() * cam.pitch.cos(),
        );
        t.translation = cam.target + dir * cam.distance;
        t.look_at(cam.target, Vec3::Y);
    }
}

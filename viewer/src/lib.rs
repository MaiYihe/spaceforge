use bevy::prelude::*;

mod camera;
mod config;
mod scene;
mod ui;

pub use config::ViewerConfig;
pub use scene::ScenePayload;

pub fn run() {
    run_with_config(ViewerConfig::default());
}

pub fn run_with_config(config: ViewerConfig) {
    run_with_config_and_rx(config, None);
}

pub fn run_with_config_and_rx(
    config: ViewerConfig,
    rx: Option<crossbeam_channel::Receiver<ScenePayload>>,
) {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(camera::MouseState::default())
        .insert_resource(config)
        .insert_resource(scene::SceneReceiver(rx))
        .add_systems(Startup, scene::setup)
        .add_systems(Startup, scene::init_file_watcher.after(scene::setup))
        .add_systems(Startup, scene::load_scene_from_file.after(scene::setup))
        .add_systems(
            Update,
            (
                camera::camera_input,
                camera::camera_sync_system,
                ui::draw_gizmos,
                ui::reset_camera_hotkey,
                scene::apply_scene_updates,
                scene::apply_file_watch_updates,
            ),
        )
        .run();
}

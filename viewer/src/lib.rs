use bevy::prelude::*;

mod camera;
mod config;
mod scene;
mod ui;

pub use config::ViewerConfig;

pub fn run() {
    run_with_config(ViewerConfig::default());
}

pub fn run_with_config(config: ViewerConfig) {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(camera::MouseState::default())
        .insert_resource(config)
        .add_systems(Startup, scene::setup)
        .add_systems(
            Update,
            (
                camera::camera_input,
                camera::camera_sync_system,
                ui::draw_gizmos,
                ui::toggle_bbox,
                ui::update_bbox_text,
            ),
        )
        .run();
}

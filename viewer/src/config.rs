use bevy::prelude::Resource;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewMode {
    Sdf,
    Voxel,
    Obj,
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::Voxel
    }
}

#[derive(Debug, Clone, Deserialize, Resource)]
pub struct ViewerConfig {
    #[serde(default)]
    pub mode: ViewMode,
    #[serde(default = "default_voxel_size")]
    pub voxel_size: f32,
    #[serde(default = "default_obj_scale")]
    pub obj_scale: f32,
    #[serde(default)]
    pub assets: Vec<PlacementConfig>,
}

impl Default for ViewerConfig {
    fn default() -> Self {
        ViewerConfig {
            mode: ViewMode::Voxel,
            voxel_size: default_voxel_size(),
            obj_scale: default_obj_scale(),
            assets: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlacementConfig {
    pub obj_path: String,
    #[serde(default)]
    pub pose: Pose2D,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Pose2D {
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default)]
    pub theta: f32,
}

impl Default for Pose2D {
    fn default() -> Self {
        Pose2D {
            x: 0.0,
            y: 0.0,
            theta: 0.0,
        }
    }
}

fn default_voxel_size() -> f32 {
    15.0
}

fn default_obj_scale() -> f32 {
    1000.0
}

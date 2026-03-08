use crossbeam_channel::{Receiver, Sender};
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};

use super::FileWatchResource;

pub(super) fn create_file_watcher(
    scene_path: &str,
    transforms_path: &str,
    debug_points_path: &str,
    debug_forbidden_points_path: &str,
    debug_space_boundary_path: &str,
    debug_nfp_path: &str,
    debug_ifp_path: &str,
    debug_boundary_path: &str,
) -> Result<FileWatchResource, String> {
    let (tx, rx): (Sender<notify::Result<notify::Event>>, Receiver<notify::Result<notify::Event>>) =
        crossbeam_channel::unbounded();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        NotifyConfig::default(),
    )
    .map_err(|err| format!("create watcher failed: {err}"))?;
    watcher
        .watch(std::path::Path::new(scene_path), RecursiveMode::NonRecursive)
        .map_err(|err| format!("watch scene.json failed: {err}"))?;
    watcher
        .watch(
            std::path::Path::new(transforms_path),
            RecursiveMode::NonRecursive,
        )
        .map_err(|err| format!("watch transforms.json failed: {err}"))?;
    watcher
        .watch(
            std::path::Path::new(debug_points_path),
            RecursiveMode::NonRecursive,
        )
        .map_err(|err| format!("watch debug_points.json failed: {err}"))?;
    watcher
        .watch(
            std::path::Path::new(debug_forbidden_points_path),
            RecursiveMode::NonRecursive,
        )
        .map_err(|err| format!("watch debug_forbidden_points.json failed: {err}"))?;
    watcher
        .watch(
            std::path::Path::new(debug_space_boundary_path),
            RecursiveMode::NonRecursive,
        )
        .map_err(|err| format!("watch debug_space_boundary.json failed: {err}"))?;
    watcher
        .watch(
            std::path::Path::new(debug_nfp_path),
            RecursiveMode::NonRecursive,
        )
        .map_err(|err| format!("watch debug_nfp.json failed: {err}"))?;
    watcher
        .watch(
            std::path::Path::new(debug_ifp_path),
            RecursiveMode::NonRecursive,
        )
        .map_err(|err| format!("watch debug_ifp.json failed: {err}"))?;
    watcher
        .watch(
            std::path::Path::new(debug_boundary_path),
            RecursiveMode::NonRecursive,
        )
        .map_err(|err| format!("watch debug_boundary.json failed: {err}"))?;

    Ok(FileWatchResource {
        rx,
        _watcher: watcher,
        scene_path: scene_path.to_string(),
        transforms_path: transforms_path.to_string(),
        debug_points_path: debug_points_path.to_string(),
        debug_forbidden_points_path: debug_forbidden_points_path.to_string(),
        debug_space_boundary_path: debug_space_boundary_path.to_string(),
        debug_nfp_path: debug_nfp_path.to_string(),
        debug_ifp_path: debug_ifp_path.to_string(),
        debug_boundary_path: debug_boundary_path.to_string(),
    })
}

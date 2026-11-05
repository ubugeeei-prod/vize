//! A retired backend keeps no prepared project alive.

use crate::batch::virtual_project::live_projects::live_projects;

use super::vue_document::{
    CorsaProjectEnvironment, build_vue_virtual_project_with_overlays_and_options_and_package_routes,
};
use super::{CorsaBridge, CorsaBridgeConfig};

const HOST: &str = "<script setup lang='ts'>import Child from './Child.vue';</script><template><Child /></template>";

#[test]
#[expect(clippy::disallowed_types, reason = "a worker keeps the session by Arc")]
fn a_retired_bridge_releases_its_prepared_projects_while_a_worker_holds_the_session() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    std::fs::write(root.join("tsconfig.json"), r#"{"include":["*.vue"]}"#).unwrap();
    std::fs::write(root.join("Child.vue"), "<template><span /></template>").unwrap();
    let host = root.join("Host.vue");
    std::fs::write(&host, HOST).unwrap();
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        working_dir: Some(root.clone()),
        ..Default::default()
    });
    let prepare = |bridge: &CorsaBridge| {
        build_vue_virtual_project_with_overlays_and_options_and_package_routes(
            &host,
            HOST,
            Default::default(),
            &[(host.clone(), HOST)],
            CorsaProjectEnvironment {
                virtual_ts_options: &Default::default(),
                package_routes: &Default::default(),
                project_root: Some(&root),
                tsconfig_path: None,
                editor_session: &bridge.editor_session,
            },
        )
        .unwrap();
    };
    prepare(&bridge);
    assert_eq!(live_projects(), 1);

    let worker = std::sync::Arc::clone(&bridge.editor_session);
    bridge.release_editor_projects();
    assert_eq!(live_projects(), 0);
    drop(worker);

    // The next request on a live bridge builds its revision again.
    prepare(&bridge);
    assert_eq!(live_projects(), 1);
}

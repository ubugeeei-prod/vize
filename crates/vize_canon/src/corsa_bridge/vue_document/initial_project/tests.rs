//! Real graph construction and worker admission, without a native process.
#![expect(clippy::disallowed_types, reason = "owned request inputs use Arc")]

use std::sync::Arc;

use super::*;
use crate::corsa_bridge::CorsaBridgeConfig;
use crate::virtual_ts::VirtualTsOptions;

fn workspace() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true},"include":["**/*"]}"#,
    )
    .unwrap();
    std::fs::write(
        root.path().join("support.ts"),
        "export const support = 42;\n",
    )
    .unwrap();
    root
}

fn bridge(root: &std::path::Path) -> CorsaBridge {
    CorsaBridge::with_config_and_package_routes(
        CorsaBridgeConfig {
            working_dir: Some(root.to_path_buf()),
            ..Default::default()
        },
        crate::PackageRouteResolver::default(),
    )
}

fn request(root: &std::path::Path) -> CorsaVueWorkspaceRequest {
    CorsaVueWorkspaceRequest {
        source_path: root.join("App.vue"),
        content:
            "<script setup lang=\"ts\">import { support } from './support'; const answer = support;</script><template>{{ answer }}</template>"
                .into(),
        options: Default::default(),
        overlays: Vec::new(),
        virtual_ts_options: VirtualTsOptions::default(),
        requested_sources: Vec::new(),
    }
}

#[test]
fn complete_owned_seed_is_not_ready_and_drop_restores_an_unstarted_revision() {
    let root = workspace();
    let bridge = bridge(root.path());
    let seed = bridge
        .prepare_initial_vue_project(request(root.path()))
        .unwrap()
        .expect("authored project admits a real Canon mirror");
    let canon_root = seed.project.session_project_root.as_ref().unwrap().clone();
    assert_ne!(canon_root, root.path());
    assert!(seed.project.session_config_path.as_ref().unwrap().is_file());
    assert!(!seed.project.documents.is_empty());
    assert!(seed.project.documents.iter().any(|(uri, code)| {
        uri == &seed.project.host.request_uri && code == &seed.project.host.code
    }));
    assert!(!bridge.is_initialized());
    assert_eq!(bridge.pending_startups(), 1);
    assert!(
        bridge
            .prepare_initial_vue_project(request(root.path()))
            .unwrap()
            .is_none()
    );
    drop(seed);
    assert_eq!(bridge.pending_startups(), 0);
    assert!(!canon_root.exists());
    let retried = bridge
        .prepare_initial_vue_project(request(root.path()))
        .unwrap()
        .expect("a failed unstarted admission may build again");
    assert!(!retried.project.materialized_changes.is_empty());
}

#[test]
fn old_independent_and_rootless_constructors_do_not_admit_a_seed() {
    let root = workspace();
    let independent = CorsaBridge::with_config(CorsaBridgeConfig {
        working_dir: Some(root.path().to_path_buf()),
        ..Default::default()
    });
    assert!(
        independent
            .prepare_initial_vue_project(request(root.path()))
            .unwrap()
            .is_none()
    );
    let rootless = CorsaBridge::with_config_and_package_routes(
        CorsaBridgeConfig::default(),
        crate::PackageRouteResolver::default(),
    );
    assert!(
        rootless
            .prepare_initial_vue_project(request(root.path()))
            .unwrap()
            .is_none()
    );
    let no_config = tempfile::tempdir().unwrap();
    let no_config_bridge = bridge(no_config.path());
    assert!(
        no_config_bridge
            .prepare_initial_vue_project(request(no_config.path()))
            .unwrap()
            .is_none()
    );
    assert_eq!(independent.pending_startups(), 0);
    assert_eq!(rootless.pending_startups(), 0);
    assert_eq!(no_config_bridge.pending_startups(), 0);
}

#[test]
fn graph_configuration_error_rolls_back_without_starting_or_consuming_delta() {
    let root = workspace();
    std::fs::write(root.path().join("tsconfig.json"), "{").unwrap();
    let bridge = bridge(root.path());
    let marker = bridge.editor_session.root().unwrap().join("unstarted");
    std::fs::write(&marker, "old preparation").unwrap();
    assert!(matches!(
        bridge.prepare_initial_vue_project(request(root.path())),
        Err(CorsaBridgeError::Configuration(_))
    ));
    assert!(!marker.exists());
    assert!(!bridge.is_initialized());
    assert_eq!(bridge.pending_startups(), 0);
    std::fs::write(
        root.path().join("tsconfig.json"),
        "{\"include\":[\"**/*\"]}",
    )
    .unwrap();
    let repaired = bridge
        .prepare_initial_vue_project(request(root.path()))
        .unwrap()
        .unwrap();
    assert!(!repaired.project.materialized_changes.is_empty());
}

#[test]
fn foreign_bridge_seed_refuses_before_spawn_and_leaves_destination_bytes_exact() {
    let root = workspace();
    let origin = bridge(root.path());
    let destination = bridge(root.path());
    let marker = destination
        .editor_session
        .root()
        .unwrap()
        .join("foreign-owner");
    std::fs::write(&marker, "destination bytes").unwrap();
    let seed = origin
        .prepare_initial_vue_project(request(root.path()))
        .unwrap()
        .unwrap();
    let result = corsa::runtime::block_on(destination.spawn_with_initial_project(seed, || true));
    assert!(matches!(result, Err(CorsaBridgeError::NotInitialized)));
    assert_eq!(
        std::fs::read_to_string(marker).unwrap(),
        "destination bytes"
    );
    assert_eq!(origin.pending_startups(), 0);
    assert_eq!(destination.pending_startups(), 0);
    assert!(!origin.is_initialized());
    assert!(!destination.is_initialized());
}

#[test]
fn explicit_shutdown_keeps_live_seed_files_and_rejects_the_original_epoch() {
    let root = workspace();
    let bridge = bridge(root.path());
    let seed = bridge
        .prepare_initial_vue_project(request(root.path()))
        .unwrap()
        .unwrap();
    let canon_root = seed.project.session_project_root.as_ref().unwrap().clone();
    corsa::runtime::block_on(bridge.shutdown()).unwrap();
    assert!(canon_root.is_dir());
    let result = corsa::runtime::block_on(bridge.spawn_with_initial_project(seed, || true));
    assert!(matches!(result, Err(CorsaBridgeError::ProcessTerminated)));
    assert!(!canon_root.exists());
    assert_eq!(bridge.pending_startups(), 0);
    assert!(!bridge.is_initialized());
}

#[test]
fn rejected_original_environment_never_spawns_and_rolls_back_its_own_seed() {
    let root = workspace();
    let bridge = bridge(root.path());
    let seed = bridge
        .prepare_initial_vue_project(request(root.path()))
        .unwrap()
        .unwrap();
    let canon_root = seed.project.session_project_root.as_ref().unwrap().clone();
    let current = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let predicate = Arc::clone(&current);
    let result = corsa::runtime::block_on(bridge.spawn_with_initial_project(seed, move || {
        predicate.load(std::sync::atomic::Ordering::SeqCst)
    }));
    assert!(matches!(result, Err(CorsaBridgeError::ProcessTerminated)));
    assert!(!canon_root.exists());
    assert_eq!(bridge.pending_startups(), 0);
    assert!(!bridge.is_initialized());
}

#[test]
fn receipt_revalidates_whole_ordered_inputs_catalog_config_and_real_delta() {
    let root = workspace();
    let bridge = bridge(root.path());
    let seed = bridge
        .prepare_initial_vue_project(request(root.path()))
        .unwrap()
        .unwrap();
    let CorsaInitialVueProject {
        project,
        lease,
        config_bytes,
    } = seed;
    let receipt = CorsaInitialVueDocument {
        project,
        owner: lease.receipt(),
        operation_epoch: bridge
            .client_operation_epoch
            .load(std::sync::atomic::Ordering::SeqCst),
        config_bytes,
    };
    let mut current = bridge
        .build_owned_initial_request(&request(root.path()))
        .unwrap();
    let canon_root = current.session_project_root.as_deref().unwrap();
    current
        .host
        .source_catalog
        .extend_native_project_documents(canon_root, &mut current.documents);
    let bytes = read_config(&current).unwrap();
    assert!(receipt.matches(&current, &bytes));
    assert!(
        !receipt.owner.is_current(&bridge),
        "a graph receipt is not a native handshake"
    );

    let original = current.documents[0].1.clone();
    current.documents[0].1.push_str("\n// changed input");
    assert!(!receipt.matches(&current, &bytes));
    current.documents[0].1 = original;
    current
        .documents
        .push(("file:///unadmitted.ts".into(), "export {}".into()));
    assert!(!receipt.matches(&current, &bytes));
    current.documents.pop();
    assert!(
        current.documents.len() > 1,
        "closed support is part of the native graph"
    );
    current.documents.swap(0, 1);
    assert!(!receipt.matches(&current, &bytes));
    current.documents.swap(0, 1);
    let old_config = current.session_config_path.take();
    assert!(!receipt.matches(&current, &bytes));
    current.session_config_path = old_config;
    let old_root = current.session_project_root.take();
    assert!(!receipt.matches(&current, &bytes));
    current.session_project_root = old_root;
    assert!(!receipt.matches(&current, b"foreign config bytes"));
    let old_catalog = current.host.source_catalog.clone();
    current.host.source_catalog = Default::default();
    assert!(!receipt.matches(&current, &bytes));
    current.host.source_catalog = old_catalog;
    current.materialized_changes = Default::default();
    current
        .materialized_changes
        .changed
        .push(root.path().join("package.json"));
    assert!(!receipt.matches(&current, &bytes));
    current.materialized_changes = Default::default();
    current.materialized_changes.topology_changed = true;
    assert!(!receipt.matches(&current, &bytes));
    drop(lease);
}

#[test]
fn sticky_failure_keeps_actual_canon_generated_file_after_later_success_and_bridge_drop() {
    let root = workspace();
    let bridge = bridge(root.path());
    let seed = bridge
        .prepare_initial_vue_project(request(root.path()))
        .unwrap()
        .unwrap();
    let generated = crate::file_uri::file_uri_to_path(&seed.project.host.request_uri).unwrap();
    let expected = seed.project.host.code.clone();
    let owner = bridge.editor_session.root().unwrap().to_path_buf();
    assert_eq!(
        std::fs::read_to_string(&generated).unwrap(),
        expected.as_str()
    );
    // Models the storage consequence of an entered startup/retirement error;
    // no native executable is run by this ownership control.
    bridge
        .editor_session
        .arm_retention_before_startup()
        .unwrap();
    bridge.editor_session.retain_after_failed_shutdown();
    corsa::runtime::block_on(bridge.shutdown()).unwrap();
    drop(seed);
    bridge.editor_session.clear();
    drop(bridge);
    assert_eq!(
        std::fs::read_to_string(&generated).unwrap(),
        expected.as_str()
    );
    assert!(owner.join(".retained-after-failed-shutdown").is_file());
    std::fs::remove_dir_all(owner).unwrap();
}

#[test]
fn missing_private_config_disables_receipt_reuse_without_replacing_the_normal_error_route() {
    let root = workspace();
    let bridge = bridge(root.path());
    let seed = bridge
        .prepare_initial_vue_project(request(root.path()))
        .unwrap()
        .unwrap();
    let CorsaInitialVueProject {
        project,
        lease,
        config_bytes,
    } = seed;
    let receipt = CorsaInitialVueDocument {
        project,
        owner: lease.receipt(),
        operation_epoch: 0,
        config_bytes,
    };
    let current = bridge
        .build_owned_initial_request(&request(root.path()))
        .unwrap();
    let path = current.session_config_path.as_ref().unwrap();
    std::fs::remove_file(path).unwrap();
    let unreadable = read_config(&current);
    assert!(unreadable.is_err(), "actual missing private config");
    assert!(!receipt.matches_config_read(&current, unreadable));
    drop(lease);
}

#[test]
fn seeded_marker_failure_rolls_back_before_physical_startup_without_sticky_retention() {
    let root = workspace();
    let bridge = bridge(root.path());
    let seed = bridge
        .prepare_initial_vue_project(request(root.path()))
        .unwrap()
        .unwrap();
    let generated = crate::file_uri::file_uri_to_path(&seed.project.host.request_uri).unwrap();
    let owner = bridge.editor_session.root().unwrap().to_path_buf();
    assert!(generated.is_file());
    std::fs::create_dir(owner.join(".retained-after-failed-shutdown")).unwrap();
    let expected = bridge
        .editor_session
        .arm_retention_before_startup()
        .unwrap_err();
    let result = corsa::runtime::block_on(bridge.spawn_with_initial_project(seed, || true));
    let actual = match result {
        Err(error) => error,
        Ok(_) => panic!("unarmed seed must not enter its native factory"),
    };
    assert_eq!(actual.to_string(), expected.to_string());
    assert!(!bridge.is_initialized());
    assert_eq!(bridge.pending_startups(), 0);
    assert!(!generated.exists());
    assert!(!owner.join(".retained-after-failed-shutdown").exists());
    drop(bridge);
    assert!(!owner.exists());
}

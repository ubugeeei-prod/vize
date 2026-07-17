use std::sync::atomic::Ordering;

use tower_lsp::lsp_types::Url;

use super::ServerState;

const DECLARATION: &str =
    "declare module 'vue' { interface GlobalComponents { NuxtCard: unknown } }";

#[test]
fn declaration_file_events_refresh_created_renamed_and_deleted_paths() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join(".nuxt")).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    assert!(crate::runtime::block_on(state.global_component_reference_paths()).is_empty());

    let created = root.path().join(".nuxt/components.d.ts");
    std::fs::write(&created, DECLARATION).unwrap();
    let created_uri = Url::from_file_path(&created).unwrap();
    assert!(state.invalidate_global_component_references([created_uri.as_str()]));
    assert_eq!(
        crate::runtime::block_on(state.global_component_reference_paths()),
        [std::fs::canonicalize(&created).unwrap()]
    );

    let renamed = root.path().join(".nuxt/components.d.mts");
    std::fs::rename(&created, &renamed).unwrap();
    let renamed_uri = Url::from_file_path(&renamed).unwrap();
    assert!(
        state.invalidate_global_component_references([created_uri.as_str(), renamed_uri.as_str(),])
    );
    assert_eq!(
        crate::runtime::block_on(state.global_component_reference_paths()),
        [std::fs::canonicalize(&renamed).unwrap()]
    );

    std::fs::remove_file(&renamed).unwrap();
    assert!(state.invalidate_global_component_references([renamed_uri.as_str()]));
    assert!(crate::runtime::block_on(state.global_component_reference_paths()).is_empty());
    assert!(!state.invalidate_global_component_references(["file:///workspace/App.vue"]));
    assert_eq!(
        state
            .global_component_references
            .scan_count
            .load(Ordering::Acquire),
        4
    );
}

#[test]
fn excluded_tree_events_do_not_restart_discovery_but_nuxt_events_do() {
    let root = tempfile::tempdir().unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    assert!(crate::runtime::block_on(state.global_component_reference_paths()).is_empty());

    for directory in [".git", "node_modules", "target", "coverage", "dist"] {
        let uri = Url::from_file_path(root.path().join(directory).join("components.d.ts")).unwrap();
        assert!(
            !state.invalidate_global_component_references([uri.as_str()]),
            "{directory} events must not invalidate discovery"
        );
    }
    let outside = Url::from_file_path(root.path().parent().unwrap().join("outside.d.ts")).unwrap();
    assert!(!state.invalidate_global_component_references([outside.as_str()]));
    assert!(crate::runtime::block_on(state.global_component_reference_paths()).is_empty());
    assert_eq!(
        state
            .global_component_references
            .scan_count
            .load(Ordering::Acquire),
        1
    );

    let nuxt = root.path().join(".nuxt/components.d.cts");
    std::fs::create_dir_all(nuxt.parent().unwrap()).unwrap();
    std::fs::write(&nuxt, DECLARATION).unwrap();
    let nuxt_uri = Url::from_file_path(&nuxt).unwrap();
    assert!(state.invalidate_global_component_references([nuxt_uri.as_str()]));
    assert_eq!(
        crate::runtime::block_on(state.global_component_reference_paths()),
        [std::fs::canonicalize(nuxt).unwrap()]
    );
    assert_eq!(
        state
            .global_component_references
            .scan_count
            .load(Ordering::Acquire),
        2
    );
}

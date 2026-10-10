//! Package isolation, precedence, cache reuse, and watcher generations.
use super::*;
use tower_lsp::lsp_types::{FileChangeType, FileEvent};
use vize_l0::cstr;

fn fixture() -> (tempfile::TempDir, Arc<ServerState>, Url, Url) {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    std::fs::write(
        root.join("package.json"),
        r#"{"workspaces":["packages/*"]}"#,
    )
    .unwrap();
    for (package, hover) in [("a", false), ("b", true)] {
        let path = root.join("packages").join(package);
        std::fs::create_dir_all(path.join("src")).unwrap();
        std::fs::write(path.join("package.json"), "{}").unwrap();
        std::fs::write(
            path.join("vite.config.mjs"),
            cstr!("export default {{vize:{{lsp:{{hover:{hover}}}}}}};").as_bytes(),
        )
        .unwrap();
    }
    let state = Arc::new(ServerState::new());
    state.set_workspace_root(root.to_path_buf());
    state.set_workspace_folders(vec![root.to_path_buf()]);
    let a = Url::from_file_path(root.join("packages/a/src/App.vue")).unwrap();
    let b = Url::from_file_path(root.join("packages/b/src/App.vue")).unwrap();
    (fixture, state, a, b)
}

#[test]
fn closest_package_owns_configuration_and_reuses_its_context() {
    let (_fixture, state, a, b) = fixture();
    let a_state = state.document_project_state(&a).unwrap();
    let b_state = state.document_project_state(&b).unwrap();
    assert!(!a_state.lsp_features().hover);
    assert!(b_state.lsp_features().hover);
    assert!(Arc::ptr_eq(
        &a_state,
        &state.document_project_state(&a).unwrap()
    ));
    assert!(Arc::ptr_eq(
        &b_state,
        &a_state.document_project_state(&b).unwrap()
    ));
    assert_eq!(state.cached_project_states().len(), 2);
    state.documents.open(
        a.clone(),
        "<template>Hello</template>".into(),
        1,
        "vue".into(),
    );
    assert_eq!(a_state.documents.version(&a), Some(1));
    assert_eq!(b_state.documents.version(&a), Some(1));
}

#[test]
fn initialization_options_override_each_project_and_capability_union() {
    let (_fixture, state, a, b) = fixture();
    state.apply_lsp_initialization_options(Some(
        &serde_json::json!({"hover":false,"formatting":false}),
    ));
    assert!(
        !state
            .document_project_state(&a)
            .unwrap()
            .lsp_features()
            .hover
    );
    assert!(
        !state
            .document_project_state(&b)
            .unwrap()
            .lsp_features()
            .hover
    );
    let capabilities = state.project_capability_features();
    assert!(!capabilities.hover);
    assert!(!capabilities.formatting);
    assert_eq!(state.cached_project_states().len(), 2);
}

#[test]
fn configuration_change_retires_only_the_affected_cached_project() {
    let (fixture, state, a, b) = fixture();
    let old_a = state.document_project_state(&a).unwrap();
    let old_b = state.document_project_state(&b).unwrap();
    let before_reload = old_a.corsa_request_stamp();
    let config = fixture.path().join("packages/a/vite.config.mjs");
    std::fs::write(&config, "export default {vize:{lsp:{hover:true}}};").unwrap();
    state.observe_project_config_events(&[FileEvent {
        uri: Url::from_file_path(config).unwrap(),
        typ: FileChangeType::CHANGED,
    }]);
    let new_a = state.document_project_state(&a).unwrap();
    assert!(!before_reload.is_current(&old_a));
    assert!(!old_a.corsa_request_stamp().is_current(&old_a));
    assert!(!Arc::ptr_eq(&old_a, &new_a));
    assert!(new_a.lsp_features().hover);
    assert!(Arc::ptr_eq(
        &old_b,
        &state.document_project_state(&b).unwrap()
    ));
}

#[test]
fn capabilities_do_not_instantiate_unused_package_contexts() {
    let (_fixture, state, _a, _b) = fixture();
    assert!(state.project_capability_features().formatting);
    assert!(state.cached_project_states().is_empty());
}

#[test]
fn commented_reference_manifests_advertise_the_union_without_loading_projects() {
    let (fixture, state, _a, _b) = fixture();
    std::fs::write(fixture.path().join("package.json"), "{}").unwrap();
    std::fs::write(
        fixture.path().join("tsconfig.json"),
        r#"{ // package references
        "references": [{"path":"packages/a"},],
    }"#,
    )
    .unwrap();
    assert!(state.project_capability_features().formatting);
    assert!(state.cached_project_states().is_empty());
}

#[test]
fn tsconfig_files_inside_source_folders_select_the_nearest_owner() {
    let (fixture, state, a, _b) = fixture();
    let source = fixture.path().join("packages/a/src");
    std::fs::write(source.join("tsconfig.json"), "{}").unwrap();
    let context = state.document_project_state(&a).unwrap();
    assert_eq!(context.get_workspace_root(), Some(source));
    assert!(context.lsp_features().hover);
}

#[test]
fn vite_source_root_honors_host_projected_checker_paths() {
    let (fixture, state, a, _b) = fixture();
    let package = fixture.path().join("packages/a");
    std::fs::write(
        package.join("vite.config.mjs"),
        "export default {root:'src',vize:{typeChecker:{tsconfig:'../config/tsconfig.app.json'}}};",
    )
    .unwrap();
    let context = state.document_project_state(&a).unwrap();
    assert_eq!(
        context.get_workspace_root(),
        Some(package.join("src").canonicalize().unwrap())
    );
    let checker = context.get_type_checker_config();
    assert_eq!(
        checker.tsconfig.as_deref(),
        Some(
            package
                .canonicalize()
                .unwrap()
                .join("config/tsconfig.app.json")
                .to_str()
                .unwrap()
        )
    );
}

#[test]
fn dedicated_package_policy_remains_opt_in_and_explicit_options_still_win() {
    let (project, state, a, _b) = fixture();
    std::fs::write(
        project.path().join("packages/a/vize.config.json"),
        r#"{"languageServer":{"hover":true}}"#,
    )
    .unwrap();
    let context = state.document_project_state(&a).unwrap();
    assert!(context.lsp_features().hover);
    assert!(!context.lsp_features().formatting);
    let (_fixture, state, a, _b) = fixture();
    state.apply_lsp_initialization_options(Some(&serde_json::json!({"hover":true})));
    assert!(
        state
            .document_project_state(&a)
            .unwrap()
            .lsp_features()
            .hover
    );
}

#[test]
fn removing_a_config_reloads_project_defaults_without_sibling_policy() {
    let (fixture, state, a, b) = fixture();
    let before = state.document_project_state(&a).unwrap();
    let sibling = state.document_project_state(&b).unwrap();
    let config = fixture.path().join("packages/a/vite.config.mjs");
    std::fs::remove_file(&config).unwrap();
    state.observe_project_config_events(&[FileEvent {
        uri: Url::from_file_path(config).unwrap(),
        typ: FileChangeType::DELETED,
    }]);
    let after = state.document_project_state(&a).unwrap();
    assert!(after.lsp_features().hover);
    assert!(before.project_context_retired());
    assert!(Arc::ptr_eq(
        &sibling,
        &state.document_project_state(&b).unwrap()
    ));
}

#[test]
fn pnpm_workspaces_advertise_nested_providers_without_eager_initialization() {
    let (fixture, state, _a, _b) = fixture();
    std::fs::write(fixture.path().join("package.json"), "{}").unwrap();
    std::fs::write(
        fixture.path().join("pnpm-workspace.yaml"),
        "packages:\n  - packages/*\n",
    )
    .unwrap();
    assert!(state.project_capability_features().formatting);
    assert!(state.cached_project_states().is_empty());
}

#[test]
fn retired_initialization_cells_never_admit_old_native_owners() {
    let (fixture, state, _a, _b) = fixture();
    let context = registry::ProjectContext::new(fixture.path().join("packages/a"));
    context.retire();
    assert!(context.initialize(&state).is_none());
    assert!(context.initialized_state().is_none());
}

#[test]
fn reverse_importer_discovery_crosses_initialized_package_owners() {
    let (fixture, state, a, b) = fixture();
    let dependency = fixture.path().join("packages/b/src/Shared.ts");
    std::fs::write(&dependency, "export const count = 1;").unwrap();
    let source = "<script setup lang=\"ts\">import { count } from '../../b/src/Shared';</script>";
    let importer = state.document_project_state(&a).unwrap();
    state
        .documents
        .open(a.clone(), source.into(), 1, "vue".into());
    importer.update_virtual_docs(&a, source);
    let dependency_owner = state.document_project_state(&b).unwrap();
    let uri = Url::from_file_path(dependency).unwrap();
    assert_eq!(
        state.project_open_typecheck_dependents(&uri).as_slice(),
        std::slice::from_ref(&a)
    );
    assert_eq!(
        dependency_owner.project_open_typecheck_dependents(&uri),
        [a]
    );
}

#[test]
fn a_pending_config_import_never_locks_siblings_or_admits_a_retired_generation() {
    use std::time::{Duration, Instant};
    let (fixture, state, a, b) = fixture();
    let package = fixture.path().join("packages/a");
    let config = package.join("vite.config.mjs");
    std::fs::write(&config, "import {writeFileSync, existsSync} from 'node:fs';\nwriteFileSync(new URL('./started', import.meta.url), '1');\nwhile (!existsSync(new URL('./release', import.meta.url))) await new Promise(r => setTimeout(r, 10));\nexport default {vize:{lsp:{hover:false}}};").unwrap();
    let a_owner = state.clone();
    let a_worker = std::thread::spawn(move || a_owner.document_project_state(&a).unwrap());
    let deadline = Instant::now() + Duration::from_secs(5);
    while !package.join("started").exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    let started = package.join("started").exists();
    if !started {
        std::fs::write(package.join("release"), "1").unwrap();
    }
    assert!(
        started,
        "Node did not reach the fixture initialization latch"
    );
    let (sender, receiver) = std::sync::mpsc::channel();
    let b_owner = state.clone();
    let b_worker =
        std::thread::spawn(move || sender.send(b_owner.document_project_state(&b)).unwrap());
    let sibling = receiver.recv_timeout(Duration::from_secs(3));
    if sibling.is_err() {
        std::fs::write(package.join("release"), "1").unwrap();
    }
    assert!(sibling.unwrap().unwrap().lsp_features().hover);
    b_worker.join().unwrap();
    let pending = state
        .project_contexts
        .contexts
        .lock()
        .get(&package)
        .unwrap()
        .clone();
    std::fs::write(&config, "export default {vize:{lsp:{hover:true}}};").unwrap();
    state.observe_project_config_events(&[FileEvent {
        uri: Url::from_file_path(config).unwrap(),
        typ: FileChangeType::CHANGED,
    }]);
    std::fs::write(package.join("release"), "1").unwrap();
    assert!(a_worker.join().unwrap().lsp_features().hover);
    assert!(pending.initialized_state().is_none());
    assert_eq!(state.cached_project_states().len(), 2);
}

#[test]
fn initializing_multiple_editor_folders_does_not_import_unused_packages() {
    use tower_lsp::lsp_types::WorkspaceFolder;
    let (fixture, state, a, _b) = fixture();
    let folders = ["a", "b"].map(|name| {
        let package = fixture.path().join("packages").join(name);
        std::fs::write(package.join("vite.config.mjs"), "import {appendFileSync} from 'node:fs'; appendFileSync(new URL('./evaluations', import.meta.url), '1'); export default {vize:{lsp:{hover:true}}};").unwrap();
        WorkspaceFolder { uri: Url::from_file_path(package).unwrap(), name: name.into() }
    });
    state.load_workspace_config(fixture.path());
    state.apply_initialize_workspace_folders(Some(&folders), Some(fixture.path()));
    for name in ["a", "b"] {
        assert!(
            !fixture
                .path()
                .join("packages")
                .join(name)
                .join("evaluations")
                .exists()
        );
    }
    state.document_project_state(&a).unwrap();
    assert_eq!(
        std::fs::read_to_string(fixture.path().join("packages/a/evaluations")).unwrap(),
        "1"
    );
    assert!(!fixture.path().join("packages/b/evaluations").exists());
    assert_eq!(state.cached_project_states().len(), 1);
}

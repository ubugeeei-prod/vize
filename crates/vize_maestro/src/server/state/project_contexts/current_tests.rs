//! Initial continuation retains complete physical, source, and owner guards.
use super::{Arc, InitialConfigurationContinuation, ServerState};
use tower_lsp::lsp_types::{FileChangeType, FileEvent, Url};

#[test]
fn continuation_union_keeps_common_disjoint_absent_and_conflicting_inputs_in_both_orders() {
    use super::super::{ProjectContextKey, freshness::ConfigurationObservation};
    use vize_canon::{CompilerConfigurationSnapshot, ConfigurationObservations};

    let fixture = crate::test_support::tempdir().unwrap();
    let root = super::super::paths::physical_path(fixture.path());
    let common = root.join("policy.mjs");
    let absent = root.join("optional.mjs");
    let a = root.join("a.json");
    let b = root.join("b.json");
    for path in [&common, &a, &b] {
        std::fs::write(path, "{}").unwrap();
    }
    let owner = Arc::new(ServerState::new());
    owner.set_workspace_root(root.clone());
    owner.load_workspace_config(&root);
    let mut raw = owner.project_contexts.loaded_config.read().clone().unwrap();
    raw.watch_dependencies = vec![common.clone(), absent.clone()];
    let before = ConfigurationObservations::capture(&raw.watch_dependencies);
    let mut records = [&a, &b].map(|leaf| {
        let compiler = CompilerConfigurationSnapshot::capture(&root, Some(leaf)).unwrap();
        let record =
            ConfigurationObservation::checked(&raw, &raw, &before, compiler.observations(), None);
        (
            ProjectContextKey {
                root: root.clone(),
                tsconfig: Some(leaf.clone()),
            },
            record,
        )
    });
    let expected = InitialConfigurationContinuation::expected_inputs(&records);
    assert!(expected.changed_paths().is_empty());
    for path in [&common, &absent, &a, &b] {
        assert!(expected.paths().contains(path));
    }
    let modified = std::fs::metadata(&common).unwrap().modified().unwrap();
    std::fs::write(&common, "[]").unwrap();
    std::fs::File::open(&common)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .unwrap();
    std::fs::write(&b, "{ }").unwrap();
    std::fs::write(&absent, "{}").unwrap();
    let mut changed = vec![common.clone(), absent.clone(), b.clone()];
    changed.sort();
    assert_eq!(expected.changed_paths(), changed);
    records.reverse();
    assert_eq!(
        InitialConfigurationContinuation::expected_inputs(&records).changed_paths(),
        changed
    );
    let compiler = CompilerConfigurationSnapshot::capture(&root, Some(&b)).unwrap();
    let current = ConfigurationObservation::checked(
        &raw,
        &raw,
        &ConfigurationObservations::capture(&raw.watch_dependencies),
        compiler.observations(),
        None,
    );
    assert!(current.changes().is_empty());
    records[0].1 = current;
    changed = vec![common, absent];
    changed.sort();
    // A freshly captured record must not seed current values for a retained
    // old revision, whichever record is encountered first in the union.
    assert_eq!(
        InitialConfigurationContinuation::expected_inputs(&records).changed_paths(),
        changed
    );
    records.reverse();
    assert_eq!(
        InitialConfigurationContinuation::expected_inputs(&records).changed_paths(),
        changed
    );
}

#[test]
fn initial_continuation_rechecks_physical_and_queue_identity_after_its_first_read() {
    #[derive(Clone, Copy, Debug)]
    enum Mutation {
        Config,
        SameTime,
        #[cfg(unix)]
        Symlink,
        Source,
        Routing,
        Replacement,
    }
    #[cfg(unix)]
    use Mutation::Symlink;
    use Mutation::{Config, Replacement, Routing, SameTime, Source};

    for control in [
        Config,
        SameTime,
        #[cfg(unix)]
        Symlink,
        Source,
        Routing,
        Replacement,
    ] {
        let directory = crate::test_support::tempdir().unwrap();
        let config = directory.path().join("vite.config.mjs");
        std::fs::write(&config, "export default {vue:{}};").unwrap();
        #[cfg(unix)]
        if matches!(control, Symlink) {
            let target = config.with_file_name("original.mjs");
            std::fs::write(&target, "export default {vue:{}};").unwrap();
            std::fs::remove_file(&config).unwrap();
            std::os::unix::fs::symlink(target, &config).unwrap();
        }
        let owner = Arc::new(ServerState::new());
        owner.set_workspace_root(directory.path().to_path_buf());
        owner.load_workspace_config(directory.path());
        let uri = Url::from_file_path(directory.path().join("App.vue")).unwrap();
        owner
            .documents
            .open(uri.clone(), "<template />".into(), 1, "vue".into());
        let current = owner
            .document_project_state(&uri)
            .unwrap_or_else(|| owner.clone());
        assert!(Arc::ptr_eq(&current, &owner));
        assert!(owner.project_contexts.contexts.lock().is_empty());
        let receipt = current.initial_configuration_continuation(&uri, 1).unwrap();
        assert!(receipt.owns_current_state(&current, &uri, 1));
        assert!(!receipt.owns_current_state(&current, &uri, 2));
        let other_owner = Arc::new(ServerState::new());
        assert!(!receipt.owns_current_state(&other_owner, &uri, 1));
        assert!(receipt.validate(&owner, &uri, 1).is_some());
        let changed = owner.clone();
        let changed_uri = uri.clone();
        let key = owner
            .project_contexts
            .routes
            .get(&uri)
            .unwrap()
            .value()
            .clone();
        super::super::lock_order_tests::at_initial_continuation(move || match control {
            Config => std::fs::write(config, "export default {vue:{name:'changed'}};").unwrap(),
            SameTime => {
                let modified = std::fs::metadata(&config).unwrap().modified().unwrap();
                std::fs::write(&config, "export default {vux:{}};").unwrap();
                std::fs::File::open(&config)
                    .unwrap()
                    .set_times(std::fs::FileTimes::new().set_modified(modified))
                    .unwrap();
                assert_eq!(
                    std::fs::metadata(config).unwrap().modified().unwrap(),
                    modified
                );
            }
            #[cfg(unix)]
            Symlink => {
                let target = config.with_file_name("replacement.mjs");
                std::fs::write(&target, "export default {vue:{}};").unwrap();
                std::fs::remove_file(&config).unwrap();
                std::os::unix::fs::symlink(target, config).unwrap();
            }
            Source => changed.documents.open(
                changed_uri,
                "<template>changed</template>".into(),
                1,
                "vue".into(),
            ),
            Routing => {
                let _change = changed.project_routing_change();
            }
            Replacement => {
                changed.project_contexts.contexts.lock().insert(
                    key.clone(),
                    Arc::new(super::super::registry::ProjectContext::new(key)),
                );
            }
        });
        assert!(receipt.validate(&owner, &uri, 1).is_none(), "{control:?}");
    }
}

#[test]
fn primary_replacement_is_lazy_unique_and_respects_explicit_options() {
    let root = crate::test_support::tempdir().unwrap();
    let config = root.path().join("vite.config.mjs");
    std::fs::write(
        &config,
        r#"export default { vue: {"languageServer":{"fileRename":false}} };"#,
    )
    .unwrap();
    let owner = Arc::new(ServerState::new());
    owner.set_workspace_root(root.path().to_path_buf());
    owner.load_workspace_config(root.path());
    owner.apply_lsp_initialization_options(Some(&serde_json::json!({"workspaceSymbols":false})));
    assert!(Arc::ptr_eq(&owner, &owner.current_primary_project_state()));
    let generation = owner.project_routing_generation();
    std::fs::write(
        &config,
        r#"export default { vue: {"languageServer":{"fileRename":true,"workspaceSymbols":true}} };"#,
    )
    .unwrap();
    owner.observe_project_config_events(&[FileEvent {
        uri: Url::from_file_path(&config).unwrap(),
        typ: FileChangeType::CHANGED,
    }]);
    assert!(owner.project_routing_generation() > generation);
    assert!(owner.cached_project_states().is_empty());
    let current = owner.current_primary_project_state();
    assert!(!Arc::ptr_eq(&owner, &current));
    assert!(!current.project_context_retired());
    assert!(current.lsp_features().file_rename);
    assert!(!current.lsp_features().workspace_symbols);
    assert!(Arc::ptr_eq(
        &current,
        &owner.current_primary_project_state()
    ));
    let states = owner.current_project_states();
    assert_eq!(states.len(), 1);
    assert!(Arc::ptr_eq(&states[0], &current));
    owner.retire_project_contexts();
    assert!(owner.cached_project_states().is_empty());
    assert!(Arc::ptr_eq(&owner, &owner.current_primary_project_state()));
}

#[test]
fn initial_api_entry_keeps_all_diagnostic_providers_off_even_when_native_typecheck_is_enabled() {
    use crate::server::MaestroServer;
    let directory = crate::test_support::tempdir().unwrap();
    std::fs::write(directory.path().join("vite.config.mjs"), "export default {vue:{typeChecker:{corsaPath:'./vize-must-not-start-for-disabled-providers'}}};").unwrap();
    let owner = Arc::new(ServerState::new());
    owner.set_workspace_root(directory.path().to_path_buf());
    owner.load_workspace_config(directory.path());
    let uri = Url::from_file_path(directory.path().join("App.vue")).unwrap();
    owner.documents.open(
        uri.clone(),
        "<script setup lang=\"ts\">const x: string = 1;</script>".into(),
        1,
        "vue".into(),
    );
    let current = owner
        .document_project_state(&uri)
        .unwrap_or_else(|| owner.clone());
    let receipt = current.initial_configuration_continuation(&uri, 1).unwrap();
    {
        let mut features = current.lsp_features.write();
        features.lint = false;
        features.typecheck = false;
        features.ecosystem = false;
    }
    current
        .lsp_typecheck_enabled
        .store(true, std::sync::atomic::Ordering::SeqCst);
    assert!(current.is_lsp_typecheck_enabled());
    assert!(!current.lsp_features().has_diagnostics());
    assert!(receipt.owns_current_state(&current, &uri, 1));
    let (service, _socket) = tower_lsp::LspService::new(MaestroServer::new);
    let server = service.inner().with_project_state(current.clone());
    futures::executor::block_on(server.publish_initial_api_diagnostics_if_version(
        &uri,
        1,
        Some(&receipt),
    ));
    assert!(!current.has_corsa_bridge());
    assert!(
        current.corsa_init_failure().is_none(),
        "disabled providers must not even attempt the configured missing backend"
    );
    assert!(
        !current
            .corsa_init_failed
            .load(std::sync::atomic::Ordering::SeqCst)
    );
}

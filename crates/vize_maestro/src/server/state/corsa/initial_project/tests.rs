use super::*;
use vize_canon::{CorsaBridgeConfig, CorsaBridgeError};

#[test]
fn source_admission_uses_real_resident_identity_and_original_environment_generation() {
    let state = Arc::new(ServerState::new());
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    state
        .documents
        .open(uri.clone(), "<template />".into(), 1, "vue".into());
    let source = state.documents.text_snapshot(&uri).unwrap();
    let generation = state.corsa_environment_revision.load(Ordering::Acquire);
    let input = InitialCanonicalSource {
        owner: &state,
        uri: &uri,
        source: &source,
    };
    assert!(input.is_current(&state, generation));
    let copied = Arc::new(source.as_str().to_owned());
    let copied_input = InitialCanonicalSource {
        owner: &state,
        uri: &uri,
        source: &copied,
    };
    assert!(!copied_input.is_current(&state, generation));
    state.documents.close(&uri);
    state
        .documents
        .open(uri.clone(), "<template />".into(), 1, "vue".into());
    assert!(!input.is_current(&state, generation));
    let current = state.documents.text_snapshot(&uri).unwrap();
    let input = InitialCanonicalSource {
        owner: &state,
        uri: &uri,
        source: &current,
    };
    assert!(input.is_current(&state, generation));
    state
        .corsa_environment_revision
        .fetch_add(1, Ordering::Release);
    assert!(!input.is_current(&state, generation));
}

#[test]
fn disabled_and_retired_owners_never_start_a_canonical_or_eager_backend() {
    for control in ["disabled", "retired"] {
        let state = Arc::new(ServerState::new());
        let uri = Url::parse("file:///workspace/App.vue").unwrap();
        state
            .documents
            .open(uri.clone(), "<template />".into(), 1, "vue".into());
        let source = state.documents.text_snapshot(&uri).unwrap();
        if control == "disabled" {
            state.apply_lsp_initialization_options(Some(&serde_json::json!({"typecheck":false})));
        } else {
            state.retire_project_contexts();
        }
        let (bridge, receipt) =
            crate::runtime::block_on(state.get_corsa_bridge_for_canonical_source(&uri, &source));
        assert!(bridge.is_none() && receipt.is_none(), "{control}");
        assert!(state.corsa_init_failure().is_none(), "{control}");
        assert!(state.corsa_bridge.read().is_none(), "{control}");
    }
}

#[test]
fn outer_shutdown_fences_an_owned_unentered_seed_without_starting_a_provider() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("App.vue");
    let source = "<script setup lang=\"ts\">const n = 1</script><template>{{ n }}</template>";
    std::fs::write(&path, source).unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true}}"#,
    )
    .unwrap();
    let state = Arc::new(ServerState::new());
    state.set_workspace_root(root.path().to_path_buf());
    let bridge = Arc::new(CorsaBridge::with_config_and_package_routes(
        CorsaBridgeConfig {
            corsa_path: Some(root.path().join("never-created-runtime")),
            working_dir: Some(root.path().to_path_buf()),
            ..Default::default()
        },
        Default::default(),
    ));
    let seed = bridge
        .prepare_initial_vue_project(CorsaVueWorkspaceRequest {
            source_path: path,
            content: source.into(),
            options: state.canonical_vue_document_options(),
            overlays: Vec::new(),
            virtual_ts_options: Default::default(),
            requested_sources: Vec::new(),
        })
        .unwrap()
        .expect("actual owned Canon graph");
    assert!(!bridge.is_initialized());
    assert_eq!(bridge.pending_startups(), 1);
    *state.corsa_bridge.write() = Some(Arc::clone(&bridge));
    crate::runtime::block_on(state.shutdown_project_contexts()).unwrap();
    assert!(!bridge.is_initialized());
    let outcome = crate::runtime::block_on(bridge.spawn_with_initial_project(seed, || true));
    assert!(matches!(outcome, Err(CorsaBridgeError::ProcessTerminated)));
    assert!(!bridge.is_initialized());
    assert!(state.corsa_init_failure().is_none());
}

#[test]
fn missing_provider_uses_old_eager_error_before_preparing_an_owned_graph() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("App.vue");
    std::fs::write(root.path().join("tsconfig.json"), "{}").unwrap();
    let state = Arc::new(ServerState::new());
    state.set_workspace_root(root.path().to_path_buf());
    let uri = Url::from_file_path(&path).unwrap();
    state
        .documents
        .open(uri.clone(), "<template />".into(), 1, "vue".into());
    let source = state.documents.text_snapshot(&uri).unwrap();
    let config = CorsaBridgeConfig {
        corsa_path: Some(root.path().join("missing-provider")),
        working_dir: Some(root.path().to_path_buf()),
        ..Default::default()
    };
    let baseline = CorsaBridge::with_config_and_package_routes(config.clone(), Default::default());
    let bridge = Arc::new(CorsaBridge::with_config_and_package_routes(
        config,
        Default::default(),
    ));
    let initial = InitialCanonicalSource {
        owner: &state,
        uri: &uri,
        source: &source,
    };
    let generation = state.corsa_environment_revision.load(Ordering::Acquire);
    crate::runtime::block_on(async {
        let expected = baseline.spawn().await.unwrap_err();
        let (actual, receipt) = state
            .spawn_initial_corsa_bridge(&bridge, Some(&initial), generation)
            .await
            .expect("current source");
        assert_eq!(actual.unwrap_err().to_string(), expected.to_string());
        assert!(receipt.is_none());
    });
    assert!(!bridge.is_initialized());
    assert_eq!(bridge.pending_startups(), 0);
    assert!(state.corsa_init_failure().is_none());
}

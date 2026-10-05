use super::{
    Arc, NativeNamesConfigurationError, SOURCE, ServerState, assert_original, enable,
    linked_ticket, parser_aba, project, worker,
};
use vize_l0::config::{VueDialect, VueVersion};

fn config(source: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("vize.config.json"), source).unwrap();
    dir
}

#[test]
fn actual_vue_version_loader_aba_revokes_pending_original_names_publication() {
    let changed = config(r#"{"vue":{"version":"2.7"}}"#);
    let restored = config(r#"{"vue":{"version":"3"}}"#);
    parser_aba(|state| {
        state.load_lsp_config(changed.path());
        state.load_lsp_config(restored.path());
    });
}
#[test]
fn actual_patterned_workspace_loader_aba_revokes_pending_original_names_publication() {
    let changed = config(r#"{"experimentals":{"patternedTemplate":true}}"#);
    let restored = config("{}");
    parser_aba(|state| {
        state.load_workspace_config(changed.path());
        state.load_workspace_config(restored.path());
    });
}

#[test]
fn real_no_op_initialization_setter_and_both_loaders_preserve_tickets_and_one_production() {
    let (state, project) = project(SOURCE);
    enable(&state);
    assert_original(&project);
    let original = worker(&project);
    let parser = state.capture_native_names_parser();
    let linked = linked_ticket(&state);
    let values = state.native_names_settings();
    let dir = config("{}");
    state.set_dialect_config(None);
    enable(&state);
    state.load_lsp_config(dir.path());
    state.load_workspace_config(dir.path());
    assert_eq!(state.native_names_settings(), values);
    assert_eq!(
        state.with_current_native_names_parser(&parser, |s| s),
        Ok(values)
    );
    assert_eq!(
        state.with_current_native_linked_names(&linked, |s| s),
        Ok(values)
    );
    assert_original(&project);
    assert!(Arc::ptr_eq(&original, &worker(&project)));
    assert_eq!(original.sfc_productions(), 1);
}

#[test]
fn linked_identity_changes_when_either_actual_route_input_changes_while_route_is_disabled() {
    let state = ServerState::new();
    enable(&state);
    let linked = linked_ticket(&state);
    for options in [
        serde_json::json!({"rename":false}),
        serde_json::json!({"nativeLinkedEditing":false}),
        serde_json::json!({"rename":true}),
        serde_json::json!({"nativeLinkedEditing":true}),
    ] {
        state.apply_lsp_initialization_options(Some(&options));
    }
    assert_eq!(
        state.with_current_native_linked_names(&linked, |_| ()),
        Err(NativeNamesConfigurationError::RouteChanged)
    );
    let current = linked_ticket(&state);
    assert!(
        state
            .with_current_native_linked_names(&current, |_| ())
            .is_ok()
    );
}

#[test]
fn both_real_config_loaders_capture_only_complete_actual_parser_and_route_transactions() {
    let first = config(
        r#"{"vue":{"version":"3"},"dialect":"vue","languageServer":{"rename":true,"legacyVue2":false},"experimentals":{"patternedTemplate":false}}"#,
    );
    let second = config(
        r#"{"vue":{"version":"2.7"},"dialect":"petite-vue","languageServer":{"rename":false,"legacyVue2":true},"experimentals":{"patternedTemplate":true}}"#,
    );
    for load in [
        ServerState::load_lsp_config,
        ServerState::load_workspace_config,
    ] {
        let state = Arc::new(ServerState::new());
        enable(&state);
        load(&state, first.path());
        let a = state.native_names_settings();
        load(&state, second.path());
        let b = state.native_names_settings();
        assert_eq!(
            (
                a.version(),
                a.configured_dialect(),
                a.legacy(),
                a.patterned()
            ),
            (VueVersion::V3, Some(VueDialect::Vue), false, false)
        );
        assert_eq!(
            (
                b.version(),
                b.configured_dialect(),
                b.legacy(),
                b.patterned()
            ),
            (VueVersion::V2_7, Some(VueDialect::PetiteVue), true, true)
        );
        assert!(matches!(
            state.capture_native_linked_route(),
            crate::server::NativeLinkedNamesRoute::Disabled
        ));
        let start = Arc::new(std::sync::Barrier::new(2));
        std::thread::scope(|scope| {
            let writer = scope.spawn(|| {
                start.wait();
                for _ in 0..32 {
                    load(&state, first.path());
                    load(&state, second.path());
                }
            });
            start.wait();
            for _ in 0..2048 {
                let values = state.native_names_settings();
                assert!(
                    values == a || values == b,
                    "partially applied real config: {values:?}"
                );
            }
            writer.join().unwrap();
        });
    }
}

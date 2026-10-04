use super::{ModuleLinkContextError, ServerState};
use std::path::Path;
use vize_l0::config::TypeCheckerConfig;

const CONFIG: &str = r#"{"typeChecker":{"enabled":false,"strict":true,"checkProps":false,"checkEmits":false,"checkTemplateBindings":false,"checkReactivity":false,"checkSetupContext":false,"checkInvalidExports":false,"checkFallthroughAttrs":false,"tsconfig":"app/tsconfig.json","corsaPath":"tools/corsa","globalsFile":"globals.d.ts","servers":2,"lspRequestTimeoutMs":91000}}"#;

#[test]
fn module_link_context_retains_complete_actual_applied_values_and_separate_load_origin() {
    for load in [
        ServerState::load_workspace_config,
        ServerState::load_lsp_config,
    ] {
        let dir = tempfile::tempdir().unwrap();
        let config_dir = dir.path().join("config");
        std::fs::create_dir(&config_dir).unwrap();
        let config_path = config_dir.join("vize.config.json");
        std::fs::write(&config_path, CONFIG).unwrap();
        let root = dir.path().join("actual-root");
        let state = ServerState::new();
        assert!(matches!(
            state.capture_module_link_context(),
            Err(ModuleLinkContextError::MissingRoot)
        ));
        state.set_workspace_root(root.clone());
        let defaults = state.capture_module_link_context().unwrap();
        assert_eq!(defaults.checker(), &TypeCheckerConfig::default());
        assert_eq!(defaults.timeout_ms(), 60_000);
        assert_eq!(defaults.load_origin(), None);
        load(&state, &config_dir);
        let context = state.capture_module_link_context().unwrap();
        assert_eq!(context.root(), root.as_path());
        assert_eq!(
            context.checker(),
            &TypeCheckerConfig {
                enabled: false,
                strict: true,
                check_props: false,
                check_emits: false,
                check_template_bindings: false,
                check_reactivity: false,
                check_setup_context: false,
                check_invalid_exports: false,
                check_fallthrough_attrs: false,
                tsconfig: Some("app/tsconfig.json".into()),
                tsgo_path: Some("tools/corsa".into()),
                globals_file: Some("globals.d.ts".into()),
                servers: Some(2),
            }
        );
        assert_eq!(context.timeout_ms(), 91_000);
        assert_eq!(context.load_origin(), Some(config_path.as_path()));
        assert_eq!(context.project().root(), Some(root.as_path()));
        assert_eq!(context.project().config_source(), None);
        assert_eq!(
            context.project().tsconfig(),
            Some(root.join("app/tsconfig.json").as_path())
        );
        assert_eq!(
            state.with_current_module_link_context(&defaults, || ()),
            Err(ModuleLinkContextError::Superseded)
        );
    }
}

#[test]
fn module_link_actual_equal_reload_and_root_checker_aba_invalidate_original_ticket() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vize.config.json");
    std::fs::write(&path, CONFIG).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(dir.path().into());
    state.load_lsp_config(dir.path());
    let a = state.capture_module_link_context().unwrap();
    state.load_lsp_config(dir.path());
    assert_eq!(
        state.with_current_module_link_context(&a, || ()),
        Err(ModuleLinkContextError::Superseded)
    );
    let equal = state.capture_module_link_context().unwrap();
    state.set_workspace_root(dir.path().into());
    assert_eq!(
        state.with_current_module_link_context(&equal, || ()),
        Err(ModuleLinkContextError::Superseded)
    );
    let root_a = state.capture_module_link_context().unwrap();
    state.set_workspace_root(dir.path().join("B"));
    state.set_workspace_root(dir.path().into());
    assert_eq!(
        state.with_current_module_link_context(&root_a, || ()),
        Err(ModuleLinkContextError::Superseded)
    );
    let checker_a = state.capture_module_link_context().unwrap();
    std::fs::write(&path, "{}").unwrap();
    state.load_workspace_config(dir.path());
    std::fs::write(&path, CONFIG).unwrap();
    state.load_workspace_config(dir.path());
    assert_eq!(
        state.with_current_module_link_context(&checker_a, || ()),
        Err(ModuleLinkContextError::Superseded)
    );
    let fresh = state.capture_module_link_context().unwrap();
    assert_eq!(fresh.checker(), checker_a.checker());
    assert_eq!(fresh.timeout_ms(), checker_a.timeout_ms());
    assert_eq!(state.with_current_module_link_context(&fresh, || 7), Ok(7));
}

#[test]
fn module_link_missing_config_does_not_replace_actual_origin_or_generation() {
    let dir = tempfile::tempdir().unwrap();
    let no_config = tempfile::tempdir().unwrap();
    let path = dir.path().join("vize.config.json");
    std::fs::write(&path, CONFIG).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(dir.path().into());
    state.load_workspace_config(dir.path());
    let original = state.capture_module_link_context().unwrap();
    for load in [
        ServerState::load_workspace_config,
        ServerState::load_lsp_config,
    ] {
        load(&state, no_config.path());
        assert_eq!(
            state.with_current_module_link_context(&original, || ()),
            Ok(())
        );
        assert_eq!(
            state.capture_module_link_context().unwrap().load_origin(),
            Some(path.as_path())
        );
    }
}

#[test]
fn module_link_equal_foreign_server_is_not_the_original_session() {
    let a = ServerState::new();
    let b = ServerState::new();
    for state in [&a, &b] {
        state.set_workspace_root(Path::new("/actual-identical-root").into());
    }
    let a_context = a.capture_module_link_context().unwrap();
    let b_context = b.capture_module_link_context().unwrap();
    assert_eq!(a_context.checker(), b_context.checker());
    assert_eq!(a_context.root(), b_context.root());
    assert_eq!(
        b.with_current_module_link_context(&a_context, || ()),
        Err(ModuleLinkContextError::ForeignSession)
    );
    assert_eq!(
        a.with_current_module_link_context(&a_context, || ()),
        Ok(())
    );
}

#[test]
fn module_link_capture_waits_for_complete_applied_tuple_origin_and_generation_commit() {
    let state = std::sync::Arc::new(ServerState::new());
    state.set_workspace_root("/actual-before".into());
    let original = state.capture_module_link_context().unwrap();
    let (applied, applied_rx) = std::sync::mpsc::channel();
    let (release, release_rx) = std::sync::mpsc::channel();
    let (attempt, attempt_rx) = std::sync::mpsc::channel();
    let (captured, captured_rx) = std::sync::mpsc::channel();
    let writer_state = std::sync::Arc::clone(&state);
    let writer = std::thread::spawn(move || {
        // Test-only rendezvous exposes the real private assignment boundary.
        // The production closure only performs its authoritative assignment.
        writer_state.update_module_link_context(
            Some("/actual-config/vize.config.json".into()),
            || {
                *writer_state.type_checker_config.write() = (
                    TypeCheckerConfig {
                        strict: true,
                        tsconfig: Some("tsconfig.boundary.json".into()),
                        ..Default::default()
                    },
                    93_000,
                );
                applied.send(()).unwrap();
                release_rx.recv().unwrap();
            },
        );
    });
    applied_rx.recv().unwrap();
    let reader_state = std::sync::Arc::clone(&state);
    let reader = std::thread::spawn(move || {
        attempt.send(()).unwrap();
        captured
            .send(reader_state.capture_module_link_context().unwrap())
            .unwrap_or_else(|_| panic!("original capture receiver disappeared"));
    });
    attempt_rx.recv().unwrap();
    assert!(matches!(
        captured_rx.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Empty)
    ));
    release.send(()).unwrap();
    writer.join().unwrap();
    let fresh = captured_rx.recv().unwrap();
    reader.join().unwrap();
    assert_eq!(
        fresh.checker(),
        &TypeCheckerConfig {
            strict: true,
            tsconfig: Some("tsconfig.boundary.json".into()),
            ..Default::default()
        }
    );
    assert_eq!(fresh.timeout_ms(), 93_000);
    assert_eq!(fresh.root(), Path::new("/actual-before"));
    assert_eq!(
        fresh.load_origin(),
        Some(Path::new("/actual-config/vize.config.json"))
    );
    assert_eq!(
        fresh.project().tsconfig(),
        Some(Path::new("/actual-before/tsconfig.boundary.json"))
    );
    assert_eq!(
        state.with_current_module_link_context(&original, || ()),
        Err(ModuleLinkContextError::Superseded)
    );
    assert_eq!(
        state.with_current_module_link_context(&fresh, || ()),
        Ok(())
    );
}

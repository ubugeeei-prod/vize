use super::{ModuleLinkContextError, ModuleLinkRetirement, Phase, ProjectModel, ServerState};
#[cfg(feature = "native")]
mod native;

#[cfg(not(feature = "native"))]
#[test]
fn minimal_module_link_context_has_typed_unavailable_without_invented_root() {
    let state = ServerState::new();
    assert!(matches!(
        state.capture_module_link_context(),
        Err(ModuleLinkContextError::NativeUnavailable)
    ));
    state.retire_module_links(ModuleLinkRetirement::Shutdown);
    assert!(matches!(
        state.capture_module_link_context(),
        Err(ModuleLinkContextError::Retired(
            ModuleLinkRetirement::Shutdown
        ))
    ));
}

#[test]
fn module_link_mutation_unwind_retires_before_gate_release() {
    let state = ServerState::new();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.update_module_link_context(None, || {
            let config = vize_l0::config::TypeCheckerConfig {
                strict: true,
                ..Default::default()
            };
            *state.type_checker_config.write() =
                (config.clone(), 123, ProjectModel::new(None, None, &config));
            panic!("original assignment interrupted before commit");
        });
    }));
    assert!(panic.is_err());
    assert_eq!(state.type_checker_config.read().1, 123);
    assert!(state.get_type_checker_config().strict);
    assert_eq!(
        state.module_links.read().phase,
        Phase::Retired(ModuleLinkRetirement::MutationUnwound)
    );
    // Actual legacy config assignment still runs after this sticky retirement.
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vite.config.mjs"),
        r#"export default { vue: {"typeChecker":{"lspRequestTimeoutMs":91000}} };"#,
    )
    .unwrap();
    state.load_lsp_config(dir.path());
    assert_eq!(state.lsp_request_timeout_ms(), 91_000);
    assert_eq!(
        state.get_type_checker_config(),
        vize_l0::config::TypeCheckerConfig::default()
    );
    assert!(matches!(
        state.capture_module_link_context(),
        Err(ModuleLinkContextError::Retired(
            ModuleLinkRetirement::MutationUnwound
        ))
    ));
}

#[test]
fn module_link_retirement_is_sticky_and_generation_overflow_keeps_legacy_apply() {
    let state = ServerState::new();
    state.module_links.write().generation = u64::MAX;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vite.config.mjs");
    std::fs::write(
        &path,
        r#"export default { vue: {"typeChecker":{"strict":true,"lspRequestTimeoutMs":91000}} };"#,
    )
    .unwrap();
    state.load_workspace_config(dir.path());
    assert_eq!(state.module_links.read().generation, u64::MAX);
    assert_eq!(state.module_links.read().load_origin.as_ref(), Some(&path));
    assert_eq!(state.lsp_request_timeout_ms(), 91_000);
    assert!(state.get_type_checker_config().strict);
    assert!(matches!(
        state.capture_module_link_context(),
        Err(ModuleLinkContextError::Retired(
            ModuleLinkRetirement::GenerationExhausted
        ))
    ));
    state.retire_module_links(ModuleLinkRetirement::Shutdown);
    state.load_lsp_config(dir.path());
    assert_eq!(
        state.module_links.read().phase,
        Phase::Retired(ModuleLinkRetirement::GenerationExhausted)
    );
    #[cfg(feature = "native")]
    {
        state.set_workspace_root(dir.path().join("after-overflow"));
        assert_eq!(
            state.get_workspace_root(),
            Some(dir.path().join("after-overflow"))
        );
        assert!(matches!(
            state.capture_module_link_context(),
            Err(ModuleLinkContextError::Retired(
                ModuleLinkRetirement::GenerationExhausted
            ))
        ));
    }
}

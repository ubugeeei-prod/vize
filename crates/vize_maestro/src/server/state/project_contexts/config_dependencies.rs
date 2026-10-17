//! The evaluated configuration graph controls owner invalidation.
use super::*;
use tower_lsp::lsp_types::{FileChangeType, FileEvent};

#[test]
fn imported_helper_refreshes_its_owner_and_preserves_sibling_and_open_buffer() {
    let (fixture, state, a, b) = tests::fixture();
    let helper = fixture.path().join("shared-policy.mjs");
    let config = fixture.path().join("packages/a/vite.config.mjs");
    std::fs::write(
        &helper,
        "export default { languageServer: { hover: false } };",
    )
    .unwrap();
    std::fs::write(
        &config,
        "import vue from '../../shared-policy.mjs'; export default { vue };",
    )
    .unwrap();
    state.documents.open(
        a.clone(),
        "<template>unsaved buffer</template>".into(),
        7,
        "vue".into(),
    );
    let previous = state.document_project_state(&a).unwrap();
    let sibling = state.document_project_state(&b).unwrap();
    assert!(!previous.lsp_features().hover);
    let pending = previous.corsa_request_stamp();
    std::fs::write(
        &helper,
        "export default { languageServer: { hover: true } };",
    )
    .unwrap();
    let affected = state.observe_project_config_events(&[FileEvent {
        uri: Url::from_file_path(&helper).unwrap(),
        typ: FileChangeType::CHANGED,
    }]);
    assert_eq!(affected, vec![(a.clone(), 7)]);
    let current = state.document_project_state(&a).unwrap();
    assert!(current.lsp_features().hover);
    assert!(!Arc::ptr_eq(&previous, &current));
    assert!(!pending.is_current(&previous));
    assert!(Arc::ptr_eq(
        &sibling,
        &state.document_project_state(&b).unwrap()
    ));
    assert_eq!(current.documents.version(&a), Some(7));
    assert_eq!(
        current.documents.text(&a).as_deref(),
        Some("<template>unsaved buffer</template>")
    );
}

#[test]
fn unrelated_javascript_does_not_reinitialize_project_configuration() {
    let (fixture, state, a, _) = tests::fixture();
    let before = state.document_project_state(&a).unwrap();
    state.observe_project_config_events(&[FileEvent {
        uri: Url::from_file_path(fixture.path().join("packages/a/src/utils.mjs")).unwrap(),
        typ: FileChangeType::CHANGED,
    }]);
    assert!(Arc::ptr_eq(
        &before,
        &state.document_project_state(&a).unwrap()
    ));
}

#[test]
fn inherited_tsconfig_target_refreshes_from_a_nonstandard_config_name() {
    let (fixture, state, a, b) = tests::fixture();
    let parent = fixture.path().join("shared-options.json");
    std::fs::write(&parent, r#"{"vueCompilerOptions":{"target":2.7}}"#).unwrap();
    std::fs::write(
        fixture.path().join("packages/a/tsconfig.json"),
        r#"{"extends":"../../shared-options.json"}"#,
    )
    .unwrap();
    let before = state.document_project_state(&a).unwrap();
    let sibling = state.document_project_state(&b).unwrap();
    assert_eq!(
        before.type_checker_vue_version(),
        vize_l0::config::VueVersion::V2_7
    );
    std::fs::write(&parent, r#"{"vueCompilerOptions":{"target":3.5}}"#).unwrap();
    state.observe_project_config_events(&[FileEvent {
        uri: Url::from_file_path(&parent).unwrap(),
        typ: FileChangeType::CHANGED,
    }]);
    let current = state.document_project_state(&a).unwrap();
    assert_eq!(
        current.type_checker_vue_version(),
        vize_l0::config::VueVersion::V3
    );
    assert!(!Arc::ptr_eq(&before, &current));
    assert!(Arc::ptr_eq(
        &sibling,
        &state.document_project_state(&b).unwrap()
    ));
}

#[test]
fn explicit_vite_dialect_wins_over_the_tsconfig_target() {
    let (fixture, state, a, _) = tests::fixture();
    let package = fixture.path().join("packages/a");
    std::fs::write(
        package.join("tsconfig.json"),
        r#"{"vueCompilerOptions":{"target":2.7}}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("vite.config.mjs"),
        "export default { vue: { version: '3' } };",
    )
    .unwrap();
    let project = state.document_project_state(&a).unwrap();
    assert_eq!(
        project.type_checker_vue_version(),
        vize_l0::config::VueVersion::V3
    );
    assert!(!project.legacy_vue2_enabled());
}

#[test]
fn vite_source_root_uses_the_existing_typescript_project() {
    let (fixture, state, a, _) = tests::fixture();
    let package = fixture.path().join("packages/a");
    let config = package.join("tsconfig.json");
    std::fs::write(&config, r#"{"vueCompilerOptions":{"target":2.7}}"#).unwrap();
    std::fs::write(
        package.join("vite.config.mjs"),
        "export default { root: 'src' };",
    )
    .unwrap();
    let project = state.document_project_state(&a).unwrap();
    assert_eq!(
        project.type_checker_vue_version(),
        vize_l0::config::VueVersion::V2_7
    );
    assert_eq!(
        project.get_type_checker_config().tsconfig.as_deref(),
        config.to_str()
    );
}

//! A rejected dependency edit never replaces a last-good project owner.
use super::*;
use tower_lsp::lsp_types::{FileChangeType, FileEvent};
use vize_l0::cstr;

fn event(path: &Path, typ: FileChangeType) -> FileEvent {
    FileEvent {
        uri: Url::from_file_path(path).unwrap(),
        typ,
    }
}

fn settings(state: &ServerState) -> serde_json::Value {
    serde_json::json!({
        "checker": state.get_type_checker_config(),
        "timeout": state.lsp_request_timeout_ms(),
        "linter": state.get_linter_config(),
        "rules": state.get_linter_rule_options(),
        "dialect": state.get_dialect_config(),
        "vue": state.type_checker_vue_version().as_str(),
        "dependencies": *state.project_contexts.config_dependencies.read(),
    })
}

fn assert_error(result: &config_reload::ProjectConfigChanges, root: &Path, changed: &Path) {
    let root = Url::from_file_path(root).unwrap();
    let changed = Url::from_file_path(changed).unwrap();
    assert_eq!(
        serde_json::to_value(&result.errors).unwrap(),
        serde_json::json!([{
            "type": 1,
            "message": cstr!("Vize could not reload project configuration for {root} after changes to {changed}. Keeping the last good configuration.").as_str(),
        }])
    );
}

#[test]
fn missing_and_invalid_imported_helpers_keep_owner_settings_and_recover_without_source_edit() {
    let (fixture, state, a, b) = tests::fixture();
    let root = fixture.path().join("packages/a");
    let helper = root.join("vue-settings.mjs");
    std::fs::write(
        root.join("vite.config.mjs"),
        "import vue from './vue-settings.mjs'; export default { vue };",
    )
    .unwrap();
    let good = "export default { formatter: { singleQuote: true }, languageServer: { hover: false }, linter: { rules: { 'a11y/alt-text': 'error' } } };";
    std::fs::write(&helper, good).unwrap();
    state.documents.open(
        a.clone(),
        "<template>unsaved</template>".into(),
        7,
        "vue".into(),
    );
    let sibling = state.document_project_state(&b).unwrap();
    let sibling_settings = settings(&sibling);
    let sibling_stamp = sibling.corsa_request_stamp();
    for invalid in [None, Some("export default { broken syntax")] {
        let previous = state.document_project_state(&a).unwrap();
        let before = settings(&previous);
        let features = previous.lsp_features();
        #[cfg(feature = "glyph")]
        let formatter = serde_json::to_value(previous.get_formatter_context()).unwrap();
        let stamp = previous.corsa_request_stamp();
        let routing = state.stable_project_routing_generation();
        let typ = if let Some(invalid) = invalid {
            std::fs::write(&helper, invalid).unwrap();
            FileChangeType::CHANGED
        } else {
            std::fs::remove_file(&helper).unwrap();
            FileChangeType::DELETED
        };
        let rejected_event = event(&helper, typ);
        let unrelated = event(&root.join("src/normal.ts"), FileChangeType::CHANGED);
        let result = state
            .observe_project_config_events_checked(&[rejected_event.clone(), unrelated.clone()]);
        assert_error(&result, &root, &helper);
        assert_eq!(result.documents, vec![(a.clone(), 7)]);
        assert!(Arc::ptr_eq(
            &previous,
            &state.document_project_state(&a).unwrap()
        ));
        assert!(!previous.project_context_retired());
        assert!(stamp.is_current(&previous));
        assert_eq!(state.stable_project_routing_generation(), routing);
        assert_eq!(settings(&previous), before);
        assert_eq!(previous.lsp_features(), features);
        #[cfg(feature = "glyph")]
        assert_eq!(
            serde_json::to_value(previous.get_formatter_context()).unwrap(),
            formatter
        );
        assert_eq!(
            result.accepted_file_events(&previous, &[rejected_event.clone(), unrelated.clone()]),
            vec![unrelated]
        );
        assert!(
            result
                .accepted_file_events(&sibling, &[rejected_event])
                .is_empty()
        );
        assert!(Arc::ptr_eq(
            &sibling,
            &state.document_project_state(&b).unwrap()
        ));
        assert_eq!(settings(&sibling), sibling_settings);
        assert!(sibling_stamp.is_current(&sibling));
        std::fs::write(&helper, good).unwrap();
        let repaired =
            state.observe_project_config_events_checked(&[event(&helper, FileChangeType::CREATED)]);
        assert!(repaired.errors.is_empty());
        let current = state.document_project_state(&a).unwrap();
        assert!(!Arc::ptr_eq(&previous, &current));
        assert!(!stamp.is_current(&previous));
        assert_eq!(settings(&current), before);
        assert_eq!(
            current.documents.text(&a).as_deref(),
            Some("<template>unsaved</template>")
        );
        assert_eq!(current.documents.version(&a), Some(7));
        assert!(Arc::ptr_eq(
            &sibling,
            &state.document_project_state(&b).unwrap()
        ));
    }
}

#[test]
fn inherited_base_failure_is_checked_even_with_explicit_vite_vue_dialect() {
    for explicit_vue in [false, true] {
        let (fixture, state, a, b) = tests::fixture();
        let root = fixture.path().join("packages/a");
        let base = root.join("tsconfig.base.json");
        let good =
            r#"{"compilerOptions":{"strictNullChecks":true},"vueCompilerOptions":{"target":2.7}}"#;
        std::fs::write(&base, good).unwrap();
        std::fs::write(
            root.join("tsconfig.json"),
            r#"{"extends":"./tsconfig.base.json"}"#,
        )
        .unwrap();
        if explicit_vue {
            std::fs::write(
                root.join("vite.config.mjs"),
                "export default { vue: { version: '3' } };",
            )
            .unwrap();
        }
        let sibling = state.document_project_state(&b).unwrap();
        // A missing base is skipped like `tsc`; malformed bases still reject.
        for invalid in [Some("{ invalid JSON"), Some("{")] {
            let previous = state.document_project_state(&a).unwrap();
            let before = settings(&previous);
            let stamp = previous.corsa_request_stamp();
            let typ = if let Some(invalid) = invalid {
                std::fs::write(&base, invalid).unwrap();
                FileChangeType::CHANGED
            } else {
                std::fs::remove_file(&base).unwrap();
                FileChangeType::DELETED
            };
            let change = event(&base, typ);
            let result = state.observe_project_config_events_checked(std::slice::from_ref(&change));
            assert_error(&result, &root, &base);
            assert!(Arc::ptr_eq(
                &previous,
                &state.document_project_state(&a).unwrap()
            ));
            assert_eq!(settings(&previous), before);
            assert!(stamp.is_current(&previous));
            assert!(
                result
                    .accepted_file_events(&previous, std::slice::from_ref(&change))
                    .is_empty()
            );
            assert!(result.accepted_file_events(&sibling, &[change]).is_empty());
            std::fs::write(&base, good).unwrap();
            let repaired = state
                .observe_project_config_events_checked(&[event(&base, FileChangeType::CREATED)]);
            assert!(repaired.errors.is_empty());
            let current = state.document_project_state(&a).unwrap();
            assert!(!Arc::ptr_eq(&previous, &current));
            assert_eq!(settings(&current), before);
            assert!(Arc::ptr_eq(
                &sibling,
                &state.document_project_state(&b).unwrap()
            ));
        }
    }
}

#[test]
fn prepared_snapshot_is_used_once_and_remains_watchable_before_lazy_initialization() {
    let (fixture, state, a, _) = tests::fixture();
    let root = fixture.path().join("packages/a");
    let helper = root.join("settings.mjs");
    std::fs::write(
        &helper,
        "export default { languageServer: { hover: false } };",
    )
    .unwrap();
    std::fs::write(
        root.join("vite.config.mjs"),
        "import vue from './settings.mjs'; export default { vue };",
    )
    .unwrap();
    let original = state.document_project_state(&a).unwrap();
    std::fs::write(
        &helper,
        "export default { languageServer: { hover: true } };",
    )
    .unwrap();
    let result =
        state.observe_project_config_events_checked(&[event(&helper, FileChangeType::CHANGED)]);
    assert!(result.errors.is_empty());
    assert!(state.cached_project_states().is_empty());
    // A second event must see the prepared configuration's dependency graph,
    // even before any document initializes that replacement context.
    std::fs::remove_file(&helper).unwrap();
    let rejected =
        state.observe_project_config_events_checked(&[event(&helper, FileChangeType::DELETED)]);
    assert_error(&rejected, &root, &helper);
    let prepared = state.document_project_state(&a).unwrap();
    assert!(
        prepared.lsp_features().hover,
        "no second evaluation of the now-missing helper"
    );
    assert!(!Arc::ptr_eq(&original, &prepared));
    assert!(!prepared.project_context_retired());
    assert!(
        prepared
            .project_contexts
            .config_reload_failed
            .load(Ordering::Acquire)
    );
    assert!(prepared.linter_settings_for_uri(&a).is_some());
}

#[test]
fn deleting_root_vite_config_can_select_defaults() {
    let (fixture, state, a, b) = tests::fixture();
    let root = fixture.path().join("packages/a");
    let previous = state.document_project_state(&a).unwrap();
    let sibling = state.document_project_state(&b).unwrap();
    let config = root.join("vite.config.mjs");
    std::fs::remove_file(&config).unwrap();
    let result =
        state.observe_project_config_events_checked(&[event(&config, FileChangeType::DELETED)]);
    assert!(result.errors.is_empty());
    let current = state.document_project_state(&a).unwrap();
    assert!(!Arc::ptr_eq(&previous, &current));
    assert!(current.lsp_features().hover);
    assert!(Arc::ptr_eq(
        &sibling,
        &state.document_project_state(&b).unwrap()
    ));
}

#[test]
fn lazy_native_owner_retains_admitted_compiler_authority_after_inherited_config_corruption() {
    let (fixture, state, a, b) = tests::fixture();
    let root = fixture.path().join("packages/a").canonicalize().unwrap();
    let base = root.join("base.json");
    let config = root.join("tsconfig.json");
    std::fs::write(
        &config,
        r#"{"extends":"./base.json","include":["src/**/*"]}"#,
    )
    .unwrap();
    std::fs::write(&base, r#"{"compilerOptions":{"strictNullChecks":true}}"#).unwrap();
    let initial = state.document_project_state(&a).unwrap();
    let sibling = state.document_project_state(&b).unwrap();
    assert!(initial.corsa_bridge.read().is_none());
    let accepted =
        state.observe_project_config_events_checked(&[event(&base, FileChangeType::CHANGED)]);
    assert!(accepted.errors.is_empty());
    assert!(initial.project_context_retired());
    // The replacement has admitted config bytes but no initialized state/backend.
    assert_eq!(state.cached_project_states().len(), 1);
    // A deleted base would be skipped like `tsc`; a malformed one still rejects.
    std::fs::write(&base, "{").unwrap();
    let rejected =
        state.observe_project_config_events_checked(&[event(&base, FileChangeType::CHANGED)]);
    assert_error(&rejected, &root, &base);
    let retained = state.document_project_state(&a).unwrap();
    assert!(retained.corsa_bridge.read().is_none());
    let compiler = retained
        .project_contexts
        .compiler_configuration
        .read()
        .clone()
        .unwrap();
    assert!(compiler.dependencies().contains(&base));
    assert!(vize_canon::CompilerConfigurationSnapshot::capture(&root, Some(&config)).is_err());
    assert!(!retained.project_context_retired());
    assert!(Arc::ptr_eq(
        &sibling,
        &state.document_project_state(&b).unwrap()
    ));
    std::fs::write(&base, r#"{"compilerOptions":{"strictNullChecks":false}}"#).unwrap();
    let recovered =
        state.observe_project_config_events_checked(&[event(&base, FileChangeType::CHANGED)]);
    assert!(recovered.errors.is_empty());
    let repaired = state.document_project_state(&a).unwrap();
    assert!(!Arc::ptr_eq(&retained, &repaired));
    assert!(!Arc::ptr_eq(
        &compiler,
        repaired
            .project_contexts
            .compiler_configuration
            .read()
            .as_ref()
            .unwrap()
    ));
    assert!(Arc::ptr_eq(
        &sibling,
        &state.document_project_state(&b).unwrap()
    ));
}

#[test]
fn config_helper_that_is_also_a_sibling_source_import_still_invalidates_that_importer() {
    let (fixture, state, a, b) = tests::fixture();
    let helper = fixture.path().join("packages/a/shared.mjs");
    std::fs::write(
        &helper,
        "export default { languageServer: { hover: false } };",
    )
    .unwrap();
    std::fs::write(
        fixture.path().join("packages/a/vite.config.mjs"),
        "import vue from './shared.mjs'; export default { vue };",
    )
    .unwrap();
    state.document_project_state(&a).unwrap();
    let sibling = state.document_project_state(&b).unwrap();
    state.documents.open(
        b.clone(),
        "<script setup>import policy from '../../a/shared.mjs'</script>".into(),
        1,
        "vue".into(),
    );
    sibling
        .open_imports
        .update(&b, &state.documents.text(&b).unwrap());
    std::fs::write(
        &helper,
        "export default { languageServer: { hover: true } };",
    )
    .unwrap();
    let changed = event(&helper, FileChangeType::CHANGED);
    let result = state.observe_project_config_events_checked(std::slice::from_ref(&changed));
    assert!(result.errors.is_empty());
    assert_eq!(
        result.accepted_file_events(&sibling, std::slice::from_ref(&changed)),
        vec![changed]
    );
}

#[test]
fn newly_introduced_missing_or_malformed_imports_recover_on_creation_or_change() {
    for shared in [false, true] {
        for malformed in [false, true] {
            let (fixture, state, a, b) = tests::fixture();
            let root = fixture.path().join("packages/a");
            let helper = root.join("settings.mjs");
            let (specifier, target) = if shared {
                (
                    "../../shared-settings.mjs",
                    fixture.path().join("shared-settings.mjs"),
                )
            } else {
                ("./next-settings.mjs", root.join("next-settings.mjs"))
            };
            std::fs::write(
                root.join("vite.config.mjs"),
                "import vue from './settings.mjs'; export default { vue };",
            )
            .unwrap();
            std::fs::write(
                &helper,
                "export default { languageServer: { hover: false } };",
            )
            .unwrap();
            let before = state.document_project_state(&a).unwrap();
            let sibling = state.document_project_state(&b).unwrap();
            if malformed {
                std::fs::write(&target, "export default {").unwrap();
            }
            std::fs::write(
                &helper,
                cstr!("export {{ default }} from '{specifier}';").as_bytes(),
            )
            .unwrap();
            let rejected = state
                .observe_project_config_events_checked(&[event(&helper, FileChangeType::CHANGED)]);
            assert_error(&rejected, &root, &helper);
            assert!(Arc::ptr_eq(
                &before,
                &state.document_project_state(&a).unwrap()
            ));
            let unrelated = event(&root.join("src/ordinary.ts"), FileChangeType::CHANGED);
            let retry =
                state.observe_project_config_events_checked(std::slice::from_ref(&unrelated));
            assert!(
                retry.errors.is_empty(),
                "incidental retries do not repeat editor error toasts"
            );
            assert_eq!(
                retry.accepted_file_events(&before, std::slice::from_ref(&unrelated)),
                vec![unrelated]
            );
            assert!(Arc::ptr_eq(
                &before,
                &state.document_project_state(&a).unwrap()
            ));
            std::fs::write(
                &target,
                "export default { languageServer: { hover: true } };",
            )
            .unwrap();
            let change = event(
                &target,
                if malformed {
                    FileChangeType::CHANGED
                } else {
                    FileChangeType::CREATED
                },
            );
            let repaired =
                state.observe_project_config_events_checked(std::slice::from_ref(&change));
            assert!(repaired.errors.is_empty());
            let current = state.document_project_state(&a).unwrap();
            assert!(!Arc::ptr_eq(&before, &current));
            assert!(current.lsp_features().hover);
            assert!(Arc::ptr_eq(
                &sibling,
                &state.document_project_state(&b).unwrap()
            ));
            assert!(
                repaired
                    .accepted_file_events(&sibling, &[change])
                    .is_empty()
            );
            assert!(
                state
                    .project_contexts
                    .pending_config_reloads
                    .lock()
                    .is_empty()
            );
        }
    }
}

#[path = "config_reload_recovery_tests.rs"]
mod config_reload_recovery_tests;

#[cfg(unix)]
#[test]
fn a_reload_notice_names_a_symlinked_change_once() {
    let parent = crate::test_support::tempdir().unwrap();
    let physical = parent.path().join("project");
    std::fs::create_dir_all(&physical).unwrap();
    std::fs::write(physical.join("vite.config.mjs"), "export default {}").unwrap();
    let alias = parent.path().join("alias");
    std::os::unix::fs::symlink(&physical, &alias).unwrap();
    let through_alias = Url::from_file_path(alias.join("vite.config.mjs")).unwrap();
    let direct = Url::from_file_path(physical.join("vite.config.mjs")).unwrap();
    let notice = config_reload::reload_error(&physical, &[through_alias.clone(), direct]);
    let root = Url::from_file_path(&physical).unwrap();
    assert_eq!(
        notice.message,
        cstr!("Vize could not reload project configuration for {root} after changes to {through_alias}. Keeping the last good configuration.").as_str()
    );
}

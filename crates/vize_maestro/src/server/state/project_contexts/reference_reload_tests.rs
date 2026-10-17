//! Last good configuration retention for referenced TypeScript owners.
use super::*;

#[test]
fn invalid_leaf_reload_keeps_the_last_valid_dialect_until_the_config_is_fixed() {
    let fixture = ReferenceFixture::new("export default {};");
    let state = &fixture.state;
    let uri = &fixture.legacy;
    let source = fixture.open_dirty(uri, "unsavedDuringReload", 11);
    let mut previous = state.document_project_state(uri).unwrap();
    let modern = state.document_project_state(&fixture.modern).unwrap();
    assert_dialect(&previous, VueVersion::V2_7);
    for (target, expected, failed) in [(4.0, VueVersion::V2_7, true), (3.5, VueVersion::V3, false)]
    {
        let pending = previous.corsa_request_stamp();
        ReferenceFixture::write_leaf(&fixture.root, "legacy", "legacy", target);
        assert_eq!(
            state.observe_project_config_events(&[fixture.event("legacy")]),
            vec![(uri.clone(), 11)]
        );
        let current = state.document_project_state(uri).unwrap();
        if failed {
            assert!(Arc::ptr_eq(&previous, &current));
            assert!(!previous.project_context_retired());
            assert!(pending.is_current(&previous));
        } else {
            assert!(!Arc::ptr_eq(&previous, &current));
            assert!(previous.project_context_retired());
            assert!(!pending.is_current(&previous));
        }
        assert_dialect(&current, expected);
        assert_eq!(
            current
                .project_contexts
                .config_reload_failed
                .load(Ordering::Acquire),
            failed
        );
        assert!(Arc::ptr_eq(
            &modern,
            &state.document_project_state(&fixture.modern).unwrap()
        ));
        assert!(!modern.project_context_retired());
        assert_eq!(
            current.documents.text(uri).as_deref(),
            Some(source.as_str())
        );
        assert_eq!(current.documents.version(uri), Some(11));
        assert_projection(&current, uri, "unsavedDuringReload");
        previous = current;
    }
}

#[test]
fn invalid_vite_reload_keeps_the_last_valid_policy_until_the_config_is_fixed() {
    let (fixture, state, a, b) = tests::fixture();
    let source = "<script>export const unsavedViteReload = 1;</script><template><div /></template>";
    state
        .documents
        .open(a.clone(), source.into(), 12, "vue".into());
    let mut previous = state.document_project_state(&a).unwrap();
    let sibling = state.document_project_state(&b).unwrap();
    let config = fixture.path().join("packages/a/vite.config.mjs");
    assert!(!previous.lsp_features().hover);
    for (contents, hover, failed) in [
        ("export default { vue:", false, true),
        (
            "export default { vue: { languageServer: { hover: true } } };",
            true,
            false,
        ),
    ] {
        let pending = previous.corsa_request_stamp();
        std::fs::write(&config, contents).unwrap();
        assert_eq!(
            state.observe_project_config_events(&[FileEvent {
                uri: Url::from_file_path(&config).unwrap(),
                typ: FileChangeType::CHANGED,
            }]),
            vec![(a.clone(), 12)]
        );
        let current = state.document_project_state(&a).unwrap();
        if failed {
            assert!(Arc::ptr_eq(&previous, &current));
            assert!(!previous.project_context_retired());
            assert!(pending.is_current(&previous));
        } else {
            assert!(!Arc::ptr_eq(&previous, &current));
            assert!(previous.project_context_retired());
            assert!(!pending.is_current(&previous));
        }
        assert_eq!(current.lsp_features().hover, hover);
        assert_eq!(
            current
                .project_contexts
                .config_reload_failed
                .load(Ordering::Acquire),
            failed
        );
        assert!(Arc::ptr_eq(
            &sibling,
            &state.document_project_state(&b).unwrap()
        ));
        assert_eq!(current.documents.text(&a).as_deref(), Some(source));
        assert_eq!(current.documents.version(&a), Some(12));
        assert_projection(&current, &a, "unsavedViteReload");
        previous = current;
    }
}

#[test]
fn typescript_only_references_select_the_same_checker_and_editor_project() {
    let fixture = ReferenceFixture::new("export default {};");
    std::fs::remove_file(fixture.root.join("vite.config.mjs")).unwrap();
    let state = Arc::new(ServerState::new());
    state.set_workspace_root(fixture.root.clone());
    state.load_workspace_config(&fixture.root);
    state.apply_initialize_workspace_folders(None, Some(&fixture.root));
    let legacy = state.document_project_state(&fixture.legacy).unwrap();
    let modern = state.document_project_state(&fixture.modern).unwrap();
    assert_dialect(&legacy, VueVersion::V2_7);
    assert_dialect(&modern, VueVersion::V3);
    assert_eq!(
        legacy.get_type_checker_config().tsconfig.as_deref(),
        fixture.root.join("tsconfig.legacy.json").to_str()
    );
    assert_eq!(
        modern.get_type_checker_config().tsconfig.as_deref(),
        fixture.root.join("tsconfig.modern.json").to_str()
    );
}

#[test]
fn unavailable_reload_worker_retains_retry_only_for_affected_reference_keys() {
    let fixture = ReferenceFixture::new("export default {};");
    fixture.open_dirty(&fixture.legacy, "retainedLegacy", 5);
    fixture.open_dirty(&fixture.modern, "unaffectedModern", 6);
    let legacy = fixture
        .state
        .document_project_state(&fixture.legacy)
        .unwrap();
    let modern = fixture
        .state
        .document_project_state(&fixture.modern)
        .unwrap();
    let changes = [fixture.event("legacy")];
    let failure = fixture
        .state
        .config_worker_unavailable(&fixture.root, &changes);
    assert_eq!(failure.errors.len(), 1);
    assert!(failure.accepted_file_events(&legacy, &changes).is_empty());
    assert!(failure.accepted_file_events(&modern, &changes).is_empty());
    assert!(
        legacy
            .project_contexts
            .config_reload_failed
            .load(Ordering::Acquire)
    );
    assert!(
        !modern
            .project_contexts
            .config_reload_failed
            .load(Ordering::Acquire)
    );
    let pending = fixture.state.project_contexts.pending_config_reloads.lock();
    assert!(pending.contains(legacy.project_contexts.key.read().as_ref().unwrap()));
    assert!(!pending.contains(modern.project_contexts.key.read().as_ref().unwrap()));
}

#[test]
fn failed_reference_retry_preserves_source_import_authority_for_a_sibling_marker() {
    let fixture = ReferenceFixture::new("export default {};");
    std::fs::write(
        fixture.root.join("tsconfig.legacy.json"),
        r#"{"compilerOptions":{"resolveJsonModule":true},"include":["src/legacy/**/*.vue"],"vueCompilerOptions":{"target":2.7}}"#,
    ).unwrap();
    let source = "<script lang=\"ts\">import modern from '../../tsconfig.modern.json'; export const keepImport = modern;</script><template><div /></template>";
    fixture
        .state
        .documents
        .open(fixture.legacy.clone(), source.into(), 7, "vue".into());
    let legacy = fixture
        .state
        .document_project_state(&fixture.legacy)
        .unwrap();
    fixture
        .state
        .document_project_state(&fixture.modern)
        .unwrap();
    // JSON imports arrive through Canon's resolved dependency graph rather
    // than the script-only syntactic fallback index. Use its production
    // revision-guarded recording path for this real, explicitly enabled import.
    let revision = legacy.documents.get(&fixture.legacy).unwrap().revision();
    legacy.record_typecheck_dependencies(
        &fixture.legacy,
        revision,
        &[fixture.root.join("tsconfig.modern.json")],
    );
    assert_eq!(
        legacy.open_importers(&fixture.event("modern").uri),
        vec![fixture.legacy.clone()]
    );
    ReferenceFixture::write_leaf(&fixture.root, "legacy", "legacy", 4.0);
    let initial = fixture
        .state
        .observe_project_config_events_checked(&[fixture.event("legacy")]);
    assert_eq!(initial.errors.len(), 1);
    let changes = [fixture.event("modern")];
    // A remains invalid and pending, but B's marker is an ordinary imported
    // source for A, not an inherited configuration dependency of A.
    let fallback = fixture
        .state
        .config_worker_unavailable(&fixture.root, &changes);
    assert_eq!(fallback.accepted_file_events(&legacy, &changes), changes);
    let retry = fixture
        .state
        .observe_project_config_events_checked(&changes);
    assert!(retry.errors.is_empty());
    assert_eq!(retry.accepted_file_events(&legacy, &changes), changes);
    assert!(Arc::ptr_eq(
        &legacy,
        &fixture
            .state
            .document_project_state(&fixture.legacy)
            .unwrap()
    ));
    assert_dialect(&legacy, VueVersion::V2_7);
    assert_projection(&legacy, &fixture.legacy, "keepImport");
}

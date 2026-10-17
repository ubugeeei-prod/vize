//! Same-directory TypeScript references retain independent editor dialects.
use super::*;
use tower_lsp::lsp_types::{FileChangeType, FileEvent, TextDocumentContentChangeEvent};
use vize_l0::{config::VueVersion, cstr};

struct ReferenceFixture {
    _directory: tempfile::TempDir,
    root: PathBuf,
    state: Arc<ServerState>,
    legacy: Url,
    modern: Url,
}

impl ReferenceFixture {
    fn new(vite_config: &str) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        for dialect in ["legacy", "modern"] {
            std::fs::create_dir_all(root.join("src").join(dialect)).unwrap();
            std::fs::write(
                root.join("src").join(dialect).join("App.vue"),
                "<template><div>on disk</div></template>",
            )
            .unwrap();
        }
        std::fs::write(root.join("package.json"), "{}").unwrap();
        std::fs::write(root.join("vite.config.mjs"), vite_config).unwrap();
        std::fs::write(
            root.join("tsconfig.json"),
            r#"{"files":[],"references":[{"path":"./tsconfig.legacy.json"},{"path":"./tsconfig.modern.json"}]}"#,
        )
        .unwrap();
        Self::write_leaf(&root, "legacy", "legacy", 2.7);
        Self::write_leaf(&root, "modern", "modern", 3.5);
        let state = Arc::new(ServerState::new());
        state.set_workspace_root(root.clone());
        state.load_workspace_config(&root);
        state.apply_initialize_workspace_folders(None, Some(&root));
        let legacy = Url::from_file_path(root.join("src/legacy/App.vue")).unwrap();
        let modern = Url::from_file_path(root.join("src/modern/App.vue")).unwrap();
        Self {
            _directory: directory,
            root,
            state,
            legacy,
            modern,
        }
    }

    fn write_leaf(root: &Path, name: &str, source: &str, target: f64) {
        let config = serde_json::json!({
            "include": [cstr!("src/{source}/**/*.vue")],
            "vueCompilerOptions": {"target": target},
        });
        std::fs::write(
            root.join(cstr!("tsconfig.{name}.json").as_str()),
            serde_json::to_vec(&config).unwrap(),
        )
        .unwrap();
    }

    fn event(&self, leaf: &str) -> FileEvent {
        FileEvent {
            uri: Url::from_file_path(self.root.join(cstr!("tsconfig.{leaf}.json").as_str()))
                .unwrap(),
            typ: FileChangeType::CHANGED,
        }
    }

    fn open_dirty(&self, uri: &Url, marker: &str, version: i32) -> String {
        let source: String = cstr!(
            "<script lang=\"ts\">export const {marker} = 1;</script><template><div /></template>"
        )
        .into();
        self.state
            .documents
            .open(uri.clone(), source.clone(), version, "vue".into());
        source
    }
}

fn assert_dialect(state: &ServerState, version: VueVersion) {
    assert_eq!(
        state.type_checker_vue_version(),
        version,
        "owner={:?}, checker={:?}, authored={:?}",
        state.project_contexts.key.read().as_ref(),
        state.get_type_checker_config(),
        state
            .project_contexts
            .loaded_config
            .read()
            .as_ref()
            .map(|loaded| loaded.features)
    );
    assert_eq!(state.legacy_vue2_enabled(), version == VueVersion::V2_7);
}

fn assert_projection(state: &ServerState, uri: &Url, marker: &str) {
    let virtual_docs = state
        .get_virtual_docs(uri)
        .expect("open source is projected");
    assert!(
        virtual_docs
            .all()
            .iter()
            .any(|document| document.content.contains(marker)),
        "virtual documents should contain the current buffer marker {marker}"
    );
}

#[test]
fn same_directory_references_isolate_dialects_and_reuse_the_effective_project() {
    let fixture = ReferenceFixture::new("export default {};");
    let source = fixture.open_dirty(&fixture.legacy, "unsavedLegacy", 7);
    let legacy = fixture
        .state
        .document_project_state(&fixture.legacy)
        .unwrap();
    let modern = fixture
        .state
        .document_project_state(&fixture.modern)
        .unwrap();
    assert_dialect(&legacy, VueVersion::V2_7);
    assert_dialect(&modern, VueVersion::V3);
    assert!(!Arc::ptr_eq(&legacy, &modern));
    fixture.state.documents.open(
        fixture.modern.clone(),
        "<template>shared mutation</template>".into(),
        9,
        "vue".into(),
    );
    assert_eq!(legacy.documents.version(&fixture.modern), Some(9));
    assert_eq!(modern.documents.version(&fixture.modern), Some(9));
    assert_eq!(modern.documents.text(&fixture.legacy), Some(source));
    assert_eq!(modern.documents.version(&fixture.legacy), Some(7));
    assert_projection(&legacy, &fixture.legacy, "unsavedLegacy");
    assert!(modern.get_virtual_docs(&fixture.legacy).is_none());
    let sibling = Url::from_file_path(fixture.root.join("src/legacy/New.vue")).unwrap();
    assert!(Arc::ptr_eq(
        &legacy,
        &fixture.state.document_project_state(&sibling).unwrap()
    ));
    assert!(Arc::ptr_eq(
        &modern,
        &legacy.document_project_state(&fixture.modern).unwrap()
    ));
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
fn initialized_vite_source_root_keeps_package_reference_ownership() {
    let fixture = ReferenceFixture::new("export default { root: 'src' };");
    assert_eq!(
        fixture.state.get_workspace_root(),
        Some(fixture.root.join("src"))
    );
    let legacy = fixture
        .state
        .document_project_state(&fixture.legacy)
        .unwrap();
    let modern = fixture
        .state
        .document_project_state(&fixture.modern)
        .unwrap();
    assert_dialect(&legacy, VueVersion::V2_7);
    assert_dialect(&modern, VueVersion::V3);
    assert_eq!(
        legacy.get_type_checker_config().tsconfig.as_deref(),
        fixture.root.join("tsconfig.legacy.json").to_str()
    );
    assert_eq!(legacy.get_workspace_root(), Some(fixture.root.join("src")));
}

#[test]
fn explicit_vite_vue_version_overrides_each_referenced_target() {
    let fixture = ReferenceFixture::new("export default { vue: { version: '3' } };");
    assert_dialect(
        &fixture
            .state
            .document_project_state(&fixture.legacy)
            .unwrap(),
        VueVersion::V3,
    );
    assert_dialect(
        &fixture
            .state
            .document_project_state(&fixture.modern)
            .unwrap(),
        VueVersion::V3,
    );
}

#[test]
fn leaf_target_changes_retire_only_its_owner_and_reproject_the_latest_buffer() {
    let fixture = ReferenceFixture::new("export default {};");
    fixture.open_dirty(&fixture.legacy, "firstLegacy", 3);
    let old_legacy = fixture
        .state
        .document_project_state(&fixture.legacy)
        .unwrap();
    let modern = fixture
        .state
        .document_project_state(&fixture.modern)
        .unwrap();
    let pending = old_legacy.corsa_request_stamp();
    let source =
        "<script lang=\"ts\">export const latestLegacy = 2;</script><template><div /></template>";
    assert!(fixture.state.documents.apply_changes(
        &fixture.legacy,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: source.into(),
        }],
        4,
    ));
    ReferenceFixture::write_leaf(&fixture.root, "legacy", "legacy", 3.5);
    assert_eq!(
        fixture
            .state
            .observe_project_config_events(&[fixture.event("legacy")]),
        vec![(fixture.legacy.clone(), 4)]
    );
    let current = fixture
        .state
        .document_project_state(&fixture.legacy)
        .unwrap();
    assert!(old_legacy.project_context_retired());
    assert!(!pending.is_current(&old_legacy));
    assert!(!Arc::ptr_eq(&old_legacy, &current));
    assert_dialect(&current, VueVersion::V3);
    assert!(!modern.project_context_retired());
    assert!(Arc::ptr_eq(
        &modern,
        &fixture
            .state
            .document_project_state(&fixture.modern)
            .unwrap()
    ));
    assert_eq!(
        current.documents.text(&fixture.legacy).as_deref(),
        Some(source)
    );
    assert_eq!(current.documents.version(&fixture.legacy), Some(4));
    assert_projection(&current, &fixture.legacy, "latestLegacy");
}

#[test]
fn reference_membership_changes_move_dirty_sources_to_their_new_owners() {
    let fixture = ReferenceFixture::new("export default {};");
    let legacy_source = fixture.open_dirty(&fixture.legacy, "legacyBuffer", 8);
    let modern_source = fixture.open_dirty(&fixture.modern, "modernBuffer", 9);
    let old_legacy = fixture
        .state
        .document_project_state(&fixture.legacy)
        .unwrap();
    let old_modern = fixture
        .state
        .document_project_state(&fixture.modern)
        .unwrap();
    ReferenceFixture::write_leaf(&fixture.root, "legacy", "modern", 2.7);
    ReferenceFixture::write_leaf(&fixture.root, "modern", "legacy", 3.5);
    let mut affected = fixture
        .state
        .observe_project_config_events(&[fixture.event("legacy"), fixture.event("modern")]);
    affected.sort();
    let mut expected = vec![(fixture.legacy.clone(), 8), (fixture.modern.clone(), 9)];
    expected.sort();
    assert_eq!(affected, expected);
    let new_legacy = fixture
        .state
        .document_project_state(&fixture.legacy)
        .unwrap();
    let new_modern = fixture
        .state
        .document_project_state(&fixture.modern)
        .unwrap();
    assert!(old_legacy.project_context_retired());
    assert!(old_modern.project_context_retired());
    assert_dialect(&new_legacy, VueVersion::V3);
    assert_dialect(&new_modern, VueVersion::V2_7);
    assert_eq!(
        new_legacy.documents.text(&fixture.legacy),
        Some(legacy_source)
    );
    assert_eq!(
        new_modern.documents.text(&fixture.modern),
        Some(modern_source)
    );
    assert_projection(&new_legacy, &fixture.legacy, "legacyBuffer");
    assert_projection(&new_modern, &fixture.modern, "modernBuffer");
    assert!(new_legacy.get_virtual_docs(&fixture.modern).is_none());
    assert!(new_modern.get_virtual_docs(&fixture.legacy).is_none());
}

#[test]
fn referenced_owners_reuse_the_already_evaluated_vite_snapshot() {
    let fixture = ReferenceFixture::new(
        "import { appendFileSync } from 'node:fs';\n\
         appendFileSync(new URL('./evaluations', import.meta.url), '1');\n\
         export default {};",
    );
    let evaluations = fixture.root.join("evaluations");
    assert_eq!(std::fs::read_to_string(&evaluations).unwrap(), "1");
    let legacy = fixture
        .state
        .document_project_state(&fixture.legacy)
        .unwrap();
    let modern = fixture
        .state
        .document_project_state(&fixture.modern)
        .unwrap();
    assert_dialect(&legacy, VueVersion::V2_7);
    assert_dialect(&modern, VueVersion::V3);
    assert_eq!(std::fs::read_to_string(evaluations).unwrap(), "1");
}

#[test]
fn invalid_solution_target_does_not_override_valid_source_projects_or_vite_policy() {
    let mut fixture =
        ReferenceFixture::new("export default { vue: { languageServer: { hover: false } } };");
    std::fs::write(
        fixture.root.join("tsconfig.json"),
        r#"{"files":[],"vueCompilerOptions":{"target":4},"references":[{"path":"./tsconfig.legacy.json"},{"path":"./tsconfig.modern.json"}]}"#,
    )
    .unwrap();
    // Start a fresh editor after writing the invalid solution-only target, so
    // a failed configuration load cannot retain the previous Vite policy.
    fixture.state = Arc::new(ServerState::new());
    fixture.state.set_workspace_root(fixture.root.clone());
    fixture
        .state
        .set_workspace_folders(vec![fixture.root.clone()]);
    fixture.state.load_workspace_config(&fixture.root);
    assert!(!fixture.state.lsp_features().hover);
    let legacy = fixture
        .state
        .document_project_state(&fixture.legacy)
        .unwrap();
    let modern = fixture
        .state
        .document_project_state(&fixture.modern)
        .unwrap();
    assert_dialect(&legacy, VueVersion::V2_7);
    assert_dialect(&modern, VueVersion::V3);
    assert!(!legacy.lsp_features().hover);
    assert!(!modern.lsp_features().hover);
}

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
        assert!(!Arc::ptr_eq(&previous, &current));
        assert!(previous.project_context_retired());
        assert!(!pending.is_current(&previous));
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
        assert!(!Arc::ptr_eq(&previous, &current));
        assert!(previous.project_context_retired());
        assert!(!pending.is_current(&previous));
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

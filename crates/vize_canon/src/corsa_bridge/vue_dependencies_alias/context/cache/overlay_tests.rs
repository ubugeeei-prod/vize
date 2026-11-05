//! Overlay changes invalidate the graphs that read them, preserving native custody.
#![expect(
    clippy::disallowed_types,
    reason = "tests inspect shared immutable context owners"
)]
use super::super::{AliasContext, PreparedAliasContext};
use crate::corsa_bridge::{EditorMirrorSession, vue_document::CorsaProjectEnvironment};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vize_carton::FxHashMap;

const HOST: &str = "<script setup lang='ts'>import Child from './Child.vue'; const token = 1;</script><template><Child />{{ token }}</template>";
const CHILD: &str =
    "<script setup lang='ts'>const child = 'é';</script><template>{{ child }}</template>";
const OTHER: &str =
    "<script setup lang='ts'>const label = '日本語';</script><template>{{ label }}</template>";

struct Workspace {
    _directory: tempfile::TempDir,
    host: PathBuf,
    child: PathBuf,
    other: PathBuf,
    session: EditorMirrorSession,
}
impl Workspace {
    fn new(watched: bool) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        std::fs::write(root.join("tsconfig.json"), r#"{"include":["**/*"]}"#).unwrap();
        let [host, child, other] =
            ["Host.vue", "Child.vue", "Other.vue"].map(|name| root.join(name));
        for (path, source) in [(&host, HOST), (&child, CHILD), (&other, OTHER)] {
            std::fs::write(path, source).unwrap();
        }
        let session = EditorMirrorSession::new();
        if watched {
            session.trust_watched_disk_inputs();
        }
        Self {
            _directory: directory,
            host,
            child,
            other,
            session,
        }
    }
    fn open(
        &self,
        path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
        requested: &[(PathBuf, &str)],
    ) -> PreparedAliasContext {
        AliasContext::for_hosts_cached(
            path,
            content,
            overlays,
            requested,
            Default::default(),
            CorsaProjectEnvironment {
                virtual_ts_options: &Default::default(),
                package_routes: &Default::default(),
                project_root: None,
                tsconfig_path: None,
                editor_session: &self.session,
            },
        )
        .unwrap()
    }
}

#[test]
fn unrelated_overlay_edits_refresh_the_sibling_without_rebuilding_the_host() {
    for watched in [false, true] {
        let workspace = Workspace::new(watched);
        let mut overlays = FxHashMap::from_iter([
            (workspace.host.clone(), HOST),
            (workspace.other.clone(), OTHER),
        ]);
        let original = workspace.open(&workspace.host, HOST, &overlays, &[]);
        let expected = original
            .mirror
            .as_ref()
            .unwrap()
            .preparation_import_facts_snapshot();
        let sibling = workspace.open(&workspace.other, OTHER, &overlays, &[]);
        let sibling_path = sibling.mirror_virtual_path(&workspace.other).unwrap();
        let old_mapping = sibling.source_catalog.get(&sibling_path).unwrap();
        drop(sibling);
        let edited = OTHER.replace("日本語", "different 🦀 prefix");
        overlays.insert(workspace.other.clone(), &edited);
        let reused = workspace.open(&workspace.host, HOST, &overlays, &[]);
        assert!(Arc::ptr_eq(&original.context, &reused.context));
        assert_eq!(reused.mirror.as_ref().unwrap().source_registrations(), 2);
        assert_eq!(
            reused
                .mirror
                .as_ref()
                .unwrap()
                .preparation_import_facts_snapshot(),
            expected
        );
        assert!(reused.materialized_changes.changed.contains(&sibling_path));
        assert!(reused.materialized_changes.created.is_empty());
        assert!(reused.materialized_changes.deleted.is_empty());
        let updated = reused.source_catalog.get(&sibling_path).unwrap();
        assert_eq!(old_mapping.source.as_str(), OTHER);
        assert_eq!(updated.source.as_str(), edited);
        assert_eq!(
            std::fs::read_to_string(&sibling_path).unwrap(),
            updated.code.as_str()
        );
        workspace.session.clear();
        let cold = workspace.open(&workspace.host, HOST, &overlays, &[]);
        assert_eq!(
            cold.mirror
                .as_ref()
                .unwrap()
                .preparation_import_facts_snapshot(),
            expected
        );
    }
}

#[test]
fn a_host_edit_keeps_the_other_revision_dependency_files() {
    let workspace = Workspace::new(false);
    let mut overlays = FxHashMap::from_iter([(workspace.host.clone(), HOST)]);
    let requested = [(workspace.other.clone(), OTHER)];
    let references = workspace.open(&workspace.host, HOST, &overlays, &requested);
    let other_path = references.mirror_virtual_path(&workspace.other).unwrap();
    let before = std::fs::read(&other_path).unwrap();
    drop(references);
    drop(workspace.open(&workspace.host, HOST, &overlays, &[]));
    let edited = HOST.replace("token = 1", "token = 2");
    overlays.insert(workspace.host.clone(), &edited);
    let hover = workspace.open(&workspace.host, &edited, &overlays, &[]);
    assert_eq!(hover.mirror.as_ref().unwrap().source_registrations(), 3);
    assert_eq!(std::fs::read(&other_path).unwrap(), before);
    assert!(hover.materialized_changes.created.is_empty());
    assert!(hover.materialized_changes.deleted.is_empty());
    let references = workspace.open(&workspace.host, &edited, &overlays, &requested);
    assert_eq!(
        references.mirror.as_ref().unwrap().source_registrations(),
        4
    );
    assert_eq!(std::fs::read(&other_path).unwrap(), before);
    assert!(references.materialized_changes.created.is_empty());
    assert!(references.materialized_changes.deleted.is_empty());
}

#[test]
fn multiple_dirty_siblings_keep_the_unchanged_query_and_all_current_native_sources() {
    let workspace = Workspace::new(false);
    let third = workspace.other.with_file_name("Third.vue");
    std::fs::write(&third, OTHER).unwrap();
    let mut overlays = FxHashMap::from_iter([
        (workspace.host.clone(), HOST),
        (workspace.other.clone(), OTHER),
        (third.clone(), OTHER),
    ]);
    let original = workspace.open(&workspace.host, HOST, &overlays, &[]);
    drop(workspace.open(&workspace.other, OTHER, &overlays, &[]));
    drop(workspace.open(&third, OTHER, &overlays, &[]));
    let edited = OTHER.replace("日本語", "a longer 🦀 prefix");
    for path in [&workspace.other, &third] {
        overlays.insert(path.clone(), &edited);
    }
    let unchanged = workspace.open(&workspace.host, HOST, &overlays, &[]);
    assert!(Arc::ptr_eq(&original.context, &unchanged.context));
    for path in [&workspace.other, &third] {
        // Locate exact admitted identities from the member that owns each path.
        let current = workspace.open(path, &edited, &overlays, &[]);
        let materialized = current.mirror_virtual_path(path).unwrap();
        let captured = unchanged.source_catalog.get(&materialized).unwrap();
        assert_eq!(captured.source.as_str(), edited);
        assert_eq!(
            std::fs::read_to_string(materialized).unwrap(),
            captured.code.as_str()
        );
    }
}

#[test]
fn an_imported_overlay_edit_rebuilds_and_matches_a_complete_cold_projection() {
    let workspace = Workspace::new(false);
    let mut overlays = FxHashMap::from_iter([
        (workspace.host.clone(), HOST),
        (workspace.child.clone(), CHILD),
    ]);
    let original = workspace.open(&workspace.host, HOST, &overlays, &[]);
    let edited = CHILD.replace("é", "changed 🦀");
    overlays.insert(workspace.child.clone(), &edited);
    let changed = workspace.open(&workspace.host, HOST, &overlays, &[]);
    assert!(!Arc::ptr_eq(&original.context, &changed.context));
    let expected = changed
        .mirror
        .as_ref()
        .unwrap()
        .preparation_import_facts_snapshot();
    drop(changed);
    workspace.session.clear();
    let cold = workspace.open(&workspace.host, HOST, &overlays, &[]);
    assert_eq!(
        cold.mirror
            .as_ref()
            .unwrap()
            .preparation_import_facts_snapshot(),
        expected
    );
}

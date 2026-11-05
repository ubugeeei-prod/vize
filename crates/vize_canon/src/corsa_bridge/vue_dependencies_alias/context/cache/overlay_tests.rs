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

fn native_documents(
    context: &PreparedAliasContext,
) -> Vec<(vize_carton::String, vize_carton::String)> {
    let mut documents = Vec::new();
    context.source_catalog.extend_native_project_documents(
        context.mirror.as_ref().unwrap().virtual_root(),
        &mut documents,
    );
    documents.sort();
    documents
}

#[test]
fn background_preparation_refreshes_the_sibling_and_preserves_the_queried_graph() {
    let workspace = Workspace::new(true);
    let mut overlays = FxHashMap::from_iter([
        (workspace.host.clone(), HOST),
        (workspace.other.clone(), OTHER),
    ]);
    let original = workspace.open(&workspace.host, HOST, &overlays, &[]);
    drop(workspace.open(&workspace.other, OTHER, &overlays, &[]));
    let edited = OTHER.replace("日本語", "a changed 🦀 prefix");
    overlays.insert(workspace.other.clone(), &edited);
    let environment = CorsaProjectEnvironment {
        virtual_ts_options: &Default::default(),
        package_routes: &Default::default(),
        project_root: None,
        tsconfig_path: None,
        editor_session: &workspace.session,
    };
    assert!(!AliasContext::hosts_are_prepared(
        &workspace.host,
        HOST,
        &overlays,
        &[],
        Default::default(),
        environment
    ));
    AliasContext::prebuild_for_hosts(
        &workspace.host,
        HOST,
        &overlays,
        &[],
        Default::default(),
        environment,
    )
    .unwrap();
    let prepared = workspace.open(&workspace.host, HOST, &overlays, &[]);
    assert!(Arc::ptr_eq(&original.context, &prepared.context));
    assert_eq!(prepared.mirror.as_ref().unwrap().source_registrations(), 2);
    assert!(!prepared.materialized_changes.is_empty());
    let sibling = workspace.open(&workspace.other, &edited, &overlays, &[]);
    let path = sibling.mirror_virtual_path(&workspace.other).unwrap();
    let source = prepared.source_catalog.get(&path).unwrap();
    assert_eq!(source.source.as_str(), edited);
    assert_eq!(std::fs::read_to_string(path).unwrap(), source.code.as_str());
}

#[test]
fn closed_dependency_disk_edits_refresh_open_siblings_before_an_unrelated_query() {
    const B: &str = "<script setup lang='ts'>import { value } from './model'; const label = value;</script><template>{{ label }}</template>";
    const MODEL: &str = "export const value: number = 1;";
    for watched in [false, true] {
        for deleted in [false, true] {
            let workspace = Workspace::new(watched);
            let model = workspace.host.with_file_name("model.ts");
            std::fs::write(&model, MODEL).unwrap();
            std::fs::write(&workspace.other, B).unwrap();
            let overlays = FxHashMap::from_iter([
                (workspace.host.clone(), HOST),
                (workspace.other.clone(), B),
            ]);
            drop(workspace.open(&workspace.host, HOST, &overlays, &[]));
            let sibling = workspace.open(&workspace.other, B, &overlays, &[]);
            let model_path = sibling.mirror_virtual_path(&model).unwrap();
            let old_catalog = sibling.source_catalog.clone();
            let old_model = old_catalog.get(&model_path).unwrap();
            assert_eq!(old_model.source.as_str(), MODEL);
            drop(sibling);
            if deleted {
                std::fs::remove_file(&model).unwrap();
            } else {
                std::fs::write(&model, "export const value: string = 'changed 🦀';").unwrap();
            }
            if watched {
                workspace.session.invalidate_disk_inputs();
            }
            let refreshed = workspace.open(&workspace.host, HOST, &overlays, &[]);
            let current_native = native_documents(&refreshed);
            if deleted {
                assert!(refreshed.source_catalog.get(&model_path).is_none());
                assert!(!model_path.exists());
            } else {
                let current = refreshed.source_catalog.get(&model_path).unwrap();
                assert_eq!(
                    current.source.as_str(),
                    "export const value: string = 'changed 🦀';"
                );
                assert_eq!(
                    std::fs::read_to_string(&model_path).unwrap(),
                    current.code.as_str()
                );
            }
            assert_eq!(old_catalog.get(&model_path).unwrap().source.as_str(), MODEL);
            workspace.session.clear();
            drop(workspace.open(&workspace.host, HOST, &overlays, &[]));
            drop(workspace.open(&workspace.other, B, &overlays, &[]));
            let cold = workspace.open(&workspace.host, HOST, &overlays, &[]);
            assert_eq!(current_native, native_documents(&cold));
        }
    }
}

#[test]
fn shape_changing_host_edits_retire_old_workspace_artifacts_and_keep_old_query_maps() {
    for remove_import in [false, true] {
        let workspace = Workspace::new(false);
        let initial = if remove_import {
            HOST.to_owned()
        } else {
            HOST.replace("lang='ts'", "lang='tsx'")
        };
        let edited = if remove_import {
            HOST.replace("import Child from './Child.vue'; ", "")
                .replace("<Child />", "")
        } else {
            HOST.to_owned()
        };
        let mut overlays = FxHashMap::from_iter([(workspace.host.clone(), initial.as_str())]);
        let requested = [(workspace.other.clone(), OTHER)];
        let references = workspace.open(&workspace.host, &initial, &overlays, &requested);
        let obsolete = if remove_import {
            references.mirror_virtual_path(&workspace.child).unwrap()
        } else {
            references.mirror_virtual_path(&workspace.host).unwrap()
        };
        let old_catalog = references.source_catalog.clone();
        let old_source = old_catalog.get(&obsolete).unwrap();
        drop(references);
        overlays.insert(workspace.host.clone(), &edited);
        let current = workspace.open(&workspace.host, &edited, &overlays, &[]);
        assert!(!obsolete.exists());
        assert!(current.source_catalog.get(&obsolete).is_none());
        assert_eq!(
            old_catalog.get(&obsolete).unwrap().source,
            old_source.source
        );
        let expected = native_documents(&current);
        workspace.session.clear();
        let cold = workspace.open(&workspace.host, &edited, &overlays, &[]);
        assert_eq!(native_documents(&cold), expected);
    }
}

#[test]
fn sibling_shape_changes_retire_old_workspace_suffixes_for_warm_and_cold_callers() {
    for cold_caller in [false, true] {
        let workspace = Workspace::new(false);
        let third = workspace.other.with_file_name("Third.vue");
        let old_third = OTHER.replace("lang='ts'", "lang='tsx'");
        std::fs::write(&third, &old_third).unwrap();
        let mut overlays = FxHashMap::from_iter([
            (workspace.host.clone(), HOST),
            (workspace.other.clone(), OTHER),
            (third.clone(), old_third.as_str()),
        ]);
        if !cold_caller {
            drop(workspace.open(&workspace.host, HOST, &overlays, &[]));
        }
        drop(workspace.open(&workspace.other, OTHER, &overlays, &[]));
        let requested = [(workspace.other.clone(), OTHER)];
        let old_workspace = workspace.open(&third, &old_third, &overlays, &requested);
        let obsolete = old_workspace.mirror_virtual_path(&third).unwrap();
        let old_catalog = old_workspace.source_catalog.clone();
        let old_source = old_catalog.get(&obsolete).unwrap();
        drop(old_workspace);
        let edited_b = OTHER.replace("日本語", "a changed B prefix");
        overlays.insert(workspace.other.clone(), &edited_b);
        overlays.insert(third.clone(), OTHER);
        let current = workspace.open(&workspace.host, HOST, &overlays, &[]);
        assert!(!obsolete.exists());
        assert!(current.source_catalog.get(&obsolete).is_none());
        assert_eq!(
            old_catalog.get(&obsolete).unwrap().source,
            old_source.source
        );
        let native = native_documents(&current);
        workspace.session.clear();
        drop(workspace.open(&workspace.host, HOST, &overlays, &[]));
        drop(workspace.open(&workspace.other, &edited_b, &overlays, &[]));
        drop(workspace.open(&third, OTHER, &overlays, &requested));
        let cold = workspace.open(&workspace.host, HOST, &overlays, &[]);
        assert_eq!(native_documents(&cold), native);
    }
}

#[test]
fn forgetting_a_member_retires_its_files_roots_and_complete_native_delta() {
    let workspace = Workspace::new(true);
    let mut overlays = FxHashMap::from_iter([
        (workspace.host.clone(), HOST),
        (workspace.other.clone(), OTHER),
    ]);
    drop(workspace.open(&workspace.host, HOST, &overlays, &[]));
    let sibling = workspace.open(&workspace.other, OTHER, &overlays, &[]);
    let obsolete = sibling.mirror_virtual_path(&workspace.other).unwrap();
    let old_catalog = sibling.source_catalog.clone();
    drop(sibling);
    AliasContext::forget_cached_sources(&workspace.session, &[workspace.other.clone()]);
    overlays.remove(&workspace.other);
    let current = workspace.open(&workspace.host, HOST, &overlays, &[]);
    assert!(!obsolete.exists());
    assert!(current.source_catalog.get(&obsolete).is_none());
    assert_eq!(current.materialized_changes.deleted, vec![obsolete.clone()]);
    assert!(current.materialized_changes.has_topology_changes());
    assert_eq!(old_catalog.get(&obsolete).unwrap().source.as_str(), OTHER);
    let native = native_documents(&current);
    workspace.session.clear();
    let cold = workspace.open(&workspace.host, HOST, &overlays, &[]);
    assert_eq!(native_documents(&cold), native);
}

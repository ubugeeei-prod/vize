#![expect(
    clippy::disallowed_types,
    reason = "cached contexts are compared by their shared Arc"
)]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use vize_carton::FxHashMap;

use super::super::{AliasContext, PreparedAliasContext};
use crate::corsa_bridge::EditorMirrorSession;
use crate::corsa_bridge::vue_document::CorsaProjectEnvironment;

const HOST: &str = "<script setup lang='ts'>import Child from './Child.vue'; const token = 1;</script><template><Child /></template>";
const CHILD: &str =
    "<script setup lang='ts'>const child = 1;</script><template>{{ child }}</template>";
const OTHER: &str =
    "<script setup lang='ts'>const other = 1;</script><template>{{ other }}</template>";

struct Workspace {
    _directory: tempfile::TempDir,
    host: PathBuf,
    child: PathBuf,
    other: PathBuf,
    session: EditorMirrorSession,
}

impl Workspace {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        std::fs::write(root.join("tsconfig.json"), r#"{"include":["**/*"]}"#).unwrap();
        let [host, child, other] =
            ["Host.vue", "Child.vue", "Other.vue"].map(|name| root.join(name));
        for (path, source) in [(&host, HOST), (&child, CHILD), (&other, OTHER)] {
            std::fs::write(path, source).unwrap();
        }
        Self {
            _directory: directory,
            host,
            child,
            other,
            session: EditorMirrorSession::new(),
        }
    }

    fn open(
        &self,
        overlays: &FxHashMap<PathBuf, &str>,
        requested: &[(PathBuf, &str)],
    ) -> PreparedAliasContext {
        AliasContext::for_hosts_cached(
            &self.host,
            HOST,
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

fn reused(left: &PreparedAliasContext, right: &PreparedAliasContext) -> bool {
    Arc::ptr_eq(&left.context, &right.context)
}

fn materialized(context: &AliasContext, original: &Path) -> String {
    let path = context
        .mirror
        .as_ref()
        .unwrap()
        .preferred_materialized_path_for_original(original)
        .unwrap();
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn host_and_workspace_revisions_of_one_host_are_both_reused() {
    let workspace = Workspace::new();
    let overlays = FxHashMap::from_iter([(workspace.host.clone(), HOST)]);
    let requested = [
        (workspace.child.clone(), CHILD),
        (workspace.other.clone(), OTHER),
    ];
    let references = workspace.open(&overlays, &requested);
    let hover = workspace.open(&overlays, &[]);
    assert!(!reused(&references, &hover));

    let references_again = workspace.open(&overlays, &requested);
    let hover_again = workspace.open(&overlays, &[]);
    assert!(reused(&references, &references_again));
    assert!(reused(&hover, &hover_again));
    // The host-only materialization kept the workspace revision's files.
    assert_eq!(
        materialized(&references_again, &workspace.other),
        materialized(&references, &workspace.other)
    );
}

#[test]
fn an_edit_to_an_imported_sfc_rebuilds_both_revisions() {
    let workspace = Workspace::new();
    let mut overlays = FxHashMap::from_iter([(workspace.host.clone(), HOST)]);
    let requested = [
        (workspace.child.clone(), CHILD),
        (workspace.other.clone(), OTHER),
    ];
    let references = workspace.open(&overlays, &requested);
    let hover = workspace.open(&overlays, &[]);

    let unsaved = CHILD.replace("child = 1", "child = 2");
    overlays.insert(workspace.child.clone(), &unsaved);
    let requested = [
        (workspace.child.clone(), unsaved.as_str()),
        (workspace.other.clone(), OTHER),
    ];
    let hover_unsaved = workspace.open(&overlays, &[]);
    let references_unsaved = workspace.open(&overlays, &requested);
    assert!(!reused(&hover, &hover_unsaved));
    assert!(!reused(&references, &references_unsaved));
    for context in [&hover_unsaved, &references_unsaved] {
        let child = materialized(context, &workspace.child);
        assert_eq!(child.matches("child = 2").count(), 1, "{child}");
    }

    // A saved edit to the closed import is a disk input of both revisions.
    overlays.remove(&workspace.child);
    let requested = [
        (workspace.child.clone(), CHILD),
        (workspace.other.clone(), OTHER),
    ];
    let hover_closed = workspace.open(&overlays, &[]);
    let references_closed = workspace.open(&overlays, &requested);
    let saved = CHILD.replace("child = 1", "child = 3");
    std::fs::write(&workspace.child, &saved).unwrap();
    let requested = [
        (workspace.child.clone(), saved.as_str()),
        (workspace.other.clone(), OTHER),
    ];
    let hover_saved = workspace.open(&overlays, &[]);
    let references_saved = workspace.open(&overlays, &requested);
    assert!(!reused(&hover_closed, &hover_saved));
    assert!(!reused(&references_closed, &references_saved));
    for context in [&hover_saved, &references_saved] {
        let child = materialized(context, &workspace.child);
        assert_eq!(child.matches("child = 3").count(), 1, "{child}");
    }
}

#[test]
fn a_watched_session_rebuilds_both_revisions_after_a_disk_event() {
    let workspace = Workspace::new();
    workspace.session.trust_watched_disk_inputs();
    let overlays = FxHashMap::from_iter([(workspace.host.clone(), HOST)]);
    let requested = [
        (workspace.child.clone(), CHILD),
        (workspace.other.clone(), OTHER),
    ];
    let references = workspace.open(&overlays, &requested);
    let hover = workspace.open(&overlays, &[]);

    let saved = CHILD.replace("child = 1", "child = 4");
    std::fs::write(&workspace.child, &saved).unwrap();
    // The owner's watcher reports the save before the next request.
    workspace.session.invalidate_disk_inputs();
    let requested = [
        (workspace.child.clone(), saved.as_str()),
        (workspace.other.clone(), OTHER),
    ];
    let hover_saved = workspace.open(&overlays, &[]);
    let references_saved = workspace.open(&overlays, &requested);
    assert!(!reused(&hover, &hover_saved));
    assert!(!reused(&references, &references_saved));
    for context in [&hover_saved, &references_saved] {
        let child = materialized(context, &workspace.child);
        assert_eq!(child.matches("child = 4").count(), 1, "{child}");
    }
}

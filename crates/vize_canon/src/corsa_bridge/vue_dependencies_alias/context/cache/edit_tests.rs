//! A host edit replaces the host, not the dependency state it did not change.

use std::path::PathBuf;
use vize_carton::{FxHashMap, String as CompactString};

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

/// One editor revision: the host's buffer, another open buffer and the
/// workspace sources a references query requests.
#[derive(Clone, Copy)]
struct Revision<'a> {
    host: &'a str,
    child: Option<&'a str>,
    workspace: bool,
}

impl<'a> Revision<'a> {
    fn host(host: &'a str) -> Self {
        Self {
            host,
            child: None,
            workspace: false,
        }
    }
}

/// A revision exactly: its generated documents and every mirror file.
type Snapshot = (
    Vec<(PathBuf, CompactString, CompactString)>,
    Vec<(PathBuf, Vec<u8>)>,
);

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

    /// Open the host as the editor does: its buffer is also an overlay.
    fn open(&self, revision: Revision<'_>) -> PreparedAliasContext {
        let mut overlays = FxHashMap::from_iter([(self.host.clone(), revision.host)]);
        overlays.extend(revision.child.map(|child| (self.child.clone(), child)));
        let requested = if revision.workspace {
            vec![(self.child.clone(), CHILD), (self.other.clone(), OTHER)]
        } else {
            Vec::new()
        };
        AliasContext::for_hosts_cached(
            &self.host,
            revision.host,
            &overlays,
            &requested,
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

    /// `context` equals a cold build of the same revision, which registers
    /// `sources` once each.
    fn assert_matches_build(
        &self,
        context: PreparedAliasContext,
        revision: Revision<'_>,
        sources: usize,
    ) {
        let revised = snapshot(&context);
        drop(context);
        self.session.clear();
        let built = self.open(revision);
        assert_eq!(registered(&built), sources);
        assert_eq!(revised, snapshot(&built));
    }
}

/// Sources this context's project has registered since it was created.
fn registered(context: &AliasContext) -> usize {
    context.mirror.as_ref().unwrap().source_registrations()
}

fn snapshot(context: &AliasContext) -> Snapshot {
    let mut documents = context
        .materialized_sources()
        .into_iter()
        .map(|document| {
            let code = document.code.as_str().into();
            (document.materialized_path, document.source, code)
        })
        .collect::<Vec<_>>();
    documents.sort();
    let mut files = context
        .mirror
        .as_ref()
        .unwrap()
        .expected_materialized_files()
        .into_iter()
        .map(|path| {
            let bytes = std::fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect::<Vec<_>>();
    files.sort();
    (documents, files)
}

#[test]
fn an_edit_that_keeps_every_import_registers_only_the_host_again() {
    let workspace = Workspace::new();
    let cold = workspace.open(Revision::host(HOST));
    assert_eq!(registered(&cold), 2, "the host and Child");
    drop(cold);

    let edited = HOST.replace("token = 1", "token = 2");
    let revision = Revision::host(&edited);
    let revised = workspace.open(revision);
    assert_eq!(registered(&revised), 3);
    workspace.assert_matches_build(revised, revision, 2);
}

#[test]
fn a_workspace_edit_that_keeps_every_import_registers_only_the_host_again() {
    let workspace = Workspace::new();
    let cold = workspace.open(Revision {
        workspace: true,
        ..Revision::host(HOST)
    });
    assert_eq!(registered(&cold), 3, "the host and both requested sources");
    drop(cold);

    let edited = HOST.replace("token = 1", "token = 2");
    let revision = Revision {
        workspace: true,
        ..Revision::host(&edited)
    };
    let revised = workspace.open(revision);
    assert_eq!(registered(&revised), 4);
    workspace.assert_matches_build(revised, revision, 3);
}

#[test]
fn an_edit_that_changes_an_import_rebuilds_the_dependency_context() {
    let workspace = Workspace::new();
    drop(workspace.open(Revision::host(HOST)));

    let importing = HOST.replace(
        "const token = 1;",
        "import Other from './Other.vue'; const token = 1; const other = Other;",
    );
    let added = Revision::host(&importing);
    let rebuilt = workspace.open(added);
    assert_eq!(registered(&rebuilt), 3);
    workspace.assert_matches_build(rebuilt, added, 3);

    // Removing the import drops Other from the next revision's graph.
    let removed = workspace.open(Revision::host(HOST));
    assert_eq!(registered(&removed), 2);
    assert_eq!(
        removed
            .mirror
            .as_ref()
            .unwrap()
            .registered_original_paths_sorted(),
        vec![workspace.child.clone(), workspace.host.clone()]
    );
    workspace.assert_matches_build(removed, Revision::host(HOST), 2);
}

#[test]
fn an_edit_to_another_open_buffer_rebuilds_the_host_revision() {
    let workspace = Workspace::new();
    drop(workspace.open(Revision {
        child: Some(CHILD),
        ..Revision::host(HOST)
    }));
    let unsaved = CHILD.replace("child = 1", "child = 2");
    let revision = Revision {
        child: Some(&unsaved),
        ..Revision::host(HOST)
    };
    let rebuilt = workspace.open(revision);
    assert_eq!(registered(&rebuilt), 2);
    workspace.assert_matches_build(rebuilt, revision, 2);
}

/// Only a revision handed to its successor survives the next one; a rebuild
/// releases the revision it replaces before building.
#[test]
fn a_rebuild_never_holds_the_revision_it_replaces() {
    use crate::batch::virtual_project::live_projects::{live_projects, take_peak_projects};
    let workspace = Workspace::new();
    drop(workspace.open(Revision {
        child: Some(CHILD),
        ..Revision::host(HOST)
    }));
    assert_eq!(live_projects(), 1);
    take_peak_projects();

    let unsaved = CHILD.replace("child = 1", "child = 2");
    drop(workspace.open(Revision {
        child: Some(&unsaved),
        ..Revision::host(HOST)
    }));
    assert_eq!(
        (take_peak_projects(), live_projects()),
        (1, 1),
        "other buffer"
    );

    let importing = HOST.replace("const token = 1;", "import './Other.vue'; const token = 1;");
    drop(workspace.open(Revision {
        child: Some(&unsaved),
        ..Revision::host(&importing)
    }));
    assert_eq!((take_peak_projects(), live_projects()), (1, 1), "import");

    let edited = importing.replace("token = 1", "token = 2");
    let revised = workspace.open(Revision {
        child: Some(&unsaved),
        ..Revision::host(&edited)
    });
    assert_eq!((take_peak_projects(), live_projects()), (1, 1), "revision");
    assert_eq!(registered(&revised), 4);
}

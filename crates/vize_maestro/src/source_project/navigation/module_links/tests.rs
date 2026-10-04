//! Whole responses join original worker operands and the actual server host.
use super::{ModuleDocumentLinkError, Refusal};
use crate::{
    document::DocumentStore,
    runtime::block_on,
    server::{ModuleLinkContextError, ServerState},
    source_project::{SourceQueryProject, navigation::NativeNavigationProject},
};
use std::sync::Arc;
use tower_lsp::lsp_types::{DocumentLink, Position, Range, Url};

#[cfg(all(feature = "native", unix))]
mod publication;

fn error_context(result: Result<Vec<DocumentLink>, ModuleDocumentLinkError>) {
    assert!(matches!(
        result,
        Err(ModuleDocumentLinkError(Refusal::Publication(
            crate::source_project::ModuleLinkPublicationError::Context(
                ModuleLinkContextError::HostUnavailable
            )
        )))
    ));
}

#[test]
fn module_link_consumer_borrowed_and_shared_hosts_never_promote_original_zero_operands() {
    let uri = Url::parse("file:///original.ts").unwrap();
    let documents = DocumentStore::new();
    let borrowed = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let shared = NativeNavigationProject::new(SourceQueryProject::new_shared(Arc::new(
        DocumentStore::new(),
    )));
    for project in [&borrowed, &shared] {
        project.source.open(
            uri.clone(),
            "const local=1;local;".into(),
            17,
            "javascript".into(),
        );
        error_context(block_on(project.module_document_links(&uri)));
        // The independent, originally admitted local query remains available.
        assert_eq!(
            block_on(project.definition(&uri, Position::new(0, 15))).unwrap(),
            Some(tower_lsp::lsp_types::Location::new(
                uri.clone(),
                Range::new(Position::new(0, 6), Position::new(0, 11))
            ))
        );
    }
}

#[test]
fn module_link_consumer_genuine_source_unavailability_precedes_any_context_or_target() {
    let state = Arc::new(ServerState::new());
    let project = NativeNavigationProject::new(SourceQueryProject::new_server(state));
    let uri = Url::parse("file:///original.ts").unwrap();
    assert!(matches!(
        block_on(project.module_document_links(&uri)),
        Err(ModuleDocumentLinkError(Refusal::Navigation(
            crate::source_project::navigation::NavigationRefusal::Host(
                crate::source_project::SnapshotRefusal::MissingDocument
            )
        )))
    ));
}

#[cfg(all(feature = "native", unix))]
fn project(
    source: &str,
) -> (
    tempfile::TempDir,
    std::path::PathBuf,
    Url,
    Arc<ServerState>,
    NativeNavigationProject<'static>,
) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("child.ts"), "export const value=1;").unwrap();
    std::fs::write(root.join("second.vue"), "<template/>").unwrap();
    let uri = Url::from_file_path(root.join("original.ts")).unwrap();
    let state = Arc::new(ServerState::new());
    state.set_workspace_root(root.clone());
    let project = NativeNavigationProject::new(SourceQueryProject::new_server(Arc::clone(&state)));
    project
        .source
        .open(uri.clone(), source.into(), 17, "typescript".into());
    (dir, root, uri, state, project)
}

#[cfg(all(feature = "native", unix))]
fn link(line: u32, start: u32, end: u32, target: Url) -> DocumentLink {
    DocumentLink {
        range: Range::new(Position::new(line, start), Position::new(line, end)),
        target: Some(target),
        tooltip: None,
        data: None,
    }
}

#[cfg(all(feature = "native", unix))]
#[test]
fn module_link_consumer_complete_original_vectors_reuse_the_local_definition_worker() {
    let (_dir, root, uri, _state, project) = project(
        "/*😀*/import './child.ts';\r\nexport {alpha,beta} from './second.vue';\r\nconst local=1;local;",
    );
    let expected = vec![
        link(
            0,
            13,
            25,
            Url::from_file_path(root.join("child.ts")).unwrap(),
        ),
        link(
            1,
            25,
            39,
            Url::from_file_path(root.join("second.vue")).unwrap(),
        ),
    ];
    assert_eq!(
        block_on(project.module_document_links(&uri)).unwrap(),
        expected
    );
    let worker = Arc::clone(
        project
            .workers
            .lock()
            .get(&uri)
            .unwrap()
            .result
            .as_ref()
            .unwrap(),
    );
    let before = block_on(worker.inspect(Position::new(2, 15))).unwrap();
    assert_eq!(
        block_on(project.definition(&uri, Position::new(2, 15))).unwrap(),
        Some(tower_lsp::lsp_types::Location::new(
            uri.clone(),
            Range::new(Position::new(2, 6), Position::new(2, 11))
        ))
    );
    assert_eq!(
        block_on(project.module_document_links(&uri)).unwrap(),
        expected
    );
    assert!(Arc::ptr_eq(
        &worker,
        project
            .workers
            .lock()
            .get(&uri)
            .unwrap()
            .result
            .as_ref()
            .unwrap()
    ));
    assert_eq!(
        block_on(worker.inspect(Position::new(2, 15))).unwrap(),
        before
    );
    assert_eq!(worker.counts().0, 1);
}

#[cfg(all(feature = "native", unix))]
#[test]
fn module_link_consumer_applied_root_aba_refuses_outside_source_without_reparsing() {
    let (_dir, root, uri, state, project) = project("import './child.ts';");
    let expected = vec![link(
        0,
        7,
        19,
        Url::from_file_path(root.join("child.ts")).unwrap(),
    )];
    assert_eq!(
        block_on(project.module_document_links(&uri)).unwrap(),
        expected
    );
    let worker = Arc::clone(
        project
            .workers
            .lock()
            .get(&uri)
            .unwrap()
            .result
            .as_ref()
            .unwrap(),
    );
    let other = tempfile::tempdir().unwrap();
    state.set_workspace_root(other.path().canonicalize().unwrap());
    assert!(matches!(
        block_on(project.module_document_links(&uri)),
        Err(ModuleDocumentLinkError(Refusal::Targets(
            crate::server::ModuleTargetError::Policy(crate::server::TargetPolicy::SourceParent)
        )))
    ));
    state.set_workspace_root(root);
    assert_eq!(
        block_on(project.module_document_links(&uri)).unwrap(),
        expected
    );
    assert!(Arc::ptr_eq(
        &worker,
        project
            .workers
            .lock()
            .get(&uri)
            .unwrap()
            .result
            .as_ref()
            .unwrap()
    ));
    assert_eq!(worker.counts().0, 1);
}

#[cfg(all(feature = "native", unix))]
#[test]
fn module_link_consumer_missing_late_target_discards_prefix_then_observes_actual_created_file() {
    let (_dir, root, uri, _state, project) =
        project("import './child.ts';\nimport './missing.ts';");
    assert!(matches!(
        block_on(project.module_document_links(&uri)),
        Err(ModuleDocumentLinkError(Refusal::Targets(
            crate::server::ModuleTargetError::Io { .. }
        )))
    ));
    let worker = Arc::clone(
        project
            .workers
            .lock()
            .get(&uri)
            .unwrap()
            .result
            .as_ref()
            .unwrap(),
    );
    std::fs::write(root.join("missing.ts"), "export const actual=2;").unwrap();
    assert_eq!(
        block_on(project.module_document_links(&uri)).unwrap(),
        vec![
            link(
                0,
                7,
                19,
                Url::from_file_path(root.join("child.ts")).unwrap()
            ),
            link(
                1,
                7,
                21,
                Url::from_file_path(root.join("missing.ts")).unwrap()
            ),
        ]
    );
    assert!(Arc::ptr_eq(
        &worker,
        project
            .workers
            .lock()
            .get(&uri)
            .unwrap()
            .result
            .as_ref()
            .unwrap()
    ));
    assert_eq!(worker.counts().0, 1);
}

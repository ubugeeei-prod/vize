//! Prepared data is not publication; real query custody precedes context errors.
use super::super::{block_on, uri};
use super::{Arc, DocumentStore, ModuleLinkContextError, ServerState, SourceQueryProject};
use crate::source_project::{ModuleLinkPublicationError, SnapshotRefusal};

const SOURCE: &str = "/*😀*/import './child.ts';\r\n";
const URI: &str = "file:///actual-source-host/original.ts";

fn server() -> (Arc<ServerState>, SourceQueryProject<'static>) {
    let state = Arc::new(ServerState::new());
    let project = SourceQueryProject::new_server(Arc::clone(&state));
    project.open(uri(URI), SOURCE.into(), 17, "typescript".into());
    (state, project)
}

#[test]
fn module_link_prepared_borrow_keeps_the_same_owned_data_and_original_cancel_lease() {
    let (_, project) = server();
    let (query, cancel) = project.begin_query(&uri(URI)).unwrap();
    let ready = block_on(query.run(|snapshot| async move {
        (
            snapshot.uri().clone(),
            snapshot.version(),
            snapshot.language_id().to_owned(),
            snapshot.source().to_owned(),
        )
    }))
    .unwrap();
    assert!(core::ptr::eq(ready.prepared(), ready.prepared()));
    assert_eq!(
        ready.prepared(),
        &(uri(URI), 17, "typescript".into(), SOURCE.into())
    );
    cancel.abort();
    assert_eq!(
        ready.capture_current_module_link_context().err(),
        Some(ModuleLinkPublicationError::Source(
            SnapshotRefusal::Cancelled
        ))
    );
    let mut published = false;
    assert_eq!(
        ready.publish(|_| published = true),
        Err(SnapshotRefusal::Cancelled)
    );
    assert!(!published);
}

#[test]
fn module_link_ready_context_checks_stale_source_before_unavailable_context() {
    let (state, project) = server();
    let (query, _) = project.begin_query(&uri(URI)).unwrap();
    let ready =
        block_on(query.run(|snapshot| async move { snapshot.source().to_owned() })).unwrap();
    // Actual store mutation deliberately does not notify this query controller.
    state
        .documents
        .open(uri(URI), SOURCE.into(), 18, "typescript".into());
    assert_eq!(ready.prepared(), SOURCE);
    assert_eq!(
        ready.capture_current_module_link_context().err(),
        Some(ModuleLinkPublicationError::Source(
            SnapshotRefusal::Superseded
        ))
    );
    let mut published = false;
    assert_eq!(
        ready.publish(|_| published = true),
        Err(SnapshotRefusal::Superseded)
    );
    assert!(!published);
}

#[test]
fn module_link_ready_borrowed_and_shared_hosts_cannot_mint_server_context() {
    let documents = DocumentStore::new();
    let borrowed = SourceQueryProject::new(&documents);
    let shared = SourceQueryProject::new_shared(Arc::new(DocumentStore::new()));
    for project in [&borrowed, &shared] {
        project.open(uri(URI), SOURCE.into(), 17, "typescript".into());
        let (query, _) = project.begin_query(&uri(URI)).unwrap();
        let ready =
            block_on(query.run(|snapshot| async move {
                (snapshot.uri().clone(), snapshot.source().to_owned())
            }))
            .unwrap();
        assert_eq!(ready.prepared(), &(uri(URI), SOURCE.into()));
        assert_eq!(
            ready.capture_current_module_link_context().err(),
            Some(ModuleLinkPublicationError::Context(
                ModuleLinkContextError::HostUnavailable
            ))
        );
    }
}

#[cfg(feature = "native")]
#[test]
fn module_link_ready_context_uses_its_original_server_applied_root_without_io() {
    let (state, project) = server();
    let (query, _) = project.begin_query(&uri(URI)).unwrap();
    let ready =
        block_on(query.run(|snapshot| async move { snapshot.source().to_owned() })).unwrap();
    assert_eq!(
        ready.capture_current_module_link_context().err(),
        Some(ModuleLinkPublicationError::Context(
            ModuleLinkContextError::MissingRoot
        ))
    );
    state.set_workspace_root("/actual-source-host".into());
    let context = ready.capture_current_module_link_context().unwrap();
    assert_eq!(context.root(), std::path::Path::new("/actual-source-host"));
    assert_eq!(
        ready.publish_with_module_link_context(&context, |value| value),
        Ok(SOURCE.into())
    );
}

#[cfg(not(feature = "native"))]
#[test]
fn module_link_ready_minimal_context_remains_genuinely_native_unavailable() {
    let (_, project) = server();
    let (query, _) = project.begin_query(&uri(URI)).unwrap();
    let ready =
        block_on(query.run(|snapshot| async move { snapshot.source().to_owned() })).unwrap();
    assert_eq!(ready.prepared(), SOURCE);
    assert_eq!(
        ready.capture_current_module_link_context().err(),
        Some(ModuleLinkPublicationError::Context(
            ModuleLinkContextError::NativeUnavailable
        ))
    );
}

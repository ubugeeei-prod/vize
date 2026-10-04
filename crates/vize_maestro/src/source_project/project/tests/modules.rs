//! Actual host and whole publication refusals; no resolver or target authority.
use super::{Arc, DocumentStore, SourceQueryProject};
#[cfg(feature = "native")]
use super::{block_on, uri};
use crate::server::{ModuleLinkContextError, ServerState};
#[cfg(feature = "native")]
mod publication;
#[cfg(all(feature = "native", unix))]
mod targets;

#[test]
fn module_link_borrowed_shared_and_minimal_hosts_cannot_fabricate_project_context() {
    let store = DocumentStore::new();
    let borrowed = SourceQueryProject::new(&store);
    let shared = SourceQueryProject::new_shared(Arc::new(DocumentStore::new()));
    for project in [&borrowed, &shared] {
        assert!(matches!(
            project.capture_module_link_context(),
            Err(ModuleLinkContextError::HostUnavailable)
        ));
    }
    let state = Arc::new(ServerState::new());
    let server = SourceQueryProject::new_server(state);
    #[cfg(feature = "native")]
    assert!(matches!(
        server.capture_module_link_context(),
        Err(ModuleLinkContextError::MissingRoot)
    ));
    #[cfg(not(feature = "native"))]
    assert!(matches!(
        server.capture_module_link_context(),
        Err(ModuleLinkContextError::NativeUnavailable)
    ));
}

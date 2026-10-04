use super::{
    Arc, ModuleLinkContextError, ModuleLinkRetirement, ModuleLinkTerminationLease, ServerState,
    retired,
};
#[cfg(feature = "native")]
use crate::server::MaestroServer;
use crate::{runtime::block_on, server::build_lsp_service};
use serde_json::json;
use tower::Service;
use tower_lsp::{LanguageServer, jsonrpc::Request};

#[test]
fn module_link_foreground_drop_and_transport_completion_retire_with_state_arc_retained() {
    let (service, socket) = build_lsp_service();
    let state = Arc::clone(&service.inner().state);
    let outer = service.inner().module_link_transport_lease().unwrap();
    drop(outer);
    retired(&state, ModuleLinkRetirement::TransportEnded);
    drop(service);
    drop(socket);
    retired(&state, ModuleLinkRetirement::TransportEnded);

    let (service, socket) = build_lsp_service();
    let state = Arc::clone(&service.inner().state);
    drop(service);
    retired(&state, ModuleLinkRetirement::ForegroundDropped);
    drop(socket);
}

#[test]
fn module_link_shutdown_preserves_complete_rpc_result_and_retires_before_foreground_drop() {
    let (mut service, socket) = build_lsp_service();
    let state = Arc::clone(&service.inner().state);
    initialize_disabled(&mut service);
    block_on(async {
        let shutdown: Request =
            serde_json::from_value(json!({"jsonrpc":"2.0","id":2,"method":"shutdown"})).unwrap();
        futures::future::poll_fn(|cx| service.poll_ready(cx))
            .await
            .unwrap();
        let response = service.call(shutdown).await.unwrap().unwrap();
        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({"jsonrpc":"2.0","id":2,"result":null})
        );
    });
    retired(&state, ModuleLinkRetirement::Shutdown);
    assert_eq!(block_on(service.inner().shutdown()), Ok(()));
    drop(service);
    drop(socket);
    retired(&state, ModuleLinkRetirement::Shutdown);
}

#[test]
fn module_link_background_owner_absence_and_weak_lease_do_not_close_or_cycle_real_session() {
    let (service, socket) = build_lsp_service();
    let state = Arc::clone(&service.inner().state);
    // Same actual diagnostic-worker constructor used before its spawn. Dropping
    // it models the owned closure's destruction on failed handoff, not an OS
    // spawn-failure execution claim.
    #[cfg(feature = "native")]
    {
        let worker =
            MaestroServer::diagnostic_worker(service.inner().client.clone(), Arc::clone(&state));
        assert!(worker.module_link_termination.is_none());
        assert!(worker.module_link_transport_lease().is_none());
        drop(worker);
    }
    assert!(!matches!(
        state.capture_module_link_context(),
        Err(ModuleLinkContextError::Retired(_))
    ));
    drop(socket);
    drop(service);
    retired(&state, ModuleLinkRetirement::ForegroundDropped);
    let detached = Arc::new(ServerState::new());
    let weak = Arc::downgrade(&detached);
    let lease = ModuleLinkTerminationLease::new(&detached, ModuleLinkRetirement::TransportEnded);
    drop(detached);
    assert!(weak.upgrade().is_none());
    drop(lease);
}

pub(super) fn initialize_disabled(
    service: &mut tower_lsp::LspService<crate::server::MaestroServer>,
) {
    let initialize: Request = serde_json::from_value(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"initializationOptions":{"enabled":false}}})).unwrap();
    block_on(async {
        futures::future::poll_fn(|cx| service.poll_ready(cx))
            .await
            .unwrap();
        let response = service.call(initialize).await.unwrap().unwrap();
        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({
                "jsonrpc":"2.0","id":1,"result":{
                    "capabilities":{
                        "textDocumentSync":{"openClose":true,"change":2,"willSave":false,"willSaveWaitUntil":false,"save":{"includeText":false}},
                        "workspace":{"workspaceFolders":{"supported":true,"changeNotifications":true}},
                        "experimental":{"vize":{"jsxTypecheck":false}}
                    },
                    "serverInfo":{"name":"vize-maestro","version":env!("CARGO_PKG_VERSION")}
                }
            })
        );
    });
}

#[test]
fn module_link_actual_exit_signal_preserves_notification_and_retires_transport_lease() {
    let (mut service, socket) = build_lsp_service();
    initialize_disabled(&mut service);
    let state = Arc::clone(&service.inner().state);
    let outer = service.inner().module_link_transport_lease().unwrap();
    let (mut service, exit) = crate::server::with_exit_signal(service);
    block_on(async {
        futures::future::poll_fn(|cx| service.poll_ready(cx))
            .await
            .unwrap();
        assert!(
            service
                .call(serde_json::from_value(json!({"jsonrpc":"2.0","method":"exit"})).unwrap())
                .await
                .unwrap()
                .is_none()
        );
        exit.await;
    });
    // Actual serve_transport drops this outer lease when the same select wins.
    drop(outer);
    retired(&state, ModuleLinkRetirement::TransportEnded);
    drop(service);
    drop(socket);
    retired(&state, ModuleLinkRetirement::TransportEnded);
}

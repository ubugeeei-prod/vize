//! Actual package-target answers still cross the production stale/cancel gate.

use crate::{
    ide::{IdeContext, definition::component_import::resolve_native_component_file},
    server::build_lsp_service,
    test_support::native_component::Workspace,
};
use futures::{
    channel::oneshot,
    future::{AbortHandle, Abortable},
};
use tower_lsp::jsonrpc::Error;

#[test]
fn resolved_native_component_targets_are_refused_after_source_or_configuration_changes() {
    crate::runtime::block_on(async {
        let Some(runtime) = crate::test_support::tsgo() else {
            return;
        };
        for configuration in [false, true] {
            let workspace = Workspace::new(false);
            let source = Workspace::source("Alpha");
            let (service, _socket) = build_lsp_service();
            let server = service.inner();
            workspace.open(&server.state, &source, 1);
            let bridge = workspace.bridge(runtime.clone()).await;
            let ctx = IdeContext::new(
                &server.state,
                &workspace.uri,
                source.rfind("<Child").unwrap() + 2,
            )
            .unwrap();
            let (ready, computed) = oneshot::channel();
            let (release, held) = oneshot::channel();
            let request = server.native_request(async {
                let target = resolve_native_component_file(&ctx, "Child", &bridge).await;
                ready.send(target.clone()).unwrap();
                held.await.unwrap();
                Ok(target)
            });
            let revoke = async {
                assert_eq!(computed.await.unwrap(), Some(workspace.target("Alpha")));
                if configuration {
                    server.state.apply_lsp_initialization_options(Some(
                        &serde_json::json!({"typecheck":false}),
                    ));
                } else {
                    workspace.open(&server.state, &Workspace::source("Bravo"), 2);
                }
                release.send(()).unwrap();
            };
            let (response, ()) = futures::join!(request, revoke);
            assert_eq!(response, Err(Error::content_modified()));
            bridge.shutdown().await.unwrap();
        }
    });
}

#[test]
fn cancelling_an_admitted_component_fallback_releases_the_native_scope_for_the_new_revision() {
    crate::runtime::block_on(async {
        let Some(runtime) = crate::test_support::tsgo() else {
            return;
        };
        let workspace = Workspace::new(false);
        let source = Workspace::source("Alpha");
        let (service, _socket) = build_lsp_service();
        let server = service.inner();
        workspace.open(&server.state, &source, 1);
        let bridge = workspace.bridge(runtime).await;
        let ctx = IdeContext::new(
            &server.state,
            &workspace.uri,
            source.rfind("<Child").unwrap() + 2,
        )
        .unwrap();
        let (ready, computed) = oneshot::channel();
        let request = server.native_request(async {
            let target = resolve_native_component_file(&ctx, "Child", &bridge).await;
            ready.send(target).unwrap();
            std::future::pending::<Result<Option<std::path::PathBuf>, Error>>().await
        });
        let (abort, registration) = AbortHandle::new_pair();
        let pending = Abortable::new(request, registration);
        let cancel = async {
            assert_eq!(computed.await.unwrap(), Some(workspace.target("Alpha")));
            abort.abort();
        };
        let (cancelled, ()) = futures::join!(pending, cancel);
        assert!(cancelled.is_err());

        let changed = Workspace::source("Bravo");
        workspace.open(&server.state, &changed, 2);
        let current = IdeContext::new(
            &server.state,
            &workspace.uri,
            changed.rfind("<Child").unwrap() + 2,
        )
        .unwrap();
        let recovered = server
            .native_request(async {
                Ok(resolve_native_component_file(&current, "Child", &bridge).await)
            })
            .await;
        assert_eq!(recovered, Ok(Some(workspace.target("Bravo"))));
        bridge.shutdown().await.unwrap();
    });
}

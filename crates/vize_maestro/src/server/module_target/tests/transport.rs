//! Actual held physical observations survive until real tower-lsp input drain.
#![expect(
    clippy::disallowed_types,
    reason = "real transport futures retain original Arc owners"
)]
use super::events::session;
use crate::{
    runtime::block_on,
    server::{
        MaestroServer, ModuleLinkContextError, ModuleLinkRetirement, ModuleTargetError,
        WatcherCoverageError,
    },
    source_project::SourceQueryProject,
};
use futures::{FutureExt, channel::oneshot, future::BoxFuture, task::noop_waker_ref};
use parking_lot::Mutex;
use serde_json::json;
use std::{
    sync::Arc,
    task::{Context, Poll},
};
use tower::Service;
use tower_lsp::{
    LspService, Server,
    jsonrpc::{Request, Response},
    lsp_types::Url,
};
mod io;
use io::{Input, InputState, Output, frame, frames};

struct Hold {
    entered: oneshot::Sender<()>,
    release: oneshot::Receiver<()>,
}
struct HeldService {
    inner: LspService<MaestroServer>,
    project: Arc<SourceQueryProject<'static>>,
    source: Url,
    native: Option<Hold>,
    legacy: Option<Hold>,
}
impl Service<Request> for HeldService {
    type Response = Option<Response>;
    type Error = <LspService<MaestroServer> as Service<Request>>::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    fn call(&mut self, request: Request) -> Self::Future {
        if request.method() == "test/heldPhysicalTargets" {
            let id = request.id().unwrap().clone();
            let context = self.project.capture_module_link_context().unwrap();
            let (query, _) = self.project.begin_query(&self.source).unwrap();
            let Hold { entered, release } = self.native.take().unwrap();
            async move {
                let ready = query.run(|snapshot| async move {
                    json!({"version":snapshot.version(),"source":snapshot.source(),"language":snapshot.language_id()})
                }).await.unwrap();
                let checked = ready.observe_module_targets(&context, &["./child.ts"])
                    .unwrap().recheck().unwrap();
                assert_eq!(checked.require_watcher_covered(), Err(WatcherCoverageError::UnknownCoverage));
                entered.send(()).unwrap();
                release.await.unwrap();
                let result = ready.publish_with_module_targets(checked, |source, targets| {
                    json!({"source":source,"targets":targets})
                });
                let value = match result {
                    Err(ModuleTargetError::Context(ModuleLinkContextError::Retired(ModuleLinkRetirement::InputEof))) => json!({"refused":"input-eof"}),
                    Err(ModuleTargetError::Context(ModuleLinkContextError::Retired(ModuleLinkRetirement::InputError))) => json!({"refused":"input-error"}),
                    Ok(value) => value,
                    error => panic!("unexpected authentic physical publication: {error:?}"),
                };
                Ok(Some(Response::from_ok(id, value)))
            }.boxed()
        } else {
            let actual = self.inner.call(request);
            let Hold { entered, release } = self.legacy.take().unwrap();
            async move {
                let response = actual.await?;
                entered.send(()).unwrap();
                release.await.unwrap();
                Ok(response)
            }
            .boxed()
        }
    }
}

#[test]
fn module_link_target_actual_eof_error_codec_and_drop_preserve_complete_legacy_drain() {
    for mode in 0..4 {
        let session = session();
        session
            .state
            .apply_lsp_initialization_options(Some(&json!({"documentLinks":true})));
        let context = session.project.capture_module_link_context().unwrap();
        let project = Arc::new(session.project);
        let (native_enter, native_entered) = oneshot::channel();
        let (native_release, native_released) = oneshot::channel();
        let (legacy_enter, legacy_entered) = oneshot::channel();
        let (legacy_release, legacy_released) = oneshot::channel();
        let bytes = [
            frame(&json!({"jsonrpc":"2.0","id":2,"method":"test/heldPhysicalTargets"})),
            frame(&json!({"jsonrpc":"2.0","id":3,"method":"textDocument/documentLink","params":{"textDocument":{"uri":session.source}}})),
        ].concat();
        let input = Arc::new(Mutex::new(InputState {
            bytes: bytes.into(),
            ended: None,
            waker: None,
        }));
        let output = Arc::new(Mutex::new(Vec::new()));
        let outer = session
            .service
            .inner()
            .module_link_transport_lease()
            .unwrap();
        let read = crate::server::module_input::ModuleLinkInput::new(
            Input(Arc::clone(&input)),
            session.service.inner().module_link_transport_lease(),
        );
        let held = HeldService {
            inner: session.service,
            project,
            source: session.source.clone(),
            native: Some(Hold {
                entered: native_enter,
                release: native_released,
            }),
            legacy: Some(Hold {
                entered: legacy_enter,
                release: legacy_released,
            }),
        };
        let mut transport =
            Box::pin(Server::new(read, Output(Arc::clone(&output)), session.socket).serve(held));
        block_on(async {
            let entered = futures::future::join(native_entered, legacy_entered);
            match futures::future::select(transport.as_mut(), Box::pin(entered)).await {
                futures::future::Either::Right(((a, b), _)) => {
                    a.unwrap();
                    b.unwrap();
                }
                _ => panic!("both actual futures must be held before input termination"),
            }
        });
        assert_eq!(
            session
                .state
                .with_current_module_link_context(&context, || 7),
            Ok(7)
        );
        if mode == 2 {
            drop(transport);
            assert!(matches!(
                session.state.capture_module_link_context(),
                Err(ModuleLinkContextError::Retired(_))
            ));
            assert!(output.lock().is_empty());
            assert!(native_release.send(()).is_err());
            assert!(legacy_release.send(()).is_err());
            drop(outer);
            continue;
        }
        let wake = {
            let mut reader = input.lock();
            if mode == 3 {
                reader.bytes.extend(b"Content-Length: 1\r\n\r\n{");
            } else {
                reader.ended = Some(mode == 1);
            }
            reader.waker.take()
        };
        if let Some(wake) = wake {
            wake.wake();
        }
        let mut cx = Context::from_waker(noop_waker_ref());
        assert!(std::future::Future::poll(transport.as_mut(), &mut cx).is_pending());
        if mode == 3 {
            assert!(session.state.capture_module_link_context().is_ok());
        } else {
            assert_eq!(
                session.state.capture_module_link_context().err(),
                Some(ModuleLinkContextError::Retired(if mode == 1 {
                    ModuleLinkRetirement::InputError
                } else {
                    ModuleLinkRetirement::InputEof
                }))
            );
        }
        assert!(
            frames(&output.lock())
                .iter()
                .all(|value| value["id"].is_null())
        );
        native_release.send(()).unwrap();
        legacy_release.send(()).unwrap();
        block_on(transport);
        let mut expected = vec![
            json!({"jsonrpc":"2.0","id":2,"result":if mode==3 {
                json!({"source":{"version":7,"source":"const value=1;value;","language":"javascript"},"targets":[Url::from_file_path(session.root.join("child.ts")).unwrap()]})
            } else {json!({"refused":if mode==1 {"input-error"} else {"input-eof"}})}}),
            json!({"jsonrpc":"2.0","id":3,"result":null}),
        ];
        if mode == 1 || mode == 3 {
            expected.insert(
                0,
                json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}),
            );
        }
        assert_eq!(frames(&output.lock()), expected);
        assert_eq!(
            session.state.documents.text(&session.source).as_deref(),
            Some("const value=1;value;")
        );
        drop(outer);
        assert_eq!(
            session.state.capture_module_link_context().err(),
            Some(ModuleLinkContextError::Retired(match mode {
                3 => ModuleLinkRetirement::TransportEnded,
                1 => ModuleLinkRetirement::InputError,
                _ => ModuleLinkRetirement::InputEof,
            }))
        );
    }
}

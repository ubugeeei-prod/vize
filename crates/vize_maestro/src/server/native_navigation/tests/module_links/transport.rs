//! Held actual native RPC futures refuse before the original transport drains.
#![expect(
    clippy::disallowed_types,
    reason = "real transport futures retain original Arc server and byte-stream owners"
)]
use super::{MaestroServer, files, initialize_only, links};
use crate::{
    runtime::block_on,
    server::{ModuleLinkContextError, ModuleLinkRetirement, module_input::ModuleLinkInput},
};
use futures::{FutureExt, channel::oneshot, future::BoxFuture, task::noop_waker_ref};
use parking_lot::Mutex;
use serde_json::json;
use std::{future::Future, sync::Arc, task::Context};
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
    native: Option<Hold>,
    legacy: Option<Hold>,
}
impl Service<Request> for HeldService {
    type Response = Option<Response>;
    type Error = <LspService<MaestroServer> as Service<Request>>::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    fn call(&mut self, request: Request) -> Self::Future {
        // Every request, including the native one, uses the actual registered
        // production dispatcher. The wrapper only parks polling in this law.
        let native = request.method() == "vize/nativeModuleDocumentLinks";
        let mut actual = Box::pin(self.inner.call(request));
        let Hold { entered, release } = if native {
            self.native.take().unwrap()
        } else {
            self.legacy.take().unwrap()
        };
        async move {
            if native {
                futures::future::poll_fn(|cx| {
                    assert!(actual.as_mut().poll(cx).is_pending());
                    std::task::Poll::Ready(())
                })
                .await;
                entered.send(()).unwrap();
                release.await.unwrap();
                actual.await
            } else {
                let response = actual.await?;
                entered.send(()).unwrap();
                release.await.unwrap();
                Ok(response)
            }
        }
        .boxed()
    }
}

#[test]
fn module_link_rpc_actual_read_eof_error_and_drop_retire_before_held_registered_handler_drain() {
    for mode in 0..3 {
        let (_dir, root, uri) = files();
        let (service, socket) = initialize_only(Some(&Url::from_directory_path(&root).unwrap()));
        let state = Arc::clone(&service.inner().state);
        state.documents.open(
            uri.clone(),
            "import './child.ts';".into(),
            17,
            "typescript".into(),
        );
        let resume = service
            .inner()
            .navigation
            .as_ref()
            .unwrap()
            .pause_module_link_worker(&uri);
        let (native_enter, native_entered) = oneshot::channel();
        let (native_release, native_released) = oneshot::channel();
        let (legacy_enter, legacy_entered) = oneshot::channel();
        let (legacy_release, legacy_released) = oneshot::channel();
        let bytes = [
            frame(&links(2, &uri)),
            frame(&json!({"jsonrpc":"2.0","id":3,
            "method":"textDocument/documentLink","params":{"textDocument":{"uri":uri}}})),
        ]
        .concat();
        let input = Arc::new(Mutex::new(InputState {
            bytes: bytes.into(),
            ended: None,
            waker: None,
        }));
        let output = Arc::new(Mutex::new(Vec::new()));
        let outer = service.inner().module_link_transport_lease().unwrap();
        let read = ModuleLinkInput::new(
            Input(Arc::clone(&input)),
            service.inner().module_link_transport_lease(),
        );
        let held = HeldService {
            inner: service,
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
            Box::pin(Server::new(read, Output(Arc::clone(&output)), socket).serve(held));
        block_on(async {
            match futures::future::select(
                transport.as_mut(),
                Box::pin(futures::future::join(native_entered, legacy_entered)),
            )
            .await
            {
                futures::future::Either::Right(((a, b), _)) => {
                    a.unwrap();
                    b.unwrap();
                }
                _ => panic!("both actual registered handler futures must remain held"),
            }
        });
        assert!(state.capture_module_link_context().is_ok());
        assert!(output.lock().is_empty());
        if mode == 2 {
            drop(transport);
            assert!(matches!(
                state.capture_module_link_context(),
                Err(ModuleLinkContextError::Retired(_))
            ));
            assert!(native_release.send(()).is_err());
            assert!(legacy_release.send(()).is_err());
            let _ = resume.send(());
            assert!(output.lock().is_empty());
            drop(outer);
            continue;
        }
        let wake = {
            let mut read = input.lock();
            read.ended = Some(mode == 1);
            read.waker.take()
        };
        if let Some(wake) = wake {
            wake.wake();
        }
        assert!(
            transport
                .as_mut()
                .poll(&mut Context::from_waker(noop_waker_ref()))
                .is_pending()
        );
        let reason = if mode == 1 {
            ModuleLinkRetirement::InputError
        } else {
            ModuleLinkRetirement::InputEof
        };
        assert_eq!(
            state.capture_module_link_context().err(),
            Some(ModuleLinkContextError::Retired(reason))
        );
        assert!(
            frames(&output.lock())
                .iter()
                .all(|value| value["id"].is_null())
        );
        resume.send(()).unwrap();
        native_release.send(()).unwrap();
        legacy_release.send(()).unwrap();
        block_on(transport);
        let mut expected = vec![
            json!({"jsonrpc":"2.0","id":2,"error":{"code":-32801,"message":"Native module-link context superseded"}}),
            json!({"jsonrpc":"2.0","id":3,"result":null}),
        ];
        if mode == 1 {
            expected.insert(
                0,
                json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}),
            );
        }
        assert_eq!(frames(&output.lock()), expected);
        assert_eq!(
            state.documents.text(&uri).as_deref(),
            Some("import './child.ts';")
        );
        drop(outer);
        assert_eq!(
            state.capture_module_link_context().err(),
            Some(ModuleLinkContextError::Retired(reason))
        );
    }
}

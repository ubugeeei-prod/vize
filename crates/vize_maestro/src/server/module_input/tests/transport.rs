//! Actual tower-lsp drain holds a genuine source result and original legacy RPC.
use super::{Arc, Context, ModuleLinkInput, ModuleLinkRetirement, Poll, retired};
use crate::{
    runtime::block_on,
    server::{MaestroServer, build_lsp_service},
    source_project::SourceQueryProject,
};
use futures::{
    FutureExt,
    channel::oneshot,
    future::BoxFuture,
    io::{AsyncRead, AsyncWrite},
    task::noop_waker_ref,
};
use parking_lot::Mutex;
use serde_json::{Value, json};
use std::{collections::VecDeque, io, pin::Pin, task::Waker};
use tower::Service;
use tower_lsp::{
    LspService, Server,
    jsonrpc::{Request, Response},
};

const URI: &str = "file:///actual-transport.ts";
struct InputState {
    bytes: VecDeque<u8>,
    ended: Option<bool>,
    waker: Option<Waker>,
}
struct Input(Arc<Mutex<InputState>>);
impl AsyncRead for Input {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }
        let mut state = self.0.lock();
        if !state.bytes.is_empty() {
            let count = state.bytes.len().min(buf.len());
            for byte in &mut buf[..count] {
                *byte = state.bytes.pop_front().unwrap();
            }
            return Poll::Ready(Ok(count));
        }
        match state.ended {
            Some(false) => Poll::Ready(Ok(0)),
            Some(true) => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::ConnectionReset,
                "actual input termination",
            ))),
            None => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}
struct Output(Arc<Mutex<Vec<u8>>>);
impl AsyncWrite for Output {
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.0.lock().extend_from_slice(bytes);
        Poll::Ready(Ok(bytes.len()))
    }
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
struct Hold {
    entered: oneshot::Sender<()>,
    release: oneshot::Receiver<()>,
}
struct HeldService {
    inner: LspService<MaestroServer>,
    project: Arc<SourceQueryProject<'static>>,
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
        if request.method() == "test/heldModuleContext" {
            let id = request.id().unwrap().clone();
            let context = self.project.capture_module_link_context().unwrap();
            let (query, _) = self
                .project
                .begin_query(&tower_lsp::lsp_types::Url::parse(URI).unwrap())
                .unwrap();
            let Hold { entered, release } = self.native.take().unwrap();
            async move {
                let ready = query.run(|snapshot| async move { json!({"version":snapshot.version(),"source":snapshot.source(),"language":snapshot.language_id()}) }).await.unwrap();
                entered.send(()).unwrap();
                release.await.unwrap();
                let result = ready.publish_with_module_link_context(&context, |value| value);
                let value = match result {
                    Err(crate::source_project::ModuleLinkPublicationError::Context(crate::server::ModuleLinkContextError::Retired(ModuleLinkRetirement::InputEof))) => json!({"refused":"input-eof"}),
                    Err(crate::source_project::ModuleLinkPublicationError::Context(crate::server::ModuleLinkContextError::Retired(ModuleLinkRetirement::InputError))) => json!({"refused":"input-error"}),
                    Ok(value) => value,
                    _ => panic!("unexpected genuine source/context refusal"),
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

fn frame(value: &Value) -> Vec<u8> {
    let body = serde_json::to_vec(value).unwrap();
    let header = vize_l0::cstr!("Content-Length: {}\r\n\r\n", body.len());
    [header.as_bytes(), body.as_slice()].concat()
}
fn frames(bytes: &[u8]) -> Vec<Value> {
    let mut input = bytes;
    let mut values: Vec<Value> = Vec::new();
    while !input.is_empty() {
        let end = input
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .unwrap();
        let header = std::str::from_utf8(&input[..end]).unwrap();
        let count: usize = header
            .strip_prefix("Content-Length: ")
            .unwrap()
            .parse()
            .unwrap();
        values.push(serde_json::from_slice(&input[end + 4..end + 4 + count]).unwrap());
        input = &input[end + 4 + count..];
    }
    values.sort_by_key(|value| value["id"].as_i64().unwrap_or(0));
    values
}

#[test]
fn module_link_input_eof_or_error_retires_held_original_query_before_real_legacy_drain() {
    // EOF, terminal error, future cancellation and codec error without read EOF.
    for mode in 0..4 {
        let error = mode == 1;
        let (mut service, socket) = build_lsp_service();
        let state = Arc::clone(&service.inner().state);
        super::lifecycle::initialize_disabled(&mut service);
        state.apply_lsp_initialization_options(Some(&json!({"documentLinks":true})));
        state.set_workspace_root("/actual-transport-root".into());
        let project = Arc::new(SourceQueryProject::new_server(Arc::clone(&state)));
        project.open(
            tower_lsp::lsp_types::Url::parse(URI).unwrap(),
            "const value=1;value;".into(),
            7,
            "javascript".into(),
        );
        let (native_enter, native_entered) = oneshot::channel();
        let (native_release, native_released) = oneshot::channel();
        let (legacy_enter, legacy_entered) = oneshot::channel();
        let (legacy_release, legacy_released) = oneshot::channel();
        let bytes = [
            frame(&json!({"jsonrpc":"2.0","id":2,"method":"test/heldModuleContext"})),
            frame(&json!({"jsonrpc":"2.0","id":3,"method":"textDocument/documentLink","params":{"textDocument":{"uri":URI}}})),
        ].concat();
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
            project: Arc::clone(&project),
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
            let entered = futures::future::join(native_entered, legacy_entered);
            match futures::future::select(transport.as_mut(), Box::pin(entered)).await {
                futures::future::Either::Right(((a, b), _)) => {
                    a.unwrap();
                    b.unwrap();
                }
                _ => panic!("transport must still hold both real handler futures"),
            }
        });
        assert!(state.capture_module_link_context().is_ok());
        if mode == 2 {
            drop(transport);
            assert!(matches!(
                state.capture_module_link_context(),
                Err(crate::server::ModuleLinkContextError::Retired(_))
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
                reader.ended = Some(error);
            }
            reader.waker.take()
        };
        if let Some(wake) = wake {
            wake.wake();
        }
        let mut cx = Context::from_waker(noop_waker_ref());
        assert!(std::future::Future::poll(transport.as_mut(), &mut cx).is_pending());
        if mode == 3 {
            assert!(state.capture_module_link_context().is_ok());
        } else {
            retired(
                &state,
                if error {
                    ModuleLinkRetirement::InputError
                } else {
                    ModuleLinkRetirement::InputEof
                },
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
            json!({"jsonrpc":"2.0","id":2,"result":if mode == 3 {json!({"version":7,"source":"const value=1;value;","language":"javascript"})} else {json!({"refused":if error {"input-error"} else {"input-eof"}})}}),
            json!({"jsonrpc":"2.0","id":3,"result":null}),
        ];
        if error || mode == 3 {
            expected.insert(
                0,
                json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}),
            );
        }
        assert_eq!(frames(&output.lock()), expected);
        assert_eq!(
            state
                .documents
                .text(&tower_lsp::lsp_types::Url::parse(URI).unwrap())
                .as_deref(),
            Some("const value=1;value;")
        );
        drop(outer);
        retired(
            &state,
            if mode == 3 {
                ModuleLinkRetirement::TransportEnded
            } else if error {
                ModuleLinkRetirement::InputError
            } else {
                ModuleLinkRetirement::InputEof
            },
        );
    }
}

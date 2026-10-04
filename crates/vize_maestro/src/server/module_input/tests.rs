#![expect(
    clippy::disallowed_types,
    reason = "actual server sessions and IO controls retain shared owner/reader state"
)]
use super::{ModuleLinkInput, ModuleLinkRetirement, ModuleLinkTerminationLease};
use crate::server::{ModuleLinkContextError, ServerState};
use futures::{io::AsyncRead, task::noop_waker_ref};
use std::{
    collections::VecDeque,
    io::{self, IoSliceMut},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
mod lifecycle;
#[cfg(feature = "native")]
mod transport;

fn observed<I>(inner: I) -> (Arc<ServerState>, ModuleLinkInput<I>) {
    let state = Arc::new(ServerState::new());
    let lease = ModuleLinkTerminationLease::new(&state, ModuleLinkRetirement::TransportEnded);
    (state, ModuleLinkInput::new(inner, Some(lease)))
}
fn live(state: &ServerState) {
    #[cfg(feature = "native")]
    assert!(matches!(
        state.capture_module_link_context(),
        Err(ModuleLinkContextError::MissingRoot)
    ));
    #[cfg(not(feature = "native"))]
    assert!(matches!(
        state.capture_module_link_context(),
        Err(ModuleLinkContextError::NativeUnavailable)
    ));
}
fn retired(state: &ServerState, reason: ModuleLinkRetirement) {
    assert!(
        matches!(state.capture_module_link_context(), Err(ModuleLinkContextError::Retired(actual)) if actual == reason)
    );
}

enum Step {
    Pending,
    Empty,
    Bytes(Vec<u8>),
    Error(io::ErrorKind),
}
struct Reader(VecDeque<Step>);
impl AsyncRead for Reader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }
        match self.0.pop_front().unwrap() {
            Step::Pending => Poll::Pending,
            Step::Empty => Poll::Ready(Ok(0)),
            Step::Error(kind) => Poll::Ready(Err(io::Error::new(kind, "original read failure"))),
            Step::Bytes(bytes) => {
                buf[..bytes.len()].copy_from_slice(&bytes);
                Poll::Ready(Ok(bytes.len()))
            }
        }
    }
}

#[test]
fn module_link_input_preserves_pending_zero_capacity_bytes_and_exact_eof_result() {
    let (state, mut input) = observed(Reader(VecDeque::from([
        Step::Pending,
        Step::Bytes(vec![1, 2, 3]),
        Step::Empty,
    ])));
    let mut cx = Context::from_waker(noop_waker_ref());
    assert!(matches!(
        Pin::new(&mut input).poll_read(&mut cx, &mut []),
        Poll::Ready(Ok(0))
    ));
    live(&state);
    let mut buf = [9; 8];
    assert!(
        Pin::new(&mut input)
            .poll_read(&mut cx, &mut buf)
            .is_pending()
    );
    live(&state);
    assert!(matches!(
        Pin::new(&mut input).poll_read(&mut cx, &mut buf),
        Poll::Ready(Ok(3))
    ));
    assert_eq!(buf, [1, 2, 3, 9, 9, 9, 9, 9]);
    live(&state);
    assert!(matches!(
        Pin::new(&mut input).poll_read(&mut cx, &mut buf),
        Poll::Ready(Ok(0))
    ));
    retired(&state, ModuleLinkRetirement::InputEof);
    drop(input);
    retired(&state, ModuleLinkRetirement::InputEof);
}

#[test]
fn module_link_input_preserves_original_terminal_error_and_nonterminal_reader_controls() {
    for kind in [
        io::ErrorKind::BrokenPipe,
        io::ErrorKind::ConnectionReset,
        io::ErrorKind::UnexpectedEof,
        io::ErrorKind::Interrupted,
        io::ErrorKind::WouldBlock,
    ] {
        let (state, mut input) = observed(Reader(VecDeque::from([Step::Error(kind)])));
        let mut cx = Context::from_waker(noop_waker_ref());
        let result = Pin::new(&mut input).poll_read(&mut cx, &mut [0; 8]);
        let Poll::Ready(Err(error)) = result else {
            panic!("original error lost");
        };
        assert_eq!(error.kind(), kind);
        assert_eq!(error.to_string(), "original read failure");
        if matches!(kind, io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock) {
            live(&state);
        } else {
            retired(&state, ModuleLinkRetirement::InputError);
        }
    }
}

struct VectoredReader {
    calls: usize,
}
impl AsyncRead for VectoredReader {
    fn poll_read(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        _: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        panic!("vectored semantics must delegate")
    }
    fn poll_read_vectored(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        if bufs.iter().all(|buf| buf.is_empty()) {
            return Poll::Ready(Ok(0));
        }
        self.calls += 1;
        if self.calls > 1 {
            return Poll::Ready(Ok(0));
        }
        bufs[1].copy_from_slice(&[1, 2]);
        bufs[2].copy_from_slice(&[3, 4]);
        Poll::Ready(Ok(4))
    }
}
#[test]
fn module_link_input_delegates_complete_vectored_reads_and_only_nonempty_eof_retires() {
    let (state, mut input) = observed(VectoredReader { calls: 0 });
    let mut cx = Context::from_waker(noop_waker_ref());
    assert!(matches!(
        Pin::new(&mut input).poll_read_vectored(&mut cx, &mut []),
        Poll::Ready(Ok(0))
    ));
    live(&state);
    let mut empty = [];
    let mut a = [0; 2];
    let mut b = [0; 2];
    let mut bufs = [
        IoSliceMut::new(&mut empty),
        IoSliceMut::new(&mut a),
        IoSliceMut::new(&mut b),
    ];
    assert!(matches!(
        Pin::new(&mut input).poll_read_vectored(&mut cx, &mut bufs),
        Poll::Ready(Ok(4))
    ));
    live(&state);
    assert!(matches!(
        Pin::new(&mut input).poll_read_vectored(&mut cx, &mut bufs),
        Poll::Ready(Ok(0))
    ));
    assert_eq!(a, [1, 2]);
    assert_eq!(b, [3, 4]);
    retired(&state, ModuleLinkRetirement::InputEof);
}

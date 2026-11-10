//! Terminate the outer LSP server as soon as its client input disappears.

use futures::{channel::oneshot, io::AsyncRead};
use std::{
    future::Future,
    io::{self, IoSliceMut},
    pin::Pin,
    task::{Context, Poll},
};

/// Reports terminal input independently from the protocol transport.
///
/// `tower-lsp` drains every in-flight request after its reader reaches EOF.
/// That is normally useful, but a request blocked on an external language
/// backend can then keep a stdio server alive after its client has gone. The
/// outer server races this signal against that drain and drops the transport,
/// cancelling those request futures instead.
pub(crate) struct InputTermination<I> {
    inner: I,
    ended: Option<oneshot::Sender<()>>,
}

pub(crate) fn observe<I>(inner: I) -> (InputTermination<I>, impl Future<Output = ()> + Send) {
    let (ended, observed) = oneshot::channel();
    (
        InputTermination {
            inner,
            ended: Some(ended),
        },
        async move {
            let _ = observed.await;
        },
    )
}

impl<I> InputTermination<I> {
    fn observe(&mut self, nonempty: bool, result: &Poll<io::Result<usize>>) {
        let terminal = match result {
            Poll::Ready(Ok(0)) => nonempty,
            // AsyncRead implementations must handle these transient results
            // themselves. Do not turn a nonconforming adapter into EOF.
            Poll::Ready(Err(error)) => !matches!(
                error.kind(),
                io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
            ),
            _ => false,
        };
        if terminal && let Some(ended) = self.ended.take() {
            let _ = ended.send(());
        }
    }
}

impl<I: AsyncRead + Unpin> AsyncRead for InputTermination<I> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        out: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let result = Pin::new(&mut this.inner).poll_read(cx, out);
        this.observe(!out.is_empty(), &result);
        result
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let nonempty = bufs.iter().any(|buf| !buf.is_empty());
        let result = Pin::new(&mut this.inner).poll_read_vectored(cx, bufs);
        this.observe(nonempty, &result);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::observe;
    use futures::{FutureExt, io::AsyncRead, task::noop_waker_ref};
    use std::{
        collections::VecDeque,
        io,
        pin::Pin,
        task::{Context, Poll},
    };

    enum Step {
        Pending,
        Bytes(Vec<u8>),
        Empty,
        Error(io::ErrorKind),
    }

    struct Reader(VecDeque<Step>);

    impl AsyncRead for Reader {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _: &mut Context<'_>,
            out: &mut [u8],
        ) -> Poll<io::Result<usize>> {
            if out.is_empty() {
                return Poll::Ready(Ok(0));
            }
            match self.0.pop_front().unwrap() {
                Step::Pending => Poll::Pending,
                Step::Bytes(bytes) => {
                    out[..bytes.len()].copy_from_slice(&bytes);
                    Poll::Ready(Ok(bytes.len()))
                }
                Step::Empty => Poll::Ready(Ok(0)),
                Step::Error(kind) => Poll::Ready(Err(io::Error::new(kind, "input failure"))),
            }
        }
    }

    #[test]
    fn preserves_reads_and_only_signals_terminal_results() {
        let (mut input, ended) = observe(Reader(VecDeque::from([
            Step::Pending,
            Step::Bytes(vec![1, 2, 3]),
            Step::Error(io::ErrorKind::Interrupted),
            Step::Empty,
        ])));
        let mut ended = Box::pin(ended);
        let mut cx = Context::from_waker(noop_waker_ref());

        assert!(matches!(
            Pin::new(&mut input).poll_read(&mut cx, &mut []),
            Poll::Ready(Ok(0))
        ));
        assert!(ended.as_mut().now_or_never().is_none());

        let mut out = [9; 8];
        assert!(
            Pin::new(&mut input)
                .poll_read(&mut cx, &mut out)
                .is_pending()
        );
        assert!(ended.as_mut().now_or_never().is_none());
        assert!(matches!(
            Pin::new(&mut input).poll_read(&mut cx, &mut out),
            Poll::Ready(Ok(3))
        ));
        assert_eq!(out, [1, 2, 3, 9, 9, 9, 9, 9]);
        assert!(ended.as_mut().now_or_never().is_none());

        let Poll::Ready(Err(error)) = Pin::new(&mut input).poll_read(&mut cx, &mut out) else {
            panic!("transient input error was not preserved");
        };
        assert_eq!(error.kind(), io::ErrorKind::Interrupted);
        assert!(ended.as_mut().now_or_never().is_none());

        assert!(matches!(
            Pin::new(&mut input).poll_read(&mut cx, &mut out),
            Poll::Ready(Ok(0))
        ));
        assert_eq!(ended.as_mut().now_or_never(), Some(()));
    }

    #[test]
    fn signals_terminal_errors() {
        let (mut input, ended) = observe(Reader(VecDeque::from([Step::Error(
            io::ErrorKind::ConnectionReset,
        )])));
        let mut ended = Box::pin(ended);
        let mut cx = Context::from_waker(noop_waker_ref());
        let Poll::Ready(Err(error)) = Pin::new(&mut input).poll_read(&mut cx, &mut [0; 8]) else {
            panic!("terminal input error was not preserved");
        };
        assert_eq!(error.kind(), io::ErrorKind::ConnectionReset);
        assert_eq!(ended.as_mut().now_or_never(), Some(()));
    }
}

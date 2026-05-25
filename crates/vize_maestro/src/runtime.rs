//! Minimal runtime utilities for the LSP server.
//!
//! This module intentionally stays tiny: a single-thread `block_on`, a simple
//! timeout helper, and thread-backed adapters that let blocking stdio/TCP
//! handles satisfy `futures::io` traits without depending on Tokio.
#![allow(clippy::disallowed_types)]

use std::future::Future;
use std::io::{self, Read, Write};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::mpsc as std_mpsc;
use std::task::{Context, Poll};
use std::thread;
use std::time::Duration;

use futures::channel::{mpsc, oneshot};
use futures::io::{AsyncRead, AsyncWrite};
use futures::stream::StreamExt;
use futures::task::{ArcWake, waker};

/// Runs a future to completion on the current thread.
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    struct ThreadWaker {
        thread: thread::Thread,
    }

    impl ArcWake for ThreadWaker {
        fn wake_by_ref(arc_self: &Arc<Self>) {
            arc_self.thread.unpark();
        }
    }

    let waker = waker(Arc::new(ThreadWaker {
        thread: thread::current(),
    }));
    let mut context = Context::from_waker(&waker);
    let mut future = Box::pin(future);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => thread::park(),
        }
    }
}

/// Error returned when a timeout expires before a future completes.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TimeoutElapsed;

/// Resolves with `future`'s output, or `TimeoutElapsed` after `duration`.
pub async fn timeout<F>(duration: Duration, future: F) -> Result<F::Output, TimeoutElapsed>
where
    F: Future,
{
    let (tx, mut rx) = oneshot::channel();
    let _ = thread::Builder::new()
        .name("vize-timeout".to_string())
        .spawn(move || {
            thread::sleep(duration);
            let _ = tx.send(());
        });

    futures::pin_mut!(future);

    futures::future::poll_fn(|cx| {
        if let Poll::Ready(output) = future.as_mut().poll(cx) {
            return Poll::Ready(Ok(output));
        }

        match Pin::new(&mut rx).poll(cx) {
            Poll::Ready(_) => Poll::Ready(Err(TimeoutElapsed)),
            Poll::Pending => Poll::Pending,
        }
    })
    .await
}

enum ReadChunk {
    Data(Vec<u8>),
    Error(String),
}

/// AsyncRead adapter backed by a blocking reader thread.
pub struct ThreadedReader {
    rx: mpsc::UnboundedReceiver<ReadChunk>,
    pending: Vec<u8>,
    offset: usize,
}

impl ThreadedReader {
    fn new(rx: mpsc::UnboundedReceiver<ReadChunk>) -> Self {
        Self {
            rx,
            pending: Vec::new(),
            offset: 0,
        }
    }
}

/// Wraps a blocking reader as a `futures::io::AsyncRead`.
pub fn threaded_reader<R>(name: &str, mut reader: R) -> io::Result<ThreadedReader>
where
    R: Read + Send + 'static,
{
    let (tx, rx) = mpsc::unbounded();
    thread::Builder::new()
        .name(name.to_string())
        .spawn(move || {
            let mut buffer = [0; 8192];

            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(len) => {
                        if tx
                            .unbounded_send(ReadChunk::Data(buffer[..len].to_vec()))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                    Err(error) => {
                        let _ = tx.unbounded_send(ReadChunk::Error(error.to_string()));
                        break;
                    }
                }
            }
        })
        .map_err(|error| io::Error::other(format!("failed to spawn reader thread: {error}")))?;

    Ok(ThreadedReader::new(rx))
}

impl AsyncRead for ThreadedReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        out: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if out.is_empty() {
            return Poll::Ready(Ok(0));
        }

        loop {
            if self.offset < self.pending.len() {
                let len = (self.pending.len() - self.offset).min(out.len());
                out[..len].copy_from_slice(&self.pending[self.offset..self.offset + len]);
                self.offset += len;

                if self.offset == self.pending.len() {
                    self.pending.clear();
                    self.offset = 0;
                }

                return Poll::Ready(Ok(len));
            }

            match self.rx.poll_next_unpin(cx) {
                Poll::Ready(Some(ReadChunk::Data(data))) => {
                    if !data.is_empty() {
                        self.pending = data;
                        self.offset = 0;
                    }
                }
                Poll::Ready(Some(ReadChunk::Error(error))) => {
                    return Poll::Ready(Err(io::Error::other(error)));
                }
                Poll::Ready(None) => return Poll::Ready(Ok(0)),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

enum WriteCommand {
    Write(Vec<u8>),
    Flush(oneshot::Sender<Result<(), String>>),
}

/// AsyncWrite adapter backed by a blocking writer thread.
pub struct ThreadedWriter {
    tx: std_mpsc::Sender<WriteCommand>,
    pending_flush: Option<oneshot::Receiver<Result<(), String>>>,
}

/// Wraps a blocking writer as a `futures::io::AsyncWrite`.
pub fn threaded_writer<W>(name: &str, mut writer: W) -> io::Result<ThreadedWriter>
where
    W: Write + Send + 'static,
{
    let (tx, rx) = std_mpsc::channel();
    thread::Builder::new()
        .name(name.to_string())
        .spawn(move || {
            let mut failure: Option<String> = None;

            while let Ok(command) = rx.recv() {
                match command {
                    WriteCommand::Write(bytes) => {
                        if failure.is_none()
                            && let Err(error) = writer.write_all(&bytes)
                        {
                            failure = Some(error.to_string());
                        }
                    }
                    WriteCommand::Flush(reply) => {
                        let result = if let Some(error) = failure.clone() {
                            Err(error)
                        } else {
                            writer.flush().map_err(|error| {
                                let message = error.to_string();
                                failure = Some(message.clone());
                                message
                            })
                        };
                        let _ = reply.send(result);
                    }
                }
            }

            let _ = writer.flush();
        })
        .map_err(|error| io::Error::other(format!("failed to spawn writer thread: {error}")))?;

    Ok(ThreadedWriter {
        tx,
        pending_flush: None,
    })
}

impl AsyncWrite for ThreadedWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        let len = buf.len();
        let result = self.tx.send(WriteCommand::Write(buf.to_vec()));
        Poll::Ready(
            result
                .map(|()| len)
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "writer thread stopped")),
        )
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        if self.pending_flush.is_none() {
            let (tx, rx) = oneshot::channel();
            self.tx
                .send(WriteCommand::Flush(tx))
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "writer thread stopped"))?;
            self.pending_flush = Some(rx);
        }

        let receiver = self
            .pending_flush
            .as_mut()
            .expect("flush receiver must exist");

        match Pin::new(receiver).poll(cx) {
            Poll::Ready(Ok(Ok(()))) => {
                self.pending_flush = None;
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Ok(Err(error))) => {
                self.pending_flush = None;
                Poll::Ready(Err(io::Error::other(error)))
            }
            Poll::Ready(Err(_)) => {
                self.pending_flush = None;
                Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "writer thread stopped",
                )))
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.poll_flush(cx)
    }
}

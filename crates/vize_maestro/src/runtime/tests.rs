use super::*;
use std::sync::mpsc as std_mpsc;

use futures::task::noop_waker;

struct BlockingFlushWriter {
    writes: std_mpsc::Sender<Vec<u8>>,
    flush_started: std_mpsc::Sender<()>,
    flush_continue: std_mpsc::Receiver<()>,
}

impl Write for BlockingFlushWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writes
            .send(buf.to_vec())
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "write log closed"))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_started
            .send(())
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "flush log closed"))?;
        self.flush_continue
            .recv()
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "flush gate closed"))?;
        Ok(())
    }
}

#[test]
fn threaded_writer_waits_for_pending_flush_before_accepting_more_writes() {
    let (writes_tx, writes_rx) = std_mpsc::channel();
    let (flush_started_tx, flush_started_rx) = std_mpsc::channel();
    let (flush_continue_tx, flush_continue_rx) = std_mpsc::channel();
    let blocking_writer = BlockingFlushWriter {
        writes: writes_tx,
        flush_started: flush_started_tx,
        flush_continue: flush_continue_rx,
    };
    let mut writer = threaded_writer("vize-test-writer", blocking_writer).unwrap();
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);

    assert!(matches!(
        Pin::new(&mut writer).poll_write(&mut cx, b"first"),
        Poll::Ready(Ok(5))
    ));
    assert!(matches!(
        Pin::new(&mut writer).poll_flush(&mut cx),
        Poll::Pending
    ));
    assert_eq!(
        writes_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
        b"first"
    );
    flush_started_rx
        .recv_timeout(Duration::from_secs(1))
        .unwrap();

    assert!(matches!(
        Pin::new(&mut writer).poll_write(&mut cx, b"second"),
        Poll::Pending
    ));
    assert!(writes_rx.try_recv().is_err());

    flush_continue_tx.send(()).unwrap();
    futures::executor::block_on(futures::future::poll_fn(|cx| {
        Pin::new(&mut writer).poll_flush(cx)
    }))
    .unwrap();

    assert!(matches!(
        Pin::new(&mut writer).poll_write(&mut cx, b"second"),
        Poll::Ready(Ok(6))
    ));
    assert_eq!(
        writes_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
        b"second"
    );

    drop(writer);
    flush_started_rx
        .recv_timeout(Duration::from_secs(1))
        .unwrap();
    flush_continue_tx.send(()).unwrap();
}

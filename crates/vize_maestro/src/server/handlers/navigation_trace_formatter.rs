//! Actual locked Full formatter and request/worker context fixture. No LSP/native calls.
#![expect(
    clippy::disallowed_methods,
    reason = "bounded test capture uses std buffers"
)]

use std::future::Future;
use std::io::{self, Write};
use std::pin::Pin;
use std::sync::{Arc, Mutex, mpsc};
use std::task::{Context, Poll};

use futures::{channel::oneshot, task::noop_waker_ref};
use tracing_subscriber::fmt::writer::MakeWriterExt;
use vize_canon::corsa_bridge::EditorPreparationPhase as Phase;

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut output = self.0.lock().unwrap();
        if output.len() + bytes.len() > 1024 * 1024 {
            return Err(io::Error::other("formatter fixture byte cap"));
        }
        output.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn poll<F: Future>(future: Pin<&mut F>) -> Poll<F::Output> {
    future.poll(&mut Context::from_waker(noop_waker_ref()))
}

#[test]
#[ignore = "diagnostic-only: requires fresh opt-in process and owned formatter output"]
fn navigation_trace_formatter_preserves_request_worker_ancestry() {
    assert_eq!(
        std::env::var("VIZE_TRACE_EDITOR_PREPARATION").as_deref(),
        Ok("1")
    );
    let file = Capture::default();
    let stderr = Capture::default();
    let left = file.clone();
    let right = stderr.clone();
    // Exact production Full formatter options and two destinations; no JSON/custom formatter.
    let subscriber = tracing_subscriber::fmt()
        .with_writer((move || left.clone()).and(move || right.clone()))
        .with_ansi(false)
        .log_internal_errors(false)
        .finish();
    let dispatch = tracing::Dispatch::new(subscriber);
    tracing::dispatcher::with_default(&dispatch, || {
        let never_polled = Phase::request("rename", async {
            panic!("unpolled request body executed");
        });
        drop(never_polled);

        futures::executor::block_on(Phase::request("references", async {
            let child =
                tracing::info_span!("fixture_child", request_id = 999_u64, method = "unrelated");
            child.in_scope(|| {
                let first = Phase::navigation("fixture.nested", 1);
                first.finish();
                let repeated = Phase::navigation("fixture.nested", 1);
                repeated.finish();
            });
        }));

        let (a_send, a_receive) = oneshot::channel::<()>();
        let (b_send, b_receive) = oneshot::channel::<()>();
        let mut a = Box::pin(Phase::request("references", async {
            let phase = Phase::navigation("fixture.concurrent_a", 1);
            a_receive.await.unwrap();
            phase.finish();
        }));
        let mut b = Box::pin(Phase::request("references", async {
            let phase = Phase::navigation("fixture.concurrent_b", 1);
            b_receive.await.unwrap();
            phase.finish();
        }));
        assert!(poll(a.as_mut()).is_pending());
        assert!(poll(b.as_mut()).is_pending());
        b_send.send(()).unwrap();
        assert!(poll(b.as_mut()).is_ready());
        a_send.send(()).unwrap();
        assert!(poll(a.as_mut()).is_ready());

        let returned_error = futures::executor::block_on(Phase::request("prepare_rename", async {
            let phase = Phase::navigation("fixture.returned_error", 1);
            phase.finish();
            Err::<(), &str>("ordinary returned error")
        }));
        assert_eq!(returned_error, Err("ordinary returned error"));

        let (_cancel_send, cancel_receive) = oneshot::channel::<()>();
        let mut cancelled = Box::pin(Phase::request("rename", async {
            let _unfinished = Phase::navigation("fixture.pending_drop", 1);
            let _ = cancel_receive.await;
        }));
        assert!(poll(cancelled.as_mut()).is_pending());
        drop(cancelled);

        let (entered_send, entered_receive) = mpsc::channel();
        let (release_send, release_receive) = mpsc::channel();
        let (worker_send, worker_receive) = mpsc::channel();
        let (_pending_send, pending_receive) = oneshot::channel::<()>();
        let worker_dispatch = dispatch.clone();
        let mut late = Box::pin(Phase::request("hover", async {
            let context = Phase::context();
            let worker = std::thread::spawn(move || {
                tracing::dispatcher::with_default(&worker_dispatch, || {
                    context.in_scope(|| {
                        let phase = Phase::navigation("fixture.late_worker", 1);
                        entered_send.send(()).unwrap();
                        release_receive.recv().unwrap();
                        phase.finish();
                    });
                });
            });
            worker_send.send(worker).unwrap();
            let _ = pending_receive.await;
        }));
        assert!(poll(late.as_mut()).is_pending());
        entered_receive.recv().unwrap();
        drop(late);
        futures::executor::block_on(Phase::request("rename", async {
            let phase = Phase::navigation("fixture.after_cancel", 1);
            phase.finish();
        }));
        release_send.send(()).unwrap();
        worker_receive.recv().unwrap().join().unwrap();

        let (send, receive) = oneshot::channel::<()>();
        let mut migrating = Box::pin(Phase::request("prepare_rename", async {
            let phase = Phase::navigation("fixture.migrating", 1);
            receive.await.unwrap();
            phase.finish();
        }));
        assert!(poll(migrating.as_mut()).is_pending());
        let thread_dispatch = dispatch.clone();
        migrating = std::thread::spawn(move || {
            tracing::dispatcher::with_default(&thread_dispatch, || {
                assert!(poll(migrating.as_mut()).is_pending());
                migrating
            })
        })
        .join()
        .unwrap();
        send.send(()).unwrap();
        let thread_dispatch = dispatch.clone();
        std::thread::spawn(move || {
            tracing::dispatcher::with_default(&thread_dispatch, || {
                assert!(poll(migrating.as_mut()).is_ready());
            });
        })
        .join()
        .unwrap();

        let background = Phase::navigation("fixture.background_unbound", 1);
        background.finish();
    });
    let actual = file.0.lock().unwrap().clone();
    assert_eq!(
        actual,
        *stderr.0.lock().unwrap(),
        "both actual formatter destinations must match"
    );
    let text = std::str::from_utf8(&actual).unwrap();
    let lines: Vec<_> = text.lines().collect();
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.contains("event=\"request\""))
            .count(),
        8
    );
    for label in [
        "fixture.nested",
        "fixture.concurrent_a",
        "fixture.concurrent_b",
        "fixture.returned_error",
        "fixture.pending_drop",
        "fixture.late_worker",
        "fixture.after_cancel",
        "fixture.migrating",
    ] {
        let matching: Vec<_> = lines
            .iter()
            .filter(|line| line.contains(&format!("phase=\"{label}\"")))
            .collect();
        assert_eq!(
            matching.len(),
            if label == "fixture.nested" { 4 } else { 2 },
            "{label}"
        );
        assert!(
            matching
                .iter()
                .all(|line| line.contains("navigation_request{request_id=")),
            "{label}"
        );
        assert!(
            matching.iter().all(|line| line.contains("context_span=")),
            "{label}"
        );
    }
    let nested = lines
        .iter()
        .find(|line| line.contains("phase=\"fixture.nested\""))
        .unwrap();
    assert!(nested.contains("fixture_child{request_id=999 method=\"unrelated\"}"));
    let cancelled_end = lines
        .iter()
        .find(|line| line.contains("fixture.pending_drop") && line.contains("event=\"end\""))
        .unwrap();
    assert!(cancelled_end.contains("completed=false"));
    let error_end = lines
        .iter()
        .find(|line| line.contains("fixture.returned_error") && line.contains("event=\"end\""))
        .unwrap();
    assert!(error_end.contains("completed=true"));
    let late_end = lines
        .iter()
        .position(|line| line.contains("fixture.late_worker") && line.contains("event=\"end\""))
        .unwrap();
    assert!(
        lines
            .iter()
            .filter(|line| line.contains("fixture.late_worker"))
            .all(|line| line.contains("method=\"hover\"")),
        "late worker must retain the actual hover request owner"
    );
    let next_request_end = lines
        .iter()
        .position(|line| line.contains("fixture.after_cancel") && line.contains("event=\"end\""))
        .unwrap();
    assert!(
        late_end > next_request_end,
        "entered worker must finish after the later request"
    );
    let output =
        std::env::var_os("VIZE_NAV_FORMATTER_OUTPUT").expect("fresh fixture output path required");
    use std::fs::OpenOptions;
    let mut destination = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .unwrap();
    destination.write_all(&actual).unwrap();
    println!(
        "actual Full formatter: 8 requests, nested/repeated/concurrent/returned-error/drop/late-worker/migrating/background cases; no native or LSP operation"
    );
}

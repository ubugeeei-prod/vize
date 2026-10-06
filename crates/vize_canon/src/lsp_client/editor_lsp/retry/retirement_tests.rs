use super::retry_transient_editor_request;
use crate::corsa_bridge::worker::BoundedWorker;
use futures::{FutureExt, executor::block_on};
use std::{sync::mpsc, time::Duration};
use vize_l0::{String, cstr};

const SETTLE: Duration = Duration::from_secs(10);

#[test]
fn retired_first_failure_finishes_cleanup_without_replacement_and_next_owner_keeps_retry() {
    let worker = BoundedWorker::new("vize-retired-retry-test", (0_u32, 0_u32));
    let (entered, observed) = mpsc::channel();
    let (release, held) = mpsc::channel();
    let (finished, result) = mpsc::channel();
    let mut retired = Box::pin(worker.submit_ready_async(SETTLE, move |state| {
        entered.send(()).unwrap();
        held.recv_timeout(SETTLE).unwrap();
        let error = retry_transient_editor_request::<_, ()>(
            state,
            Err(cstr!("Broken pipe")),
            |state| {
                state.0 += 1;
                Ok(())
            },
            |_| panic!("retired caller must not spawn a replacement"),
        )
        .unwrap_err();
        finished.send(error).unwrap();
    }));
    assert_eq!(retired.as_mut().now_or_never(), None);
    observed.recv_timeout(SETTLE).unwrap();
    drop(retired);
    release.send(()).unwrap();
    assert_eq!(
        result.recv_timeout(SETTLE).unwrap(),
        "Native caller retired before completing its operation; first editor LSP request failed: Broken pipe"
    );
    assert_eq!(
        block_on(worker.submit_ready_async(SETTLE, |state| {
            retry_transient_editor_request(
                state,
                Err(cstr!("Broken pipe")),
                |state| {
                    state.0 += 1;
                    Ok(())
                },
                |state| {
                    state.1 += 1;
                    Ok(*state)
                },
            )
        })),
        Ok(Ok((2, 1)))
    );
    assert!(!crate::lsp_client::lsp_transport_error_is_transient(
        "Native caller retired before completing its operation; first editor LSP request failed: Broken pipe; editor LSP session retirement also failed: broken pipe"
    ));
    assert!(!worker.is_draining());
}

#[test]
fn retirement_during_cleanup_finishes_cleanup_but_starts_no_retry() {
    let worker = BoundedWorker::new("vize-retired-cleanup-test", false);
    let (entered, observed) = mpsc::channel();
    let (release, held) = mpsc::channel();
    let (finished, result) = mpsc::channel();
    let mut retired = Box::pin(worker.submit_ready_async(SETTLE, move |state| {
        let error = retry_transient_editor_request::<_, ()>(
            state,
            Err(cstr!("Broken pipe")),
            |state| {
                entered.send(()).unwrap();
                held.recv_timeout(SETTLE).unwrap();
                *state = true;
                Err(cstr!("broken pipe during old-session shutdown"))
            },
            |_| panic!("retired caller must not retry after cleanup"),
        )
        .unwrap_err();
        finished.send((error, *state)).unwrap();
    }));
    assert_eq!(retired.as_mut().now_or_never(), None);
    observed.recv_timeout(SETTLE).unwrap();
    drop(retired);
    release.send(()).unwrap();
    assert_eq!(
        result.recv_timeout(SETTLE).unwrap(),
        (
            String::from(
                "Native caller retired before completing its operation; first editor LSP request failed: Broken pipe; editor LSP session retirement also failed: broken pipe during old-session shutdown"
            ),
            true
        )
    );
    assert_eq!(
        block_on(worker.submit_ready_async(SETTLE, |state| *state)),
        Ok(true)
    );
    assert!(!worker.is_draining());
}

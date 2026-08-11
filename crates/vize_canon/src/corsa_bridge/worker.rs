//! Deadline-bounded worker thread for synchronous, un-cancellable state.
//!
//! Every Corsa bridge call is synchronous underneath: `corsa`'s project
//! session drives its IPC through [`corsa::runtime::block_on`], so a bridge
//! future never yields. Wrapping such a future in an async `timeout`
//! combinator cannot bound it — the guarded future owns the executor thread
//! and the timer half never gets polled again, which is why the diagnostics
//! pass appeared to promise 10s while `CorsaBridgeConfig::timeout_ms` was read
//! by nothing (#3376).
//!
//! What was left holding the line is `corsa`'s own transport backstop, 30s per
//! request, applied per request and unaware of how many requests a pass makes
//! or that a respawn retry follows. `vize lsp` drives tower-lsp on a single
//! thread, so every one of those waits freezes the entire server — no
//! diagnostics, no responses, no timeout of vize's own.
//!
//! This module bounds the wait where the blocking actually happens. The
//! synchronous state lives on a dedicated worker thread and callers hand it a
//! job plus a hard deadline. When the deadline elapses the caller gives up and
//! reports [`WorkerError::TimedOut`], while the worker keeps draining the
//! abandoned job so the backend transport is never left half-read. Callers
//! that arrive while an abandoned job is still draining fail fast instead of
//! queueing behind it, so a wedged backend costs one deadline in total rather
//! than one deadline per request.
//!
//! The wait is deliberately blocking rather than an `.await`. Making bridge
//! calls genuinely yield would activate the latent `IdeContext` shard-guard
//! deadlock recorded in #3377; bounding without yielding keeps that hazard
//! unreachable.
#![allow(clippy::disallowed_types)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

/// A unit of work executed on the worker thread against the owned state.
type Job<T> = Box<dyn FnOnce(&mut T) + Send>;

/// Why a [`BoundedWorker::submit`] call did not produce a value.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum WorkerError {
    /// The job outran its deadline, or an earlier abandoned job still owns the
    /// worker. Either way the backend did not answer within the bound.
    TimedOut,
    /// The worker thread is gone, so no job can run any more.
    Stopped,
}

/// Owns `T` on a dedicated thread and runs jobs against it under a deadline.
pub(super) struct BoundedWorker<T> {
    /// `None` when the worker thread could not be started.
    jobs: Option<mpsc::Sender<Job<T>>>,
    /// Jobs whose caller already gave up and which the worker is still
    /// draining. Non-zero means the next `submit` must fail fast.
    abandoned: Arc<AtomicUsize>,
}

impl<T: Send + 'static> BoundedWorker<T> {
    /// Move `state` onto a worker thread named `name`.
    ///
    /// A failed thread spawn is not fatal: every later `submit` reports
    /// [`WorkerError::Stopped`], which callers surface as a bridge error.
    ///
    /// Dropping the worker closes the job channel, so the thread finishes what
    /// it is running and then drops `state`. A worker still draining an
    /// abandoned job therefore outlives its owner until that job returns —
    /// the price of never tearing down a transport mid-request.
    #[cfg(test)]
    pub(super) fn new(name: &str, state: T) -> Self {
        Self::new_with_keepalive(name, state, ())
    }

    /// Move `state` and a lifetime owner onto the same worker thread. The
    /// keepalive is dropped only after every abandoned job finishes and the
    /// closed channel drains, so external resources used by `T` cannot vanish
    /// when the [`BoundedWorker`] handle itself is dropped.
    pub(super) fn new_with_keepalive<K: Send + 'static>(
        name: &str,
        mut state: T,
        keepalive: K,
    ) -> Self {
        let (sender, receiver) = mpsc::channel::<Job<T>>();
        let spawned = thread::Builder::new()
            .name(std::string::String::from(name))
            .spawn(move || {
                let _keepalive = keepalive;
                while let Ok(job) = receiver.recv() {
                    job(&mut state);
                }
            });

        Self {
            jobs: spawned.is_ok().then_some(sender),
            abandoned: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Run `f` against the owned state, giving up after `deadline`.
    ///
    /// The returned value is whatever `f` produced. A job that outruns the
    /// deadline is *abandoned*, not cancelled: it keeps running on the worker
    /// thread until it finishes, and its result is dropped. That is deliberate
    /// — the backend transport has no way to withdraw an in-flight request, so
    /// dropping the job mid-request would desynchronize the session.
    pub(super) fn submit<R, F>(&self, deadline: Duration, f: F) -> Result<R, WorkerError>
    where
        F: FnOnce(&mut T) -> R + Send + 'static,
        R: Send + 'static,
    {
        // A previous caller already gave up on a job that still owns the
        // worker. Queueing behind it would make every following request pay
        // the full deadline again, so report the outstanding stall directly.
        if self.abandoned.load(Ordering::Acquire) > 0 {
            return Err(WorkerError::TimedOut);
        }

        let Some(jobs) = self.jobs.as_ref() else {
            return Err(WorkerError::Stopped);
        };

        // A rendezvous channel makes abandonment observable exactly once: the
        // worker's `send` can only complete while this caller is still
        // waiting, and fails as soon as the receiver below is dropped.
        let (result_sender, result_receiver) = mpsc::sync_channel::<R>(0);
        let abandoned = self.abandoned.clone();
        let job: Job<T> = Box::new(move |state| {
            let value = f(state);
            if result_sender.send(value).is_err() {
                abandoned.fetch_sub(1, Ordering::AcqRel);
            }
        });

        if jobs.send(job).is_err() {
            return Err(WorkerError::Stopped);
        }

        match result_receiver.recv_timeout(deadline) {
            Ok(value) => Ok(value),
            Err(RecvTimeoutError::Timeout) => {
                // Recorded before `result_receiver` is dropped, so the worker
                // cannot observe the disconnect before the count is raised.
                self.abandoned.fetch_add(1, Ordering::AcqRel);
                Err(WorkerError::TimedOut)
            }
            Err(RecvTimeoutError::Disconnected) => Err(WorkerError::Stopped),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BoundedWorker, WorkerError};
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    /// Generous upper bound for "the worker observed something"; the
    /// assertions below are on outcomes, never on elapsed time.
    const SETTLE: Duration = Duration::from_secs(10);
    const DEADLINE: Duration = Duration::from_millis(50);

    struct Gate {
        release: mpsc::Sender<()>,
        entered: mpsc::Receiver<()>,
    }

    /// A worker whose first job blocks until the returned gate releases it.
    fn gated_worker() -> (BoundedWorker<()>, Gate, impl Fn(&mut ()) + Send + 'static) {
        let (release, hold) = mpsc::channel::<()>();
        let (entered_tx, entered) = mpsc::channel::<()>();
        let block = move |_: &mut ()| {
            let _ = entered_tx.send(());
            let _ = hold.recv();
        };
        (
            BoundedWorker::new("vize-test-worker", ()),
            Gate { release, entered },
            block,
        )
    }

    #[test]
    fn submit_returns_the_job_result_within_the_deadline() {
        let worker = BoundedWorker::new("vize-test-worker", 40_u32);
        let doubled = worker.submit(SETTLE, |state| {
            *state *= 2;
            *state + 2
        });

        assert_eq!(doubled, Ok(82));
        assert_eq!(worker.submit(SETTLE, |state| *state), Ok(80));
    }

    #[test]
    fn submit_times_out_when_a_job_outruns_the_deadline() {
        let (worker, gate, block) = gated_worker();

        let outcome = worker.submit(DEADLINE, block);

        assert_eq!(outcome, Err(WorkerError::TimedOut));
        gate.entered
            .recv_timeout(SETTLE)
            .expect("job must have run");
        let _ = gate.release.send(());
    }

    #[test]
    fn submit_fails_fast_without_running_a_job_while_an_abandoned_job_drains() {
        let (worker, gate, block) = gated_worker();
        assert_eq!(worker.submit(DEADLINE, block), Err(WorkerError::TimedOut));
        gate.entered
            .recv_timeout(SETTLE)
            .expect("job must have run");

        let (ran_tx, ran) = mpsc::channel::<()>();
        let outcome = worker.submit(DEADLINE, move |_| {
            let _ = ran_tx.send(());
        });

        // The point of the fast path: the second caller is refused outright
        // rather than queued, so its job never reaches the worker.
        assert_eq!(outcome, Err(WorkerError::TimedOut));
        assert!(ran.try_recv().is_err(), "queued behind the abandoned job");

        let _ = gate.release.send(());
    }

    #[test]
    fn submit_recovers_once_the_abandoned_job_finishes() {
        let (worker, gate, block) = gated_worker();
        assert_eq!(worker.submit(DEADLINE, block), Err(WorkerError::TimedOut));
        gate.entered
            .recv_timeout(SETTLE)
            .expect("job must have run");
        let _ = gate.release.send(());

        // The worker clears the abandoned job on its own thread, so retry
        // until it has been observed. The assertion is that service returns,
        // not how quickly it does.
        let started = Instant::now();
        let mut recovered = Err(WorkerError::TimedOut);
        while started.elapsed() < SETTLE {
            recovered = worker.submit(SETTLE, |_| 7_u8);
            if recovered.is_ok() {
                break;
            }
            std::thread::yield_now();
        }

        assert_eq!(recovered, Ok(7));
    }

    #[test]
    fn worker_keeps_external_state_alive_until_an_abandoned_job_finishes() {
        struct DropSignal(mpsc::Sender<()>);
        impl Drop for DropSignal {
            fn drop(&mut self) {
                let _ = self.0.send(());
            }
        }

        let (release, hold) = mpsc::channel::<()>();
        let (entered_tx, entered) = mpsc::channel::<()>();
        let (dropped_tx, dropped) = mpsc::channel::<()>();
        let worker =
            BoundedWorker::new_with_keepalive("vize-test-worker", (), DropSignal(dropped_tx));
        let outcome = worker.submit(DEADLINE, move |_| {
            let _ = entered_tx.send(());
            let _ = hold.recv();
        });
        assert_eq!(outcome, Err(WorkerError::TimedOut));
        entered.recv_timeout(SETTLE).expect("job must have run");

        drop(worker);
        assert!(
            dropped.try_recv().is_err(),
            "keepalive dropped while abandoned work still owned it"
        );
        let _ = release.send(());
        dropped
            .recv_timeout(SETTLE)
            .expect("worker must release keepalive after draining");
    }

    #[test]
    fn submit_reports_a_stopped_worker_when_no_thread_is_backing_it() {
        // Models a failed thread spawn: nothing can run the job, so the caller
        // must be told instead of waiting out its deadline for a lost reply.
        let stopped = BoundedWorker::<()> {
            jobs: None,
            abandoned: Arc::new(AtomicUsize::new(0)),
        };

        assert_eq!(stopped.submit(SETTLE, |_| ()), Err(WorkerError::Stopped));
    }
}

//! Cancellation retires the caller; an entered backend request still drains.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures::channel::oneshot;
use futures::future::{Either, select};

use super::{BoundedWorker, Job, WorkerError, deadline};

#[derive(Default)]
struct JobState {
    abandoned: bool,
    finished: bool,
}

#[derive(Clone)]
struct Lifetime {
    state: Arc<Mutex<JobState>>,
    abandoned: Arc<AtomicUsize>,
}

impl Lifetime {
    fn abandon(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !state.finished && !state.abandoned {
            state.abandoned = true;
            self.abandoned.fetch_add(1, Ordering::AcqRel);
        }
    }

    fn cancelled(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .abandoned
    }

    fn finish(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.finished = true;
        if state.abandoned {
            self.abandoned.fetch_sub(1, Ordering::AcqRel);
        }
    }
}

struct Completion(Lifetime);
impl Drop for Completion {
    fn drop(&mut self) {
        self.0.finish();
    }
}

struct Cancellation(Lifetime);
impl Drop for Cancellation {
    fn drop(&mut self) {
        self.0.abandon();
    }
}

impl<T: Send + 'static> BoundedWorker<T> {
    /// Preserve the configured deadline without blocking the caller's poll.
    /// Dropping this future skips an unentered job. An entered synchronous IPC
    /// request drains, with no result delivered to the cancelled caller.
    pub(in crate::corsa_bridge) async fn submit_async<R, F>(
        &self,
        duration: Duration,
        f: F,
    ) -> Result<R, WorkerError>
    where
        F: FnOnce(&mut T) -> R + Send + 'static,
        R: Send + 'static,
    {
        if self.abandoned.load(Ordering::Acquire) > 0 {
            return Err(WorkerError::TimedOut);
        }
        self.submit_queued(duration, f).await
    }

    async fn submit_queued<R, F>(
        &self,
        duration: Duration,
        f: F,
    ) -> Result<R, WorkerError>
    where
        F: FnOnce(&mut T) -> R + Send + 'static,
        R: Send + 'static,
    {
        let jobs = self.jobs.as_ref().ok_or(WorkerError::Stopped)?;
        let at = Instant::now()
            .checked_add(duration)
            .ok_or(WorkerError::Stopped)?;
        let timeout = deadline::wake_at(at)?;
        let lifetime = Lifetime {
            state: Arc::new(Mutex::new(JobState::default())),
            abandoned: Arc::clone(&self.abandoned),
        };
        let cancellation = Cancellation(lifetime.clone());
        // Construct this before queueing: even a worker panic that drops its
        // remaining queue completes each lifetime instead of leaking a stall.
        let completion = Completion(lifetime);
        let (reply, response) = oneshot::channel();
        let job: Job<T> = Box::new(move |state| {
            if completion.0.cancelled() {
                return;
            }
            let value = f(state);
            drop(completion);
            let _ = reply.send(value);
        });
        jobs.send(job).map_err(|_| WorkerError::Stopped)?;
        let result = match select(response, timeout).await {
            Either::Left((Ok(value), _)) => Ok(value),
            Either::Left((Err(_), _)) | Either::Right((Err(_), _)) => Err(WorkerError::Stopped),
            Either::Right((Ok(()), _)) => Err(WorkerError::TimedOut),
        };
        drop(cancellation);
        result
    }
}

#[cfg(test)]
mod tests;

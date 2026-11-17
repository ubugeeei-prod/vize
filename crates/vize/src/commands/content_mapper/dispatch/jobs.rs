//! Transform jobs waiting for a worker.

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex, PoisonError};

/// Admitted jobs in request order. An idle worker waits on the condition
/// variable with the lock released, and each job wakes one of them. The
/// `Mutex<Receiver>` this replaces was held across the blocking `recv`, so
/// every other idle worker sat in `pthread_mutex_lock` behind it and each job
/// passed through two wakeups.
pub(super) struct Jobs<T> {
    state: Mutex<Queue<T>>,
    ready: Condvar,
}

struct Queue<T> {
    jobs: VecDeque<T>,
    closed: bool,
}

impl<T> Jobs<T> {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(Queue {
                jobs: VecDeque::new(),
                closed: false,
            }),
            ready: Condvar::new(),
        }
    }

    pub(super) fn push(&self, job: T) {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .jobs
            .push_back(job);
        self.ready.notify_one();
    }

    /// The next job, or `None` once the queue is closed and drained.
    pub(super) fn pop(&self) -> Option<T> {
        let mut queue = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        loop {
            if let Some(job) = queue.jobs.pop_front() {
                return Some(job);
            }
            if queue.closed {
                return None;
            }
            queue = self
                .ready
                .wait(queue)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// No more jobs: workers finish what is queued, then stop.
    pub(super) fn close(&self) {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .closed = true;
        self.ready.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::Jobs;

    #[test]
    fn workers_drain_every_job_in_order_then_stop_on_close() {
        let jobs = Jobs::new();
        let (sent, received) = thread::scope(|scope| {
            let workers = (0..4)
                .map(|_| {
                    scope.spawn(|| {
                        let mut taken = Vec::new();
                        while let Some(job) = jobs.pop() {
                            taken.push(job);
                        }
                        taken
                    })
                })
                .collect::<Vec<_>>();
            for job in 0..1000 {
                jobs.push(job);
            }
            jobs.close();
            let mut received = workers
                .into_iter()
                .flat_map(|worker| worker.join().unwrap())
                .collect::<Vec<_>>();
            received.sort_unstable();
            ((0..1000).collect::<Vec<_>>(), received)
        });
        assert_eq!(received, sent);
    }

    #[test]
    fn a_closed_queue_still_hands_out_what_was_queued() {
        let jobs = Jobs::new();
        jobs.push(1);
        jobs.push(2);
        jobs.close();
        assert_eq!(
            (jobs.pop(), jobs.pop(), jobs.pop()),
            (Some(1), Some(2), None)
        );
    }
}

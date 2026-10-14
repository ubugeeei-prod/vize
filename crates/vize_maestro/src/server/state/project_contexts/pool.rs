//! Fixed config workers and bounded queue keep Node imports off the executor.
#![expect(
    clippy::disallowed_types,
    reason = "config jobs transfer cached native owners across fixed worker threads"
)]
use futures::{channel::oneshot, future::poll_fn};
use parking_lot::{Condvar, Mutex};
use std::{
    collections::VecDeque,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Poll, Waker},
};
use vize_l0::FxHashMap;
type Job = Box<dyn FnOnce() + Send>;
const CAPACITY: usize = 16;
#[derive(Default)]
struct QueueState {
    jobs: VecDeque<Job>,
    waiting: FxHashMap<usize, Waker>,
}
#[derive(Default)]
struct Queue {
    state: Mutex<QueueState>,
    available: Condvar,
    next_waiter: AtomicUsize,
}
struct Pool(Arc<Queue>);
struct Waiting<'a>(&'a Queue, usize);
impl Drop for Waiting<'_> {
    fn drop(&mut self) {
        self.0.state.lock().waiting.remove(&self.1);
    }
}
impl Queue {
    async fn enqueue(&self, job: Job) {
        let id = self.next_waiter.fetch_add(1, Ordering::Relaxed);
        let _waiting = Waiting(self, id);
        let mut job = Some(job);
        poll_fn(|context| {
            let mut state = self.state.lock();
            if state.jobs.len() == CAPACITY {
                state.waiting.insert(id, context.waker().clone());
                return Poll::Pending;
            }
            if let Some(job) = job.take() {
                state.jobs.push_back(job);
            }
            drop(state);
            self.available.notify_one();
            Poll::Ready(())
        })
        .await;
    }
    fn take(&self) -> Job {
        loop {
            let mut state = self.state.lock();
            if let Some(job) = state.jobs.pop_front() {
                let waiting = std::mem::take(&mut state.waiting);
                drop(state);
                for waker in waiting.into_values() {
                    waker.wake();
                }
                return job;
            }
            self.available.wait(&mut state);
        }
    }
}
fn workers() -> Option<&'static Pool> {
    static WORKERS: OnceLock<Option<Pool>> = OnceLock::new();
    WORKERS
        .get_or_init(|| {
            let queue = Arc::new(Queue::default());
            let mut started = 0;
            for index in 0..2 {
                let queue = queue.clone();
                match std::thread::Builder::new()
                    .name(vize_l0::cstr!("vize-project-config-{index}").into())
                    .spawn(move || {
                        loop {
                            let job = queue.take();
                            if std::panic::catch_unwind(std::panic::AssertUnwindSafe(job)).is_err()
                            {
                                tracing::error!("project configuration worker job panicked");
                            }
                        }
                    }) {
                    Ok(_) => started += 1,
                    Err(error) => {
                        tracing::warn!("failed to start project configuration worker: {error}")
                    }
                }
            }
            (started != 0).then_some(Pool(queue))
        })
        .as_ref()
}
pub(super) async fn run<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    let pool = workers()?;
    let (sender, receiver) = oneshot::channel();
    pool.0
        .enqueue(Box::new(move || {
            let _ = sender.send(work());
        }))
        .await;
    receiver.await.ok()
}

#[cfg(test)]
mod tests {
    use super::{CAPACITY, Queue};
    use futures::{Future, FutureExt};
    use std::task::{Context, Poll};

    #[test]
    fn canceled_shared_admission_never_retains_the_queue_lock() {
        let queue = Queue::default();
        for _ in 0..CAPACITY {
            queue.state.lock().jobs.push_back(Box::new(|| {}));
        }
        let mut context = Context::from_waker(futures::task::noop_waker_ref());
        // Like a registry-held Shared initializer after its requester cancels,
        // keep the pending admission future alive without polling it again.
        let registry = {
            let pending = queue.enqueue(Box::new(|| {})).shared();
            let registry = pending.clone();
            futures::pin_mut!(pending);
            assert!(matches!(pending.as_mut().poll(&mut context), Poll::Pending));
            registry
        };
        (queue.take())();
        assert_eq!(queue.state.lock().jobs.len(), CAPACITY - 1);
        futures::executor::block_on(queue.enqueue(Box::new(|| {})));
        assert_eq!(queue.state.lock().jobs.len(), CAPACITY);
        assert!(registry.peek().is_none());
    }
}

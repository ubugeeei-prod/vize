use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use futures::{FutureExt, StreamExt};

use super::Worker;
use crate::runtime::block_on;

#[test]
fn navigation_worker_is_lazy_reuses_one_thread_and_keeps_inventory_owners_separate() {
    block_on(async {
        let worker = Worker::default();
        assert!(worker.sender.lock().await.is_none());
        let executor = std::thread::current().id();
        let first = worker.run(|| std::thread::current().id()).await.unwrap();
        assert_ne!(first, executor);
        for index in 0..40 {
            assert_eq!(
                worker
                    .run(move || (std::thread::current().id(), index))
                    .await,
                Some((first, index))
            );
        }
        let other = Worker::default();
        assert_ne!(other.run(|| std::thread::current().id()).await, Some(first));
    });
}

#[test]
fn overlapping_navigation_requests_run_once_on_one_existing_worker() {
    block_on(async {
        let worker = Worker::default();
        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let jobs = (0..40).map(|index| {
            let active = Arc::clone(&active);
            let peak = Arc::clone(&peak);
            worker.run(move || {
                let count = active.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(count, Ordering::SeqCst);
                std::thread::yield_now();
                active.fetch_sub(1, Ordering::SeqCst);
                (index, std::thread::current().id())
            })
        });
        let results = futures::future::join_all(jobs).await;
        let first = results[0].unwrap().1;
        assert_eq!(
            results,
            (0..40)
                .map(|index| Some((index, first)))
                .collect::<Vec<_>>()
        );
        assert_eq!(active.load(Ordering::SeqCst), 0);
        assert_eq!(peak.load(Ordering::SeqCst), 1);
    });
}

struct Release(Arc<AtomicUsize>);
impl Drop for Release {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn cancelled_receiver_releases_its_work_and_preserves_the_worker_for_the_next_request() {
    block_on(async {
        let worker = Worker::default();
        let released = Arc::new(AtomicUsize::new(0));
        let owner = Release(Arc::clone(&released));
        let (entered, entering) = std::sync::mpsc::channel();
        let (resume, resuming) = std::sync::mpsc::channel();
        let mut request = Box::pin(worker.run(move || {
            let _owner = owner;
            let thread = std::thread::current().id();
            entered.send(thread).unwrap();
            resuming
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            thread
        }));
        assert!(request.as_mut().now_or_never().is_none());
        let first = entering
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        drop(request);
        assert_eq!(released.load(Ordering::SeqCst), 0);
        resume.send(()).unwrap();
        assert_eq!(
            worker.run(|| std::thread::current().id()).await,
            Some(first)
        );
        assert_eq!(released.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn failed_native_work_returns_none_and_a_later_request_restarts_the_worker() {
    block_on(async {
        let worker = Worker::default();
        let first = worker.run(|| std::thread::current().id()).await.unwrap();
        assert_eq!(
            worker
                .run(|| panic!("intentional native worker failure"))
                .await,
            None::<()>
        );
        let next = worker.run(|| std::thread::current().id()).await.unwrap();
        assert_ne!(next, first);
        assert_eq!(worker.run(|| 42).await, Some(42));
    });
}

#[test]
fn queued_and_backpressured_cancellation_release_their_original_owners_once() {
    block_on(async {
        let worker = Worker::default();
        let (entered, entering) = std::sync::mpsc::channel();
        let (resume, resuming) = std::sync::mpsc::channel();
        let mut active = Box::pin(worker.run(move || {
            let thread = std::thread::current().id();
            entered.send(thread).unwrap();
            resuming
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            thread
        }));
        assert!(active.as_mut().now_or_never().is_none());
        let first = entering
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        // The first command is executing; advance its sender flush before
        // filling the single pending channel slot with a second command.
        assert!(active.as_mut().now_or_never().is_none());
        let queued_released = Arc::new(AtomicUsize::new(0));
        let queued_owner = Release(Arc::clone(&queued_released));
        let (completed, completing) = std::sync::mpsc::channel();
        let mut queued = Box::pin(worker.run(move || {
            let _owner = queued_owner;
            completed.send(std::thread::current().id()).unwrap();
        }));
        assert!(queued.as_mut().now_or_never().is_none());
        drop(queued);
        // Cancelling the waiting receiver must not pretend a committed
        // command vanished: its owned input remains until that command runs.
        assert_eq!(queued_released.load(Ordering::SeqCst), 0);
        assert!(completing.try_recv().is_err());

        let waiting_released = Arc::new(AtomicUsize::new(0));
        let waiting_owner = Release(Arc::clone(&waiting_released));
        let never_executed = Arc::new(AtomicUsize::new(0));
        let executions = Arc::clone(&never_executed);
        let mut waiting = Box::pin(worker.run(move || {
            let _owner = waiting_owner;
            executions.fetch_add(1, Ordering::SeqCst);
        }));
        assert!(waiting.as_mut().now_or_never().is_none());
        drop(waiting);
        assert_eq!(waiting_released.load(Ordering::SeqCst), 1);
        assert_eq!(never_executed.load(Ordering::SeqCst), 0);

        resume.send(()).unwrap();
        assert_eq!(active.await, Some(first));
        assert_eq!(
            completing
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap(),
            first
        );
        assert_eq!(worker.run(|| 42).await, Some(42));
        assert_eq!(queued_released.load(Ordering::SeqCst), 1);
        assert_eq!(waiting_released.load(Ordering::SeqCst), 1);
        assert_eq!(never_executed.load(Ordering::SeqCst), 0);
    });
}

struct ThreadExit(std::sync::mpsc::Sender<std::thread::ThreadId>);
impl Drop for ThreadExit {
    fn drop(&mut self) {
        let _ = self.0.send(std::thread::current().id());
    }
}

thread_local! {
    static ON_WORKER_EXIT: std::cell::RefCell<Option<ThreadExit>> = const {
        std::cell::RefCell::new(None)
    };
}

#[test]
fn dropping_an_inventory_terminates_its_idle_native_worker() {
    block_on(async {
        let inventory = super::super::Inventory::default();
        let (exited, exiting) = std::sync::mpsc::channel();
        let first = inventory
            .worker
            .run(move || {
                ON_WORKER_EXIT.with(|owner| *owner.borrow_mut() = Some(ThreadExit(exited)));
                std::thread::current().id()
            })
            .await
            .unwrap();
        assert!(exiting.try_recv().is_err());
        drop(inventory);
        // A native thread-local destructor observes actual thread exit,
        // rather than merely observing sender or request-future destruction.
        assert_eq!(
            exiting
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap(),
            first
        );
    });
}

#[test]
fn lost_reply_retirement_keeps_a_later_live_worker() {
    block_on(async {
        let worker = Worker::default();
        let (sender, mut receiver) = futures::channel::mpsc::channel::<super::Work>(0);
        *worker.sender.lock().await = Some(super::Connection {
            sender,
            identity: Arc::new(()),
        });
        let mut first = Box::pin(worker.run(|| 1));
        assert!(first.as_mut().now_or_never().is_none());
        let first_command = receiver.next().now_or_never().unwrap().unwrap();
        assert!(first.as_mut().now_or_never().is_none());
        let mut second = Box::pin(worker.run(|| 2));
        assert!(second.as_mut().now_or_never().is_none());
        let second_command = receiver.next().now_or_never().unwrap().unwrap();
        assert!(second.as_mut().now_or_never().is_none());
        assert!(
            !worker
                .sender
                .lock()
                .await
                .as_ref()
                .unwrap()
                .sender
                .is_closed()
        );

        // Explicit lost-reply injection: keep the old receiver alive while
        // dropping its original command, then create a genuine native worker.
        drop(first_command);
        assert_eq!(first.await, None);
        let replacement = worker.run(|| std::thread::current().id()).await.unwrap();
        // A delayed failure from the old connection cannot retire that newer
        // worker or make its next request spawn another native thread.
        drop(second_command);
        assert_eq!(second.await, None);
        assert_eq!(
            worker.run(|| std::thread::current().id()).await,
            Some(replacement)
        );
    });
}

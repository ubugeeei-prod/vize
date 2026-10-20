//! Cancel requests admitted before a source-set edit.
//!
//! Every request answers for the open sources it was admitted against; after
//! any edit its reply is refused as `ContentModified`. Each edit cancels the
//! current [`CancelToken`] and wakes the waiting requests, so they stop at
//! their next poll instead of finishing work whose reply would be refused.
#![expect(
    clippy::disallowed_types,
    reason = "wakers are shared between the waiting request and the editing thread"
)]

use std::future::Future;
use std::sync::{Arc, Weak};
use std::task::Poll;

use futures::task::AtomicWaker;
use parking_lot::Mutex;
use vize_incremental::snapshot::cancel::CancelToken;

pub(super) struct EditCancellation {
    current: Mutex<CancelToken>,
    waiters: Mutex<Vec<Weak<AtomicWaker>>>,
}

impl Default for EditCancellation {
    fn default() -> Self {
        Self {
            current: Mutex::new(CancelToken::root()),
            waiters: Mutex::new(Vec::new()),
        }
    }
}

impl EditCancellation {
    /// Cancel every request admitted so far.
    pub(super) fn cancel(&self) {
        let previous = std::mem::replace(&mut *self.current.lock(), CancelToken::root());
        previous.cancel();
        let waiters = std::mem::take(&mut *self.waiters.lock());
        for waiter in waiters.iter().filter_map(Weak::upgrade) {
            waiter.wake();
        }
    }

    /// Run `future` until it completes, or return `None` once an edit lands.
    pub(super) async fn until_edited<F: Future>(&self, future: F) -> Option<F::Output> {
        let token = self.current.lock().child();
        let waker = Arc::new(AtomicWaker::new());
        {
            let mut waiters = self.waiters.lock();
            waiters.retain(|waiter| waiter.strong_count() > 0);
            waiters.push(Arc::downgrade(&waker));
        }
        let mut future = std::pin::pin!(future);
        std::future::poll_fn(|cx| {
            // Register before checking, so a cancel after the check still wakes.
            waker.register(cx.waker());
            if token.is_cancelled() {
                return Poll::Ready(None);
            }
            future.as_mut().poll(cx).map(Some)
        })
        .await
    }
}

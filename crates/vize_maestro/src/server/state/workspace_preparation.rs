//! Let one native request build a workspace-wide projection between two
//! attempts instead of inside its native transaction.
//!
//! The transaction owner arms deferral. When its request reaches a workspace
//! open that is not prepared yet, the request parks a preparation here and
//! waits; the owner abandons the attempt, releases the transaction, runs the
//! preparation and starts the request over.

use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Poll, Waker};

pub(crate) type Preparation = Pin<Box<dyn Future<Output = ()> + Send>>;

#[derive(Default)]
pub(crate) struct WorkspacePreparation {
    armed: AtomicBool,
    parked: Mutex<Parked>,
}

#[derive(Default)]
struct Parked {
    preparation: Option<Preparation>,
    waiting: Vec<Waker>,
}

impl WorkspacePreparation {
    /// Only the request holding the native transaction may arm deferral.
    pub(crate) fn arm(&self) -> Armed<'_> {
        self.armed.store(true, Ordering::Release);
        Armed(self)
    }

    pub(crate) fn is_armed(&self) -> bool {
        self.armed.load(Ordering::Acquire)
    }

    /// Park `preparation` for the armed owner and wait until deferral ends.
    /// The owner drops its attempt while this waits, so the owner's request
    /// never resumes here. Any other caller resumes and opens inline.
    pub(crate) async fn defer(&self, preparation: Preparation) {
        self.parked().preparation = Some(preparation);
        std::future::poll_fn(|cx| {
            if !self.is_armed() {
                return Poll::Ready(());
            }
            self.parked().waiting.push(cx.waker().clone());
            Poll::Pending
        })
        .await;
    }

    pub(crate) fn take(&self) -> Option<Preparation> {
        self.parked().preparation.take()
    }

    fn parked(&self) -> std::sync::MutexGuard<'_, Parked> {
        self.parked
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Deferral ends with the owner's attempt, whether it finished, was abandoned
/// for a preparation, or was cancelled.
pub(crate) struct Armed<'a>(&'a WorkspacePreparation);

impl Drop for Armed<'_> {
    fn drop(&mut self) {
        self.0.armed.store(false, Ordering::Release);
        let waiting = {
            let mut parked = self.0.parked();
            parked.preparation = None;
            std::mem::take(&mut parked.waiting)
        };
        for waker in waiting {
            waker.wake();
        }
    }
}

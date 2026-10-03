//! Cascade-cancellation tokens: one per snapshot task, each a child of the
//! task that spawned it (file → block → region). Cancelling a token cancels
//! its whole subtree; a task polls [`CancelToken::is_cancelled`] between
//! units of stage work and stops with [`Cancelled`].

use std::sync::atomic::{AtomicBool, Ordering};

/// The shared node behind a token.
#[derive(Debug)]
struct Node {
    cancelled: AtomicBool,
    parent: Option<CancelToken>,
}

/// A cancellation token. Clones share the flag.
#[derive(Debug, Clone)]
pub struct CancelToken(
    #[expect(
        clippy::disallowed_types,
        reason = "tokens are shared by the task that polls them and the owner that cancels \
                  them, across worker threads; a scoped reference cannot span both lifetimes"
    )]
    std::sync::Arc<Node>,
);

/// A task stopped because its token (or an ancestor's) was cancelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cancelled;

impl CancelToken {
    /// A root token (a file's).
    #[must_use]
    pub fn root() -> Self {
        Self::with_parent(None)
    }

    #[expect(clippy::disallowed_types, reason = "builds the shared token node")]
    fn with_parent(parent: Option<CancelToken>) -> Self {
        Self(std::sync::Arc::new(Node {
            cancelled: AtomicBool::new(false),
            parent,
        }))
    }

    /// A child token: cancelled when it or any ancestor is.
    #[must_use]
    pub fn child(&self) -> Self {
        Self::with_parent(Some(self.clone()))
    }

    /// Cancel this token and, through the ancestor check, every descendant.
    pub fn cancel(&self) {
        self.0.cancelled.store(true, Ordering::Release);
    }

    /// Whether this token or an ancestor was cancelled.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        let mut node = Some(self);
        while let Some(token) = node {
            if token.0.cancelled.load(Ordering::Acquire) {
                return true;
            }
            node = token.0.parent.as_ref();
        }
        false
    }

    /// `Err(Cancelled)` once cancelled — the poll between units of work.
    pub fn check(&self) -> Result<(), Cancelled> {
        if self.is_cancelled() {
            Err(Cancelled)
        } else {
            Ok(())
        }
    }
}

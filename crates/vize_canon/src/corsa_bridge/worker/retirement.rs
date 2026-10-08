//! Cooperative checkpoints for an entered native worker operation.
#![expect(
    clippy::disallowed_types,
    reason = "one shared caller lease across scoped workers"
)]

use std::{
    cell::RefCell,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use vize_l0::{String, cstr};

pub(crate) const INCOMPLETE: &str = "Native caller retired before completing its operation";

/// Opaque caller lifetime; absent outside an entered asynchronous worker job.
#[derive(Clone, Default)]
pub(crate) struct Control(Option<Arc<AtomicBool>>);

thread_local! {
    static CURRENT: RefCell<Control> = const { RefCell::new(Control(None)) };
}

impl Control {
    pub(super) fn new() -> Self {
        Self(Some(Arc::new(AtomicBool::new(false))))
    }

    pub(super) fn retire(&self) {
        if let Some(retired) = &self.0 {
            retired.store(true, Ordering::Release);
        }
    }

    /// Check only between complete operations, never during RPC or cleanup.
    pub(crate) fn checkpoint(&self) -> Result<(), String> {
        if self
            .0
            .as_ref()
            .is_some_and(|retired| retired.load(Ordering::Acquire))
        {
            Err(cstr!("{INCOMPLETE}"))
        } else {
            Ok(())
        }
    }

    pub(super) fn enter(&self) -> Scope {
        Scope(CURRENT.with(|current| current.replace(self.clone())))
    }
}

pub(crate) fn capture() -> Control {
    CURRENT.with(|current| current.borrow().clone())
}

pub(crate) fn checkpoint() -> Result<(), String> {
    capture().checkpoint()
}

pub(super) struct Scope(Control);
impl Drop for Scope {
    fn drop(&mut self) {
        CURRENT.with(|current| {
            current.replace(self.0.clone());
        });
    }
}

#[cfg(test)]
mod tests;

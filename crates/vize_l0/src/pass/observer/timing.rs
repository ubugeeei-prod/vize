//! The timing observer: per-pass durations from a caller-supplied clock.
#![expect(clippy::todo, reason = "skeleton: #6833")]

use alloc::vec::Vec;

use super::{PassEvent, PassObserver};

/// A monotonic clock; `no_std` callers bring their own.
pub trait Clock {
    /// Nanoseconds since an arbitrary origin.
    fn now_ns(&self) -> u64;
}

/// One pass duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassTiming {
    /// The pass.
    pub pass: &'static str,
    /// Elapsed nanoseconds.
    pub elapsed_ns: u64,
}

/// Times every pass with `C`.
#[derive(Debug, Clone)]
pub struct TimingObserver<C: Clock> {
    clock: C,
    timings: Vec<PassTiming>,
}

impl<C: Clock> TimingObserver<C> {
    /// A timing observer reading `clock`.
    #[must_use]
    pub const fn new(clock: C) -> Self {
        Self {
            clock,
            timings: Vec::new(),
        }
    }

    /// The timings recorded so far.
    #[must_use]
    pub fn timings(&self) -> &[PassTiming] {
        &self.timings
    }
}

impl<C: Clock> PassObserver for TimingObserver<C> {
    fn before_pass(&mut self, event: &PassEvent<'_>) {
        let _ = (event, self.clock.now_ns());
        todo!()
    }

    fn after_pass(&mut self, event: &PassEvent<'_>) {
        let _ = event;
        todo!()
    }
}

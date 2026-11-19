//! Opt-in diagnostic observation, separate from process-global budget totals.

#[cfg(feature = "allocation-window-diagnostics")]
mod enabled;
#[cfg(feature = "allocation-window-diagnostics")]
pub use enabled::{begin_session, finish_session, set_context};
#[cfg(feature = "allocation-window-diagnostics")]
pub(super) use enabled::{record, window};

#[cfg(not(feature = "allocation-window-diagnostics"))]
#[inline]
/// Without the explicit diagnostic feature, observation is absent.
pub fn begin_session() {}
#[cfg(not(feature = "allocation-window-diagnostics"))]
#[inline]
/// Without the explicit diagnostic feature, observation is absent.
pub fn set_context(_: u64) {}
#[cfg(not(feature = "allocation-window-diagnostics"))]
#[inline]
/// Without the explicit diagnostic feature, observation is absent.
pub fn finish_session() {}

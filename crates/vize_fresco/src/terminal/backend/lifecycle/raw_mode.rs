//! Native raw-mode ownership and lock-free emergency restoration.

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(not(any(unix, windows)))]
pub(super) use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
#[cfg(unix)]
pub(super) use unix::{
    disable_raw_mode, emergency_restore_raw_mode, enable_raw_mode, raw_mode_requires_restoration,
};
#[cfg(windows)]
pub(super) use windows::{
    disable_raw_mode, emergency_restore_raw_mode, enable_raw_mode, raw_mode_requires_restoration,
};

/// Return whether a failed raw-mode transition may still require restoration.
#[cfg(not(any(unix, windows)))]
pub(super) const fn raw_mode_requires_restoration() -> bool {
    false
}

#[cfg(all(test, unix))]
mod tests;
#[cfg(all(test, windows))]
mod windows_tests;

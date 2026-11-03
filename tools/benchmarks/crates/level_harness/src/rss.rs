//! Peak-RSS sampling via `getrusage(2)`.
//!
//! Platform semantics, normalized here so no caller ever sees a raw value:
//!
//! - `ru_maxrss` is reported in **bytes on macOS** and **kilobytes on Linux**;
//!   [`peak_rss_bytes`] converts both to bytes.
//! - `ru_maxrss` is a **process-wide monotone peak**: it never decreases, and
//!   it accumulates across every bench that runs in the same process. Reports
//!   therefore always carry the baseline-subtracted delta
//!   ([`delta_since_baseline`]) per bench process - never the raw peak.
//! - Platforms without `getrusage` support report `None`, which the exporter
//!   serializes as `null`.

use std::sync::OnceLock;

static BASELINE: OnceLock<Option<u64>> = OnceLock::new();

/// Absolute peak RSS of this process in bytes.
///
/// `Some` on macOS and Linux, `None` elsewhere or when `getrusage` fails.
pub fn peak_rss_bytes() -> Option<u64> {
    imp::peak_rss_bytes()
}

/// Record the process baseline peak RSS. The first call wins; [`crate::main!`]
/// calls this before any bench group runs.
pub fn capture_baseline() {
    let _ = BASELINE.set(peak_rss_bytes());
}

/// Peak-RSS growth in bytes since [`capture_baseline`].
///
/// If nobody captured a baseline, the first call does so lazily - the delta
/// then measures growth from this point, which for a bench process that
/// forgot [`crate::main!`] degrades to a small value rather than a misleading
/// process-wide total.
pub fn delta_since_baseline() -> Option<u64> {
    let baseline = *BASELINE.get_or_init(peak_rss_bytes);
    match (peak_rss_bytes(), baseline) {
        (Some(now), Some(base)) => Some(now.saturating_sub(base)),
        _ => None,
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod imp {
    pub(super) fn peak_rss_bytes() -> Option<u64> {
        // SAFETY: `rusage` is plain data, so a zeroed value is a valid
        // initializer for `getrusage` to overwrite.
        let mut usage: libc::rusage = unsafe { core::mem::zeroed() };
        // SAFETY: `RUSAGE_SELF` is a valid `who` argument and `usage` is a
        // live, writable `rusage` out-pointer for the duration of the call.
        let rc = unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) };
        if rc != 0 {
            return None;
        }
        if usage.ru_maxrss < 0 {
            return None;
        }
        let raw = usage.ru_maxrss as u64;
        // ru_maxrss unit: bytes on macOS, kilobytes on Linux.
        #[cfg(target_os = "macos")]
        let bytes = raw;
        #[cfg(target_os = "linux")]
        let bytes = raw.saturating_mul(1024);
        Some(bytes)
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod imp {
    pub(super) fn peak_rss_bytes() -> Option<u64> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn peak_rss_is_reported_and_positive() {
        let peak = peak_rss_bytes().expect("getrusage must succeed on macOS/Linux");
        assert!(peak > 0);
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn delta_is_baseline_subtracted() {
        capture_baseline();
        let delta = delta_since_baseline().expect("delta must be reported on macOS/Linux");
        let now = peak_rss_bytes().expect("getrusage must succeed on macOS/Linux");
        // The delta can never exceed the absolute peak: it is subtraction,
        // not a raw readout.
        assert!(delta <= now);
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    #[test]
    fn unsupported_platforms_report_none() {
        assert_eq!(peak_rss_bytes(), None);
        assert_eq!(delta_since_baseline(), None);
    }
}

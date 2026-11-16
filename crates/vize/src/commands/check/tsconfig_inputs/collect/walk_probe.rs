//! Test-only record of the files an input walk examined past their directory
//! entry: matched against patterns, normalized, or stat'd. Only roots a test
//! observes are recorded, so parallel tests stay independent.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

static PROBES: Mutex<Vec<(PathBuf, Vec<PathBuf>)>> = Mutex::new(Vec::new());

/// Start recording the files examined below `root`.
pub(crate) fn observe(root: &Path) {
    PROBES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push((root.to_path_buf(), Vec::new()));
}

pub(crate) fn record(path: &Path) {
    let mut probes = PROBES.lock().unwrap_or_else(PoisonError::into_inner);
    for (root, examined) in probes.iter_mut() {
        if path.starts_with(root.as_path()) {
            examined.push(path.to_path_buf());
        }
    }
}

/// Stop observing `root` and return the files examined below it, sorted.
pub(crate) fn take(root: &Path) -> Vec<PathBuf> {
    let mut probes = PROBES.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(index) = probes.iter().position(|(observed, _)| observed == root) else {
        return Vec::new();
    };
    let mut examined = probes.swap_remove(index).1;
    examined.sort();
    examined.dedup();
    examined
}

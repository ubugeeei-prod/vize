//! Shared environment helpers for the Nuxt tsconfig CLI oracles. Every Nuxt CLI
//! test binary includes this module, so individual binaries use only a subset.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// Resolves the tsgo executable the Nuxt CLI oracles check against.
pub(crate) fn resolve_test_corsa_path() -> Option<PathBuf> {
    std::env::var_os("CORSA_PATH")
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .or_else(|| {
            let path = workspace_node_modules().join(".bin/tsgo");
            path.exists().then_some(path)
        })
}

pub(crate) fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

pub(crate) fn workspace_node_modules() -> PathBuf {
    std::env::var_os("VIZE_TEST_NODE_MODULES")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("node_modules"))
}

/// Repeat budget for the Nuxt config oracles. CI has to repeat them, so an
/// unset or undersized budget fails closed there instead of silently running
/// a single iteration.
pub(crate) fn required_iterations() -> usize {
    let iterations = match std::env::var_os("VIZE_NUXT_CONFIG_ITERATIONS") {
        Some(raw) => {
            let raw = raw.to_string_lossy();
            let iterations = raw.parse::<usize>().unwrap_or_else(|_| {
                panic!("VIZE_NUXT_CONFIG_ITERATIONS must be an integer: {raw}")
            });
            assert!(iterations > 0, "Nuxt config iterations must be positive");
            iterations
        }
        None => 1,
    };
    if std::env::var_os("CI").is_some() {
        assert!(
            iterations >= 100,
            "CI must repeat the Nuxt config oracles at least 100x"
        );
    }
    iterations
}

#[cfg(unix)]
pub(crate) fn create_fifo(path: &Path) {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};

    let path = CString::new(path.as_os_str().as_bytes()).unwrap();
    // SAFETY: `path` is a NUL-terminated filesystem path and mode is valid.
    assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
}

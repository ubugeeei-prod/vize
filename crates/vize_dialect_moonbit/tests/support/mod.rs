//! Shared fixture plumbing for the MoonBit dialect suites.
//!
//! Each fixture `<name>.vue` has three committed goldens beside it:
//! `<name>.vue.mbt` (the projection), `<name>.moonc.jsonl` (what the
//! pinned `moonc` answered) and `<name>.diagnostics` (the mapped
//! rendering). `VIZE_UPDATE_MOONBIT_FIXTURES=1` rewrites a golden instead
//! of comparing it; every comparison is exact.

#![expect(clippy::panic, reason = "tests assert by panicking")]

use std::path::PathBuf;

/// The committed fixtures, in order.
pub const FIXTURES: [&str; 2] = ["todo", "todo-typos"];

/// The fixtures directory.
pub fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Read `<name><suffix>` from the fixtures directory.
pub fn read(name: &str, suffix: &str) -> vize_s0::String {
    let path = dir().join([name, suffix].concat());
    std::fs::read_to_string(&path)
        .map_or_else(|error| panic!("{}: {error}", path.display()), Into::into)
}

/// Compare `actual` with the golden `<name><suffix>` exactly, or rewrite
/// it under `VIZE_UPDATE_MOONBIT_FIXTURES=1`.
pub fn golden(name: &str, suffix: &str, actual: &str) {
    let path = dir().join([name, suffix].concat());
    if std::env::var_os("VIZE_UPDATE_MOONBIT_FIXTURES").is_some() {
        std::fs::write(&path, actual).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "{}: {error} (VIZE_UPDATE_MOONBIT_FIXTURES=1 writes it)",
            path.display()
        )
    });
    assert_eq!(actual, expected, "golden {} differs", path.display());
}

/// The pinned toolchain version (`.moonbit-version` at the repo root).
pub fn pinned_toolchain() -> vize_s0::String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.moonbit-version");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
        .trim()
        .into()
}

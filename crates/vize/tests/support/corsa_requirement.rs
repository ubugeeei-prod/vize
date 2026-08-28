use std::path::{Path, PathBuf};

use vize_s0::corsa_resolver::{CorsaResolveRequest, resolve_corsa_executable};

const MISSING_REQUIRED_TSGO: &str =
    "VIZE_TEST_REQUIRE_TSGO is set, but no TypeScript 7/Corsa executable was found";

pub(crate) trait CorsaPathValue {
    fn from_corsa_path(path: PathBuf) -> Self;
}

impl CorsaPathValue for PathBuf {
    fn from_corsa_path(path: PathBuf) -> Self {
        path
    }
}

impl CorsaPathValue for String {
    fn from_corsa_path(path: PathBuf) -> Self {
        path.display().to_string()
    }
}

impl CorsaPathValue for vize_s0::String {
    fn from_corsa_path(path: PathBuf) -> Self {
        path.display().to_string().into()
    }
}

/// Keep dependency-free local runs skippable, but make required CI lanes fail
/// closed. `VIZE_TEST_DISABLE_TSGO` is the explicit opt-out used by the canon
/// tests and intentionally takes precedence over discovery. The env var names
/// remain legacy-compatible, but discovery falls back to the TypeScript 7
/// platform runtime that production Vize resolves.
pub(crate) fn required_or_skip<T: CorsaPathValue>(resolved: Option<T>) -> Option<T> {
    let disabled = std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some();
    let resolved = if disabled {
        resolved
    } else {
        resolved.or_else(|| resolve_fallback_runtime().map(T::from_corsa_path))
    };
    required_or_skip_with(
        resolved,
        std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_some(),
        disabled,
    )
}

pub(crate) fn required_or_skip_with<T>(
    resolved: Option<T>,
    required: bool,
    disabled: bool,
) -> Option<T> {
    if disabled {
        return None;
    }
    assert!(resolved.is_some() || !required, "{MISSING_REQUIRED_TSGO}");
    resolved
}

fn resolve_fallback_runtime() -> Option<PathBuf> {
    let root = workspace_root()?;
    resolve_corsa_executable(CorsaResolveRequest {
        explicit_path: None,
        project_root: Some(&root),
    })
    .ok()
}

fn workspace_root() -> Option<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
}

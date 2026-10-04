//! Bounded actual path observations. No resolver, File or link authority.
#![allow(
    dead_code,
    reason = "private physical provider; native module-link consumer remains unfinished"
)]
use super::{ModuleLinkContextError, ModuleTargetGateError};
use crate::source_project::SnapshotRefusal;
use std::{
    io,
    path::{Path, PathBuf},
};
#[cfg(not(unix))]
use tower_lsp::lsp_types::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TargetPolicy {
    UnsupportedPlatform,
    Root,
    SourceUri,
    SourceParent,
    Candidate,
    EmptyBatch,
    NonRegular,
    NonDirectory,
    Uri,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TargetOperation {
    Open,
    Metadata,
}

#[derive(Debug)]
pub(crate) enum ModuleTargetError {
    Source(SnapshotRefusal),
    Context(ModuleLinkContextError),
    EventSuperseded,
    ForeignSnapshot,
    Policy(TargetPolicy),
    Changed(PathBuf),
    Io {
        operation: TargetOperation,
        path: PathBuf,
        error: io::Error,
    },
}
impl From<ModuleTargetGateError> for ModuleTargetError {
    fn from(error: ModuleTargetGateError) -> Self {
        match error {
            ModuleTargetGateError::Context(error) => Self::Context(error),
            ModuleTargetGateError::EventSuperseded => Self::EventSuperseded,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WatcherCoverageError {
    UnknownCoverage,
}

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub(crate) use unix::PhysicalTargets;

#[cfg(not(unix))]
pub(crate) struct PhysicalTargets;
#[cfg(not(unix))]
impl PhysicalTargets {
    pub(crate) fn observe(_: &Path, _: &Url, _: &[&str]) -> Result<Self, ModuleTargetError> {
        Err(ModuleTargetError::Policy(TargetPolicy::UnsupportedPlatform))
    }
    pub(crate) fn recheck(&self) -> Result<Self, ModuleTargetError> {
        Err(ModuleTargetError::Policy(TargetPolicy::UnsupportedPlatform))
    }
    pub(crate) fn uris(&self) -> &[Url] {
        &[]
    }
}

#[cfg(unix)]
fn candidate(value: &str) -> Result<&str, ModuleTargetError> {
    let Some(relative) = value.strip_prefix("./") else {
        return Err(ModuleTargetError::Policy(TargetPolicy::Candidate));
    };
    if relative.is_empty()
        || relative
            .chars()
            .any(|ch| matches!(ch, '\0' | '\r' | '\n' | '\\' | '?' | '#' | '%'))
        || relative
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(ModuleTargetError::Policy(TargetPolicy::Candidate));
    }
    let extension = Path::new(relative)
        .extension()
        .and_then(|extension| extension.to_str());
    if !matches!(
        extension,
        Some("js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" | "mts" | "cts" | "vue")
    ) {
        return Err(ModuleTargetError::Policy(TargetPolicy::Candidate));
    }
    Ok(relative)
}

#[cfg(all(test, unix))]
mod tests;

#[cfg(all(test, not(unix)))]
#[test]
fn module_link_target_nonunix_backend_has_only_typed_unsupported_platform() {
    let source = Url::parse("file:///original.ts").unwrap();
    assert!(matches!(
        PhysicalTargets::observe(Path::new("/actual-root"), &source, &["./child.ts"]),
        Err(ModuleTargetError::Policy(TargetPolicy::UnsupportedPlatform))
    ));
}

//! An explicitly consumed CLI diagnostics owner, with no declaration/session escape.

use std::path::{Path, PathBuf};

use super::{BatchTypeChecker, CorsaResult, TypeCheckResult, TypeChecker};
use crate::batch::VirtualFile;

mod storage;

/// A terminal diagnostics owner for `vize check`.
///
/// This deliberately has no `Deref`, inner checker accessor, declaration emit,
/// persistent session or semantic query API. Existing `BatchTypeChecker`
/// constructors and declaration/editor consumers keep their ordinary route.
pub struct CliDiagnosticBatch {
    inner: BatchTypeChecker,
    isolated: bool,
}

impl BatchTypeChecker {
    /// Consume an unregistered checker for terminal CLI diagnostics.
    ///
    /// A separate, exclusively claimed storage context is selected before any
    /// virtual path is registered. A previously used checker or unavailable
    /// private storage keeps the old inline route; this marker alone never
    /// authenticates helper bindings, configuration or input completeness.
    pub fn into_cli_diagnostics(mut self) -> CliDiagnosticBatch {
        let storage = if self.scanned || self.owned_storage.is_some() {
            None
        } else {
            storage::claim(&mut self.project).ok().flatten()
        };
        let isolated = storage.is_some();
        if let Some(storage) = storage {
            // Existing field order drops the executor before this owner, so
            // native processes cannot outlive their private materialized files.
            self.owned_storage = Some(storage);
        }
        CliDiagnosticBatch {
            inner: self,
            isolated,
        }
    }
}

impl CliDiagnosticBatch {
    pub fn scan_paths(&mut self, paths: &[PathBuf]) -> CorsaResult<()> {
        self.inner.scan_paths(paths)
    }

    pub fn ensure_included_sources(&mut self) -> CorsaResult<()> {
        self.inner.ensure_included_sources()
    }

    pub fn adopt_diagnostic_sources(
        &mut self,
        paths: impl IntoIterator<Item = PathBuf>,
    ) -> CorsaResult<()> {
        self.inner.adopt_diagnostic_sources(paths)
    }

    pub fn set_diagnostic_paths<'a>(&mut self, paths: impl IntoIterator<Item = &'a Path>) {
        self.inner.set_diagnostic_paths(paths);
    }

    /// Immutable source/report metadata, without a project or native handle.
    pub fn virtual_files(&self) -> Vec<&VirtualFile> {
        self.inner.virtual_files()
    }

    pub fn virtual_root(&self) -> &Path {
        self.inner.virtual_root()
    }

    pub fn shared_helpers_preamble(&self) -> Option<&'static str> {
        self.inner.shared_helpers_preamble()
    }

    pub fn file_count(&self) -> usize {
        self.inner.file_count()
    }

    /// Run diagnostics only. Semantic helper preparation requires a separate
    /// internal witness; storage isolation itself does not enable a transform.
    pub fn check_project(&mut self) -> CorsaResult<TypeCheckResult> {
        let _isolated_owner = self.isolated;
        self.inner.check_project()
    }
}

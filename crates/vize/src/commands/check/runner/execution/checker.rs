//! Select the real output consumer before consuming a terminal diagnostics owner.

use std::path::{Path, PathBuf};
use vize_canon::{
    BatchTypeCheckResult, BatchTypeChecker, BatchTypeCheckerTrait, CliDiagnosticBatch, CorsaError,
    VirtualFile,
};

type Checked<T> = Result<T, CorsaError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::commands::check::runner) enum ProgramPurpose {
    DiagnosticsOnly,
    RetainedProgram,
}

impl ProgramPurpose {
    /// Counter/phase metadata alone does not request generated source. Existing
    /// declaration and VTS show/save/profile consumers keep the old full owner.
    pub(in crate::commands::check::runner) fn from_outputs(
        declaration: bool,
        show_vts: bool,
        save_vts: bool,
        profile_vts: bool,
    ) -> Self {
        if declaration || show_vts || save_vts || profile_vts {
            Self::RetainedProgram
        } else {
            Self::DiagnosticsOnly
        }
    }
}

pub(in crate::commands::check::runner) enum ProgramChecker {
    Diagnostics(CliDiagnosticBatch),
    Retained(BatchTypeChecker),
}

impl ProgramChecker {
    pub(super) fn new(checker: BatchTypeChecker, purpose: ProgramPurpose) -> Self {
        match purpose {
            ProgramPurpose::DiagnosticsOnly => Self::Diagnostics(checker.into_cli_diagnostics()),
            ProgramPurpose::RetainedProgram => Self::Retained(checker),
        }
    }

    pub(super) fn scan_paths(&mut self, paths: &[PathBuf]) -> Checked<()> {
        match self {
            Self::Diagnostics(checker) => checker.scan_paths(paths),
            Self::Retained(checker) => checker.scan_paths(paths),
        }
    }

    pub(super) fn ensure_included_sources(&mut self) -> Checked<()> {
        match self {
            Self::Diagnostics(checker) => checker.ensure_included_sources(),
            Self::Retained(checker) => checker.ensure_included_sources(),
        }
    }

    pub(super) fn adopt_diagnostic_sources(
        &mut self,
        paths: impl IntoIterator<Item = PathBuf>,
    ) -> Checked<()> {
        match self {
            Self::Diagnostics(checker) => checker.adopt_diagnostic_sources(paths),
            Self::Retained(checker) => checker.adopt_diagnostic_sources(paths),
        }
    }

    pub(super) fn set_diagnostic_paths<'a>(&mut self, paths: impl IntoIterator<Item = &'a Path>) {
        match self {
            Self::Diagnostics(checker) => checker.set_diagnostic_paths(paths),
            Self::Retained(checker) => checker.set_diagnostic_paths(paths),
        }
    }

    pub(in crate::commands::check::runner) fn virtual_files(&self) -> Vec<&VirtualFile> {
        match self {
            Self::Diagnostics(checker) => checker.virtual_files(),
            Self::Retained(checker) => checker.virtual_files(),
        }
    }

    pub(in crate::commands::check::runner) fn virtual_root(&self) -> &Path {
        match self {
            Self::Diagnostics(checker) => checker.virtual_root(),
            Self::Retained(checker) => checker.virtual_root(),
        }
    }

    pub(in crate::commands::check::runner) fn shared_helpers_preamble(
        &self,
    ) -> Option<&'static str> {
        match self {
            Self::Diagnostics(checker) => checker.shared_helpers_preamble(),
            Self::Retained(checker) => checker.shared_helpers_preamble(),
        }
    }

    pub(in crate::commands::check::runner) fn file_count(&self) -> usize {
        match self {
            Self::Diagnostics(checker) => checker.file_count(),
            Self::Retained(checker) => checker.file_count(),
        }
    }

    pub(super) fn check_project(&mut self) -> Checked<BatchTypeCheckResult> {
        match self {
            Self::Diagnostics(checker) => checker.check_project(),
            Self::Retained(checker) => checker.check_project(),
        }
    }

    pub(in crate::commands::check::runner) fn declaration_owner(
        &self,
    ) -> Option<&BatchTypeChecker> {
        match self {
            Self::Retained(checker) => Some(checker),
            Self::Diagnostics(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProgramPurpose::{self, DiagnosticsOnly, RetainedProgram};

    #[test]
    fn all_actual_output_requests_select_the_complete_consumer_purpose() {
        let actual = (0..16)
            .map(|bits| {
                let requests = (bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
                (
                    requests,
                    ProgramPurpose::from_outputs(requests.0, requests.1, requests.2, requests.3),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            vec![
                ((false, false, false, false), DiagnosticsOnly),
                ((true, false, false, false), RetainedProgram),
                ((false, true, false, false), RetainedProgram),
                ((true, true, false, false), RetainedProgram),
                ((false, false, true, false), RetainedProgram),
                ((true, false, true, false), RetainedProgram),
                ((false, true, true, false), RetainedProgram),
                ((true, true, true, false), RetainedProgram),
                ((false, false, false, true), RetainedProgram),
                ((true, false, false, true), RetainedProgram),
                ((false, true, false, true), RetainedProgram),
                ((true, true, false, true), RetainedProgram),
                ((false, false, true, true), RetainedProgram),
                ((true, false, true, true), RetainedProgram),
                ((false, true, true, true), RetainedProgram),
                ((true, true, true, true), RetainedProgram),
            ]
        );
    }
}

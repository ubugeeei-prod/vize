//! A sole completed native file supplies its artifact internally.

use super::SsrFacts;
use crate::decision::{
    DecisionBuildError, DecisionTables, NativeAnalysis, build_decisions, policy::TargetPolicy,
};
use vize_l2::{artifact::Artifact, file::FileArtifact};

/// Completed SSR analysis retains its actual immutable file owner.
///
/// A foreign artifact or independent table cannot be attached:
/// ```compile_fail
/// use vize_l3::decision::ssr::NativeSsrFileAnalysis;
/// fn detach(view: NativeSsrFileAnalysis<'_, '_>) { let _ = view.analysis; }
/// ```
/// The owner cannot be discarded while its result is live:
/// ```compile_fail
/// use vize_l2::file::FileArtifact;
/// use vize_l3::decision::ssr::build_ssr_file_decisions;
/// fn discard(file: FileArtifact<'_>) {
///     let analysis = build_ssr_file_decisions(&file).unwrap();
///     drop(file);
///     let _ = analysis.file();
/// }
/// ```
pub struct NativeSsrFileAnalysis<'owner, 'arena> {
    file: &'owner FileArtifact<'arena>,
    analysis: NativeAnalysis<'owner, 'arena>,
}

impl<'owner, 'arena> NativeSsrFileAnalysis<'owner, 'arena> {
    #[must_use]
    pub const fn file(&self) -> &'owner FileArtifact<'arena> {
        self.file
    }
    #[must_use]
    pub fn artifact(&self) -> &'owner Artifact<'arena> {
        self.file.artifact()
    }
    #[must_use]
    pub fn tables(&self) -> &DecisionTables {
        self.analysis.tables()
    }
    #[must_use]
    pub fn ssr(&self) -> Option<&SsrFacts<'owner, 'arena>> {
        self.analysis.ssr()
    }
}

/// Reject incomplete or interrupted file admission before the sole decision walk.
/// This proves native file completion, not original SFC custody or product parity.
pub fn build_ssr_file_decisions<'owner, 'arena>(
    file: &'owner FileArtifact<'arena>,
) -> Result<NativeSsrFileAnalysis<'owner, 'arena>, DecisionBuildError> {
    if !file.is_complete() {
        return Err(DecisionBuildError::IncompleteFile);
    }
    let analysis = build_decisions(file.artifact(), TargetPolicy::Ssr)?;
    Ok(NativeSsrFileAnalysis { file, analysis })
}

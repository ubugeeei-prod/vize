//! A sole completed File supplies its exact artifact to the canonical walk.

use super::VaporFacts;
use crate::decision::{
    DecisionBuildError, DecisionTables, NativeAnalysis, build_decisions, policy::TargetPolicy,
};
use vize_l2::{artifact::Artifact, file::FileArtifact};

/// Target input retains its original completed immutable File owner.
///
/// A caller cannot substitute an independently supplied analysis:
/// ```compile_fail
/// use vize_l3::decision::vapor::NativeVaporFileAnalysis;
/// fn detach(view: NativeVaporFileAnalysis<'_, '_>) { let _ = view.analysis; }
/// ```
/// Original ownership remains live through the borrowed output:
/// ```compile_fail
/// use vize_l2::file::FileArtifact;
/// use vize_l3::decision::vapor::build_vapor_file_decisions;
/// fn discard(file: FileArtifact<'_>) {
///     let analysis = build_vapor_file_decisions(&file).unwrap();
///     drop(file);
///     let _ = analysis.file();
/// }
/// ```
pub struct NativeVaporFileAnalysis<'owner, 'arena> {
    file: &'owner FileArtifact<'arena>,
    analysis: NativeAnalysis<'owner, 'arena>,
}

impl<'owner, 'arena> NativeVaporFileAnalysis<'owner, 'arena> {
    #[must_use]
    pub const fn file(&self) -> &'owner FileArtifact<'arena> {
        self.file
    }
    #[must_use]
    pub fn original_attributes(
        &self,
    ) -> Option<&crate::decision::OriginalAttributeFacts<'owner, 'arena>> {
        self.analysis.original_attributes()
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
    pub fn vapor(&self) -> Option<&VaporFacts<'owner, 'arena>> {
        self.analysis.vapor()
    }
}

/// Incomplete original file admission fails before any decision traversal.
/// This is completed File admission, not whole SFC custody or product parity.
pub fn build_vapor_file_decisions<'owner, 'arena>(
    file: &'owner FileArtifact<'arena>,
) -> Result<NativeVaporFileAnalysis<'owner, 'arena>, DecisionBuildError> {
    if !file.is_complete() {
        return Err(DecisionBuildError::IncompleteFile);
    }
    let analysis = build_decisions(file.artifact(), TargetPolicy::Vapor)?;
    Ok(NativeVaporFileAnalysis { file, analysis })
}

// Called only after the real selected template entry derives its own File.
pub(in crate::decision) fn build_original_file_decisions<'owner, 'arena>(
    file: &'owner FileArtifact<'arena>,
) -> Result<NativeVaporFileAnalysis<'owner, 'arena>, DecisionBuildError> {
    let analysis = crate::decision::build::build_with_original(
        file,
        TargetPolicy::Vapor,
        &crate::decision::dom::LiteralExpressions,
        &crate::decision::dom::vue::policy::NoReads,
    )?;
    Ok(NativeVaporFileAnalysis { file, analysis })
}

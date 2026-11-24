//! A sole completed File supplies its exact artifact to the canonical walk.

use super::VaporFacts;
use crate::decision::{
    DecisionAnalysis, DecisionBuildError, DecisionTables, build_decisions, policy::TargetPolicy,
};
use vize_l2::{artifact::Artifact, file::L2Ir};

/// Target input retains its original completed immutable File owner.
///
/// A caller cannot substitute an independently supplied analysis:
/// ```compile_fail
/// use vize_l3::decision::vapor::VaporFileAnalysis;
/// fn detach(view: VaporFileAnalysis<'_, '_>) { let _ = view.analysis; }
/// ```
/// Original ownership remains live through the borrowed output:
/// ```compile_fail
/// use vize_l2::file::L2Ir;
/// use vize_l3::decision::vapor::build_vapor_file_decisions;
/// fn discard(file: L2Ir<'_>) {
///     let analysis = build_vapor_file_decisions(&file).unwrap();
///     drop(file);
///     let _ = analysis.file();
/// }
/// ```
pub struct VaporFileAnalysis<'owner, 'arena> {
    file: &'owner L2Ir<'arena>,
    analysis: DecisionAnalysis<'owner, 'arena>,
}

impl<'owner, 'arena> VaporFileAnalysis<'owner, 'arena> {
    #[must_use]
    pub const fn file(&self) -> &'owner L2Ir<'arena> {
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
    file: &'owner L2Ir<'arena>,
) -> Result<VaporFileAnalysis<'owner, 'arena>, DecisionBuildError> {
    if !file.is_complete() {
        return Err(DecisionBuildError::IncompleteFile);
    }
    let analysis = build_decisions(file.artifact(), TargetPolicy::Vapor)?;
    Ok(VaporFileAnalysis { file, analysis })
}

// Called only after the real selected template entry derives its own File.
pub(in crate::decision) fn build_original_file_decisions<'owner, 'arena>(
    file: &'owner L2Ir<'arena>,
) -> Result<VaporFileAnalysis<'owner, 'arena>, DecisionBuildError> {
    let analysis = crate::decision::build::build_with_original(
        file,
        TargetPolicy::Vapor,
        &crate::decision::dom::LiteralExpressions,
        &crate::decision::dom::vue::policy::NoReads,
    )?;
    Ok(VaporFileAnalysis { file, analysis })
}

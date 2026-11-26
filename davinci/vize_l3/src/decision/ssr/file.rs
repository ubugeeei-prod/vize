//! A sole completed native file supplies its artifact internally.

use super::SsrAnalysis;
use crate::decision::{
    DecisionAnalysis, DecisionBuildError, DecisionTables, build_decisions, policy::TargetPolicy,
};
use vize_l2::{artifact::L2Template, file::L2Ir};

/// Completed SSR analysis retains its actual immutable file owner.
///
/// A foreign artifact or independent table cannot be attached:
/// ```compile_fail
/// use vize_l3::decision::ssr::SsrL2IrAnalysis;
/// fn detach(view: SsrL2IrAnalysis<'_, '_>) { let _ = view.analysis; }
/// ```
/// The owner cannot be discarded while its result is live:
/// ```compile_fail
/// use vize_l2::file::L2Ir;
/// use vize_l3::decision::ssr::build_ssr_file_decisions;
/// fn discard(l2ir: L2Ir<'_>) {
///     let analysis = build_ssr_file_decisions(&l2ir).unwrap();
///     drop(l2ir);
///     let _ = analysis.l2ir();
/// }
/// ```
pub struct SsrL2IrAnalysis<'owner, 'arena> {
    l2ir: &'owner L2Ir<'arena>,
    analysis: DecisionAnalysis<'owner, 'arena>,
}

impl<'owner, 'arena> SsrL2IrAnalysis<'owner, 'arena> {
    #[must_use]
    pub const fn l2ir(&self) -> &'owner L2Ir<'arena> {
        self.l2ir
    }
    #[must_use]
    pub fn original_attributes(
        &self,
    ) -> Option<&crate::decision::OriginalAttributeAnalysis<'owner, 'arena>> {
        self.analysis.original_attributes()
    }

    #[must_use]
    pub fn artifact(&self) -> &'owner L2Template<'arena> {
        self.l2ir.artifact()
    }
    #[must_use]
    pub fn tables(&self) -> &DecisionTables {
        self.analysis.tables()
    }
    #[must_use]
    pub fn ssr(&self) -> Option<&SsrAnalysis<'owner, 'arena>> {
        self.analysis.ssr()
    }
}

/// Reject incomplete or interrupted file admission before the sole decision walk.
/// This proves native file completion, not original SFC custody or product parity.
pub fn build_ssr_file_decisions<'owner, 'arena>(
    l2ir: &'owner L2Ir<'arena>,
) -> Result<SsrL2IrAnalysis<'owner, 'arena>, DecisionBuildError> {
    if !l2ir.is_complete() {
        return Err(DecisionBuildError::IncompleteFile);
    }
    let analysis = build_decisions(l2ir.artifact(), TargetPolicy::Ssr)?;
    Ok(SsrL2IrAnalysis { l2ir, analysis })
}

// Called only after the real selected template entry derives its own File.
pub(in crate::decision) fn build_original_file_decisions<'owner, 'arena>(
    l2ir: &'owner L2Ir<'arena>,
) -> Result<SsrL2IrAnalysis<'owner, 'arena>, DecisionBuildError> {
    let analysis = crate::decision::build::build_with_original(
        l2ir,
        TargetPolicy::Ssr,
        &crate::decision::dom::LiteralExpressions,
        &crate::decision::dom::vue::policy::NoReads,
    )?;
    Ok(SsrL2IrAnalysis { l2ir, analysis })
}

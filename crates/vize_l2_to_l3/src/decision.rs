//! Native L2-to-L3 decision producer boundary (issue #6839).
//!
//! The input is borrowed directly; no folio or other serialized artifact
//! crosses this boundary. The current [`crate::lower`] is not routed here.

use vize_l2::op::Region;
use vize_l3::decision::{DecisionTables, policy::TargetPolicy};

/// Compute shared decision tables without constructing a flat L3 program.
///
/// The future producer replaces the existing DOM hoist analysis and uses
/// the owning L2 artifact's page-order node ids. It must preserve authored
/// dynamic-binding order and control containment, apply the selected target
/// criteria, and pass the old-analysis and byte-output comparisons plus the
/// unchanged instruction-count gate before any product selects it.
///
/// # Panics
///
/// This deliberately unfinished skeleton always panics with a tracked TODO.
/// It is never called by a compiler, checker, linter, formatter, or LSP.
#[expect(
    clippy::todo,
    reason = "issue #6839: maintainer requested an explicit native producer skeleton before implementation"
)]
pub fn build_decisions(_root: &Region<'_>, _policy: TargetPolicy) -> DecisionTables {
    todo!("#6839: compute L2-node decisions; keep flat-program construction on demand")
}

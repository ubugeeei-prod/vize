//! Optional S3 passes, run where the exported partition facts live.

use vize_s3::extract::{Extraction, OptTier, extract};
use vize_s3::placement::annotate;

use crate::Lowered;

/// Record the canonical placement alternatives, then choose among them with
/// try-measure-commit extraction at `tier`.
///
/// Both passes write only `program.placements`, never an op, region, edge,
/// effect scope, or operand, so the partition export keeps describing the
/// canonical program: `lowered.partition.stale(&lowered.program)` stays
/// `None`. TS-17 snapshots and the TS-20 fuzz target check that after every
/// run instead of revalidating facts that cannot have moved.
pub fn optimize(lowered: &mut Lowered<'_>, tier: OptTier) -> Extraction {
    annotate(&mut lowered.program);
    extract(&mut lowered.program, tier)
}

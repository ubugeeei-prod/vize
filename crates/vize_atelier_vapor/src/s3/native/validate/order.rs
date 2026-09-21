//! Source order must agree with the explicit S3 ordering edges exactly.

use vize_carton::{FxHashMap, FxHashSet};
use vize_s3::op::{EdgeKind, OpId, Program, RegionId};

use super::Result;
use crate::s3::AdmissionFailure;

pub(super) fn check(
    program: &Program<'_>,
    regions: &FxHashMap<RegionId, std::vec::Vec<OpId>>,
    bindings: &FxHashMap<OpId, std::vec::Vec<OpId>>,
) -> Result<()> {
    let mut expected = FxHashSet::default();
    for ops in regions.values().chain(bindings.values()) {
        expected.extend(
            ops.windows(2)
                .map(|pair| (pair[0], pair[1], EdgeKind::DomOrder)),
        );
    }
    let dynamic: std::vec::Vec<_> = program
        .ops
        .iter()
        .filter(|op| op.effect.is_some())
        .collect();
    expected.extend(
        dynamic
            .windows(2)
            .map(|pair| (pair[0].id, pair[1].id, EdgeKind::EffectOrder)),
    );
    // Consume each obligation once: repeating an edge cannot conceal a missing
    // dependency, even when the total edge counts are identical.
    if program
        .edges
        .iter()
        .any(|edge| !expected.remove(&(edge.from, edge.to, edge.kind)))
        || !expected.is_empty()
    {
        return Err(AdmissionFailure::Invalid(
            "native order differs from S3 ordering edges",
        ));
    }
    Ok(())
}

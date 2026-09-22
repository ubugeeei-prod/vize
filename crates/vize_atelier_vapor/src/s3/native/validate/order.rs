//! Source order must agree with the explicit S3 ordering edges exactly.

use vize_carton::{Allocator, Vec};
use vize_s3::op::{EdgeKind, OpId, Program};

use super::Result;
use crate::s3::AdmissionFailure;

/// The ordering obligations native assembly derives, as `(from, to, kind)`.
pub(super) type Edge = (u32, u32, u8);

pub(super) fn edge(from: OpId, to: OpId, kind: EdgeKind) -> Edge {
    (from.index(), to.index(), kind as u8)
}

/// The program's edges must be exactly the derived obligations: sibling order
/// within each region, binding order per target, and effect order. Comparing
/// sorted multisets consumes each obligation once, so repeating an edge
/// cannot conceal a missing dependency even when the counts agree.
pub(super) fn check<'a>(
    program: &Program<'_>,
    mut expected: Vec<'a, Edge>,
    alloc: &'a Allocator,
) -> Result<()> {
    let mut actual = Vec::from_iter_in(
        (program.edges.iter()).map(|state| edge(state.from, state.to, state.kind)),
        &alloc,
    );
    expected.sort_unstable();
    actual.sort_unstable();
    if actual != expected {
        return Err(AdmissionFailure::Invalid(
            "native order differs from S3 ordering edges",
        ));
    }
    Ok(())
}

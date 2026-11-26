//! Source order must agree with the explicit L3 ordering edges exactly.

use vize_carton::{Allocator, Vec};
use vize_l3::op::{EdgeKind, L3Ir, OpId};

use super::Result;
use crate::l3::AdmissionFailure;

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
    l3ir: &L3Ir<'_>,
    mut expected: Vec<'a, Edge>,
    alloc: &'a Allocator,
) -> Result<()> {
    let mut actual = Vec::from_iter_in(
        (l3ir.edges.iter()).map(|state| edge(state.from, state.to, state.kind)),
        &alloc,
    );
    expected.sort_unstable();
    actual.sort_unstable();
    if actual != expected {
        return Err(AdmissionFailure::Invalid(
            "native order differs from L3 ordering edges",
        ));
    }
    Ok(())
}

//! Node identity: the key every side table and cross-stage reference uses.
//!
//! `architecture.md`'s shared-infrastructure contract is "cross-stage
//! references and analysis results are `NodeId`-keyed tables, not fat nodes
//! and not raw `*mut` traversal". [`NodeId`] is that key.

use core::fmt;
use core::num::NonZeroU32;

/// A stage-local node identity.
///
/// # Why `NonZeroU32` rather than a reserved sentinel
///
/// Both give `Option<NodeId>` a 4-byte layout, which is the requirement (an
/// S2 op holding several optional child references pays 4 bytes per slot, not
/// 8). `NonZeroU32` is chosen because it makes "no such node" **unrepresentable
/// inside `NodeId` itself**: with a reserved-sentinel `u32` every consumer must
/// remember that `NodeId(0)` is not a node, and nothing stops one from
/// constructing it. The niche is also stable-Rust: reserving a range on a plain
/// `u32` needs `rustc_layout_scalar_valid_range`, which is unstable.
///
/// The cost is that the raw value and the index differ by one, so the
/// conversion is explicit and single-sourced: [`NodeId::from_index`] and
/// [`NodeId::index`]. Nothing else may do the arithmetic.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(NonZeroU32);

/// `NodeId` is a key, copied into every side-table lookup; a niche keeps
/// `Option<NodeId>` the same width as the id.
///
/// Deliberately **not** `#[cfg(target_pointer_width = "64")]`-guarded, unlike
/// the node-size asserts in `vize_relief`. That guard exists because those
/// figures are 64-bit footprints of pointer-containing structs ("the wasm32
/// build is 32-bit", `crates/vize_relief/src/relief/elements.rs:31-36`).
/// `NodeId` holds a `u32` and no pointer, so its footprint is the same on
/// every target and guarding would only stop the wasm32 lane P2-14 makes
/// required from checking the niche - which is the property most worth
/// checking there.
const _: () = assert!(size_of::<NodeId>() == 4);
const _: () = assert!(size_of::<Option<NodeId>>() == 4);

impl NodeId {
    /// The first id a stage mints, equivalent to `from_index(0)`.
    pub const FIRST: NodeId = NodeId(NonZeroU32::MIN);

    /// The id for a 0-based node index.
    ///
    /// Returns `None` only for `u32::MAX`, whose successor does not fit — a
    /// stage with more than `u32::MAX - 1` nodes in one artifact, which no
    /// real input reaches. Callers minting ids in sequence should propagate
    /// the `None` rather than unwrap it: exhaustion is a diagnostic, not a
    /// panic.
    #[inline]
    #[must_use]
    pub const fn from_index(index: u32) -> Option<NodeId> {
        match NonZeroU32::new(index.wrapping_add(1)) {
            Some(raw) => Some(NodeId(raw)),
            None => None,
        }
    }

    /// The 0-based index this id denotes.
    ///
    /// This is the dense-side-table subscript, and the only place the
    /// off-by-one between the niche representation and the index lives.
    #[inline]
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0.get() - 1
    }

    /// The 0-based index as a `usize`, for indexing a dense table.
    #[inline]
    #[must_use]
    pub const fn index_usize(self) -> usize {
        self.index() as usize
    }

    /// The niche representation, for a folio page that serializes ids.
    #[inline]
    #[must_use]
    pub const fn to_raw(self) -> NonZeroU32 {
        self.0
    }

    /// Rebuild an id from its niche representation.
    #[inline]
    #[must_use]
    pub const fn from_raw(raw: NonZeroU32) -> NodeId {
        NodeId(raw)
    }

    /// The next id in sequence, or `None` at exhaustion.
    #[inline]
    #[must_use]
    pub const fn next(self) -> Option<NodeId> {
        match self.0.checked_add(1) {
            Some(raw) => Some(NodeId(raw)),
            None => None,
        }
    }
}

/// Prints the **index**, not the niche value, because the index is what folio
/// pages and diagnostics show: `NodeId(0)` is the first node.
impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({})", self.index())
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.index())
    }
}

#[cfg(test)]
mod tests {
    use super::NodeId;
    use vize_s0::cstr;

    #[test]
    fn index_round_trips_through_the_niche() {
        for index in [0u32, 1, 2, 41, u32::MAX - 1] {
            let id = NodeId::from_index(index).expect("index below u32::MAX has an id");
            assert_eq!(id.index(), index);
            assert_eq!(id.index_usize(), index as usize);
            assert_eq!(NodeId::from_raw(id.to_raw()), id);
        }
    }

    #[test]
    fn the_first_id_is_index_zero() {
        assert_eq!(
            NodeId::FIRST,
            NodeId::from_index(0).expect("index 0 has an id")
        );
        assert_eq!(NodeId::FIRST.index(), 0);
    }

    #[test]
    fn exhaustion_yields_none_rather_than_wrapping() {
        assert_eq!(NodeId::from_index(u32::MAX), None);
        let last = NodeId::from_raw(core::num::NonZeroU32::MAX);
        assert_eq!(last.next(), None);
    }

    #[test]
    fn next_walks_the_index_sequence() {
        let first = NodeId::FIRST;
        let second = first.next().expect("the second id exists");
        assert_eq!(second.index(), 1);
        assert_eq!(second, NodeId::from_index(1).expect("index 1 has an id"));
    }

    #[test]
    fn debug_and_display_show_the_index() {
        let id = NodeId::from_index(7).expect("index 7 has an id");
        assert_eq!(cstr!("{id:?}").as_str(), "NodeId(7)");
        assert_eq!(cstr!("{id}").as_str(), "%7");
    }

    #[test]
    fn ids_order_by_index() {
        let a = NodeId::from_index(1).expect("index 1 has an id");
        let b = NodeId::from_index(2).expect("index 2 has an id");
        assert!(a < b);
    }
}

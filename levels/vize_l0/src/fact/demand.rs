//! [`Demand`] — a set of fact groups, built in `const` context.

use crate::pass::{AnalysisId, Preserved};

/// A set of fact groups, keyed by their [`AnalysisId`].
///
/// The same 64-bit shape as [`Preserved`], because both name members of one
/// identity space: a consumer's demand is "what I read", a pass's preserved
/// set is "what I leave valid". Built only through `const fn`s so every
/// consumer's demand is a `const` item the detector and the stratification
/// check can read.
///
/// ```
/// use vize_l0::fact::Demand;
/// use vize_l0::pass::AnalysisId;
///
/// const BINDINGS: AnalysisId = AnalysisId::new(3);
/// const SCOPES: AnalysisId = AnalysisId::new(4);
/// const DEMAND: Demand = Demand::NONE.with(BINDINGS).with(SCOPES);
/// const _: () = assert!(DEMAND.contains(SCOPES) && DEMAND.len() == 2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Demand(u64);

impl Demand {
    /// Reads nothing.
    pub const NONE: Demand = Demand(0);

    /// This set plus `group`.
    #[inline]
    #[must_use]
    pub const fn with(self, group: AnalysisId) -> Demand {
        Demand(self.0 | bit(group))
    }

    /// Whether `group` is in the set.
    #[inline]
    #[must_use]
    pub const fn contains(self, group: AnalysisId) -> bool {
        self.0 & bit(group) != 0
    }

    /// Every group in either set.
    #[inline]
    #[must_use]
    pub const fn union(self, other: Demand) -> Demand {
        Demand(self.0 | other.0)
    }

    /// The groups in both sets.
    #[inline]
    #[must_use]
    pub const fn intersect(self, other: Demand) -> Demand {
        Demand(self.0 & other.0)
    }

    /// This set minus every group in `other`.
    #[inline]
    #[must_use]
    pub const fn minus(self, other: Demand) -> Demand {
        Demand(self.0 & !other.0)
    }

    /// Whether every group in this set is also in `other`.
    #[inline]
    #[must_use]
    pub const fn is_subset_of(self, other: Demand) -> bool {
        self.0 & !other.0 == 0
    }

    /// Whether the set is empty.
    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// How many groups the set holds.
    #[inline]
    #[must_use]
    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }

    /// The groups a pass preserving `preserved` leaves valid out of this set.
    #[inline]
    #[must_use]
    pub const fn surviving(self, preserved: Preserved) -> Demand {
        Demand(self.0 & preserved.to_bits())
    }

    /// The raw mask, for a folio page.
    #[inline]
    #[must_use]
    pub const fn to_bits(self) -> u64 {
        self.0
    }

    /// Rebuild a set from its raw mask.
    #[inline]
    #[must_use]
    pub const fn from_bits(bits: u64) -> Demand {
        Demand(bits)
    }

    /// The members in ascending id order.
    pub fn iter(self) -> impl Iterator<Item = AnalysisId> {
        let mut remaining = self.0;
        core::iter::from_fn(move || {
            if remaining == 0 {
                return None;
            }
            // A nonzero u64's lowest set bit is an admitted analysis (0..64).
            let index = remaining.trailing_zeros() as u8;
            remaining &= remaining - 1;
            Some(AnalysisId::new(index))
        })
    }
}

#[inline]
const fn bit(group: AnalysisId) -> u64 {
    1u64 << group.index()
}

#[cfg(test)]
mod tests {
    use super::Demand;
    use crate::pass::{AnalysisId, MAX_ANALYSES, Preserved};
    use alloc::vec::Vec;

    const A: AnalysisId = AnalysisId::new(0);
    const B: AnalysisId = AnalysisId::new(7);
    const C: AnalysisId = AnalysisId::new(63);

    #[test]
    fn set_algebra_is_exact() {
        let ab = Demand::NONE.with(A).with(B);
        let bc = Demand::NONE.with(B).with(C);
        assert_eq!(ab.union(bc), Demand::NONE.with(A).with(B).with(C));
        assert_eq!(ab.intersect(bc), Demand::NONE.with(B));
        assert_eq!(ab.minus(bc), Demand::NONE.with(A));
        assert!(Demand::NONE.with(B).is_subset_of(ab));
        assert!(!bc.is_subset_of(ab));
        assert_eq!((ab.len(), Demand::NONE.len()), (2, 0));
        assert!(Demand::NONE.is_empty() && !ab.is_empty());
    }

    #[test]
    fn iteration_is_ascending_and_complete() {
        let set = Demand::NONE.with(C).with(A).with(B);
        let ids: Vec<u8> = set.iter().map(AnalysisId::index).collect();
        assert_eq!(ids, [0, 7, 63]);
    }

    #[test]
    fn iteration_matches_the_complete_identity_space_for_dense_and_sparse_masks() {
        let mut masks = Vec::from([0, u64::MAX, 0xaaaa_aaaa_aaaa_aaaa, 0x5555_5555_5555_5555]);
        masks.extend((0..MAX_ANALYSES).map(|index| 1u64 << index));
        let mut mask = 1u64;
        for _ in 0..1024 {
            mask = mask.wrapping_mul(6364136223846793005).wrapping_add(1);
            masks.push(mask);
        }
        for mask in masks {
            let demand = Demand::from_bits(mask);
            let actual: Vec<u8> = demand.iter().map(AnalysisId::index).collect();
            let expected: Vec<u8> = (0..MAX_ANALYSES)
                .filter(|index| demand.contains(AnalysisId::new(*index)))
                .collect();
            assert_eq!(actual, expected, "mask: {mask:#018x}");
        }
    }

    #[test]
    fn exhausted_iteration_stays_empty_including_the_highest_identity() {
        let mut empty = Demand::NONE.iter();
        assert_eq!((empty.next(), empty.next()), (None, None));
        let mut highest = Demand::NONE.with(C).iter();
        assert_eq!(highest.next(), Some(C));
        assert_eq!((highest.next(), highest.next()), (None, None));
    }

    #[test]
    fn surviving_reads_a_preserved_mask_in_the_same_identity_space() {
        let set = Demand::NONE.with(A).with(B);
        assert_eq!(set.surviving(Preserved::NONE.with(B)), Demand::NONE.with(B));
        assert_eq!(set.surviving(Preserved::ALL), set);
        assert_eq!(set.surviving(Preserved::NONE), Demand::NONE);
    }

    #[test]
    fn bits_round_trip() {
        let set = Demand::NONE.with(A).with(C);
        assert_eq!(Demand::from_bits(set.to_bits()), set);
        assert_eq!(set.to_bits(), 1 | (1 << 63));
    }
}

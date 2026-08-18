//! Analysis identities and the preserved sets fusion intersects.

/// The identity of one analysis whose results a pass may invalidate.
///
/// Analyses are named by their owner — the crate that computes them declares
/// its ids — because the pass manager only needs to compare them, never to
/// interpret them. [`AnalysisId::new`] rejects an out-of-range id at compile
/// time when called in a `const` context, which is the only way ids are meant
/// to be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnalysisId(u8);

/// How many distinct analyses one [`Preserved`] set can track.
///
/// A `u64` bitmask, which is 8 bytes in every `PassDesc` and a single `and`
/// per fusion step. Raising it means changing the mask type, and the const
/// assertion below is what makes that a compile error rather than a silent
/// truncation.
pub const MAX_ANALYSES: u8 = 64;

const _: () = assert!(MAX_ANALYSES as u32 <= u64::BITS);

impl AnalysisId {
    /// The id numbered `index`.
    ///
    /// # Panics
    ///
    /// Panics if `index >= MAX_ANALYSES`. In a `const` context — how ids are
    /// meant to be declared — that panic is a compile error.
    #[inline]
    #[must_use]
    pub const fn new(index: u8) -> Self {
        assert!(
            index < MAX_ANALYSES,
            "analysis id is out of range: a Preserved set tracks MAX_ANALYSES analyses"
        );
        Self(index)
    }

    /// The bit index this analysis occupies in a [`Preserved`] mask.
    #[inline]
    #[must_use]
    pub const fn index(self) -> u8 {
        self.0
    }

    /// This id as a one-bit mask.
    #[inline]
    #[must_use]
    const fn bit(self) -> u64 {
        1u64 << self.0
    }
}

/// The set of analyses a pass leaves valid.
///
/// A pass declares what it **preserves**, not what it invalidates, because the
/// safe default has to be the conservative one: a pass that says nothing
/// preserves nothing, so forgetting to update a declaration when a pass starts
/// mutating more costs a recomputation, never a stale fact.
///
/// Fusion intersects the preserved sets of its members ([`Preserved::intersect`]):
/// a group preserves exactly what every pass in it preserves. That intersection
/// runs in a `const fn`, so a grouping regression is a compile error in a
/// `const`-evaluated pin rather than a runtime surprise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Preserved(u64);

impl Preserved {
    /// Preserves nothing. The default for a pass that has not declared.
    pub const NONE: Preserved = Preserved(0);

    /// Preserves every analysis. Correct only for a pass that does not mutate.
    pub const ALL: Preserved = Preserved(u64::MAX);

    /// This set plus `analysis`.
    #[inline]
    #[must_use]
    pub const fn with(self, analysis: AnalysisId) -> Preserved {
        Preserved(self.0 | analysis.bit())
    }

    /// This set minus `analysis`.
    #[inline]
    #[must_use]
    pub const fn without(self, analysis: AnalysisId) -> Preserved {
        Preserved(self.0 & !analysis.bit())
    }

    /// Whether `analysis` survives this pass.
    ///
    /// Named `preserves` rather than `contains` because the set is named for
    /// what it keeps: `set.preserves(SCOPES)` is the question callers actually
    /// ask, and a bare `contains` reads as a value search over a collection.
    #[inline]
    #[must_use]
    pub const fn preserves(self, analysis: AnalysisId) -> bool {
        self.0 & analysis.bit() != 0
    }

    /// The analyses both sets preserve — what a fused group preserves.
    #[inline]
    #[must_use]
    pub const fn intersect(self, other: Preserved) -> Preserved {
        Preserved(self.0 & other.0)
    }

    /// Whether nothing survives.
    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The raw mask, for a folio page that serializes a plan.
    #[inline]
    #[must_use]
    pub const fn to_bits(self) -> u64 {
        self.0
    }

    /// Rebuild a set from its raw mask.
    #[inline]
    #[must_use]
    pub const fn from_bits(bits: u64) -> Preserved {
        Preserved(bits)
    }
}

impl Default for Preserved {
    /// [`Preserved::NONE`] — the conservative default, per the type docs.
    fn default() -> Self {
        Preserved::NONE
    }
}

#[cfg(test)]
mod tests {
    use super::{AnalysisId, Preserved};

    const SCOPES: AnalysisId = AnalysisId::new(0);
    const CONSTNESS: AnalysisId = AnalysisId::new(1);
    const SLOTS: AnalysisId = AnalysisId::new(63);

    #[test]
    fn the_empty_set_preserves_nothing() {
        assert!(Preserved::NONE.is_empty());
        assert!(!Preserved::NONE.preserves(SCOPES));
    }

    #[test]
    fn the_full_set_preserves_every_declared_analysis() {
        assert!(Preserved::ALL.preserves(SCOPES));
        assert!(Preserved::ALL.preserves(CONSTNESS));
        assert!(Preserved::ALL.preserves(SLOTS));
        assert!(!Preserved::ALL.is_empty());
    }

    #[test]
    fn with_and_without_move_exactly_one_analysis() {
        let set = Preserved::NONE.with(SCOPES).with(SLOTS);
        assert!(set.preserves(SCOPES));
        assert!(!set.preserves(CONSTNESS));
        assert!(set.preserves(SLOTS));

        let narrowed = set.without(SLOTS);
        assert!(narrowed.preserves(SCOPES));
        assert!(!narrowed.preserves(SLOTS));
    }

    #[test]
    fn intersect_keeps_only_what_both_sides_preserve() {
        let left = Preserved::NONE.with(SCOPES).with(CONSTNESS);
        let right = Preserved::NONE.with(CONSTNESS).with(SLOTS);
        let both = left.intersect(right);
        assert!(!both.preserves(SCOPES));
        assert!(both.preserves(CONSTNESS));
        assert!(!both.preserves(SLOTS));
    }

    #[test]
    fn intersecting_with_nothing_preserves_nothing() {
        let set = Preserved::ALL.intersect(Preserved::NONE);
        assert!(set.is_empty());
    }

    #[test]
    fn bits_round_trip() {
        let set = Preserved::NONE.with(SCOPES).with(SLOTS);
        assert_eq!(Preserved::from_bits(set.to_bits()), set);
    }

    #[test]
    fn the_default_set_is_the_conservative_one() {
        assert_eq!(Preserved::default(), Preserved::NONE);
    }

    /// The intersection runs in a `const fn`, which is what lets a pinned
    /// fusion plan be `const`-evaluated. Checked here in a `const` item so a
    /// regression is a compile error.
    #[test]
    fn intersection_is_const_evaluable() {
        const LEFT: Preserved = Preserved::NONE.with(SCOPES).with(CONSTNESS);
        const RIGHT: Preserved = Preserved::NONE.with(CONSTNESS);
        const BOTH: Preserved = LEFT.intersect(RIGHT);
        const _: () = assert!(BOTH.preserves(CONSTNESS));
        const _: () = assert!(!BOTH.preserves(SCOPES));
        assert_eq!(BOTH, RIGHT);
    }
}

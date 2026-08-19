//! Pass classification and fusability.

/// What a pass is for, and therefore when it runs.
///
/// Imported from SIL's mandatory/optional split. The distinction is not a
/// priority hint: it decides three things at once, and the three are not
/// separable.
///
/// 1. **Mandatory passes run at every optimization level.** A tier that
///    skipped one would change program meaning or lose a diagnostic, not just
///    lose speed.
/// 2. **Mandatory passes are unfusable barriers.** Enforced at construction —
///    see [`super::PassDesc::new`] — because a mandatory pass either needs
///    whole-tree facts or establishes an invariant the next pass reads, and
///    both are exactly what fusion breaks.
/// 3. **Only a mandatory pass performs the raw → canonical transition.** That
///    is enforced at the type level rather than by review; see
///    [`super::Canonical`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PassKind {
    /// Runs at every level to produce diagnostics the user must see.
    ///
    /// Skipping it silently loses an error, which is why it is not optional
    /// even at the fastest tier.
    MandatoryDiagnostic,
    /// Runs at every level to establish an invariant later stages assume.
    ///
    /// This is the kind that canonicalizes.
    MandatoryLowering,
    /// Improves the output and may be skipped.
    ///
    /// An optional pass may not change what the program means, only how well
    /// it is expressed — so a tier that drops it stays correct.
    Optional,
}

impl PassKind {
    /// Whether this kind runs at every optimization level.
    #[inline]
    #[must_use]
    pub const fn is_mandatory(self) -> bool {
        matches!(
            self,
            PassKind::MandatoryDiagnostic | PassKind::MandatoryLowering
        )
    }

    /// Stable identifier for folio pages and pipeline dumps.
    #[inline]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            PassKind::MandatoryDiagnostic => "mandatory-diagnostic",
            PassKind::MandatoryLowering => "mandatory-lowering",
            PassKind::Optional => "optional",
        }
    }
}

/// Whether a pass can share a walk with its neighbours.
///
/// The pass manager fuses adjacent [`Fusability::Fusable`] passes into one
/// traversal (`architecture.md`: "passes declare themselves fusable
/// (single-visit, local, synthesized-attribute style) or barrier (needs
/// whole-tree or fixpoint facts), and the pass manager fuses adjacent fusable
/// passes into one walk").
///
/// Declaring `Fusable` is a claim about the pass, not a request: it says the
/// pass visits each node once, reads only that node and facts already computed
/// for it, and never looks at a sibling it has not reached. A pass that needs
/// lookahead — sibling `v-else` collection, slot gathering — is a
/// [`Fusability::Barrier`] and says so.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Fusability {
    /// Single-visit and local: may share a walk with its neighbours.
    Fusable,
    /// Needs whole-tree or fixpoint facts: owns its walk.
    Barrier,
}

impl Fusability {
    /// Whether this pass may share a walk.
    #[inline]
    #[must_use]
    pub const fn is_fusable(self) -> bool {
        matches!(self, Fusability::Fusable)
    }

    /// Stable identifier for folio pages and pipeline dumps.
    #[inline]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Fusability::Fusable => "fusable",
            Fusability::Barrier => "barrier",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Fusability, PassKind};

    #[test]
    fn both_mandatory_kinds_report_mandatory_and_optional_does_not() {
        assert!(PassKind::MandatoryDiagnostic.is_mandatory());
        assert!(PassKind::MandatoryLowering.is_mandatory());
        assert!(!PassKind::Optional.is_mandatory());
    }

    #[test]
    fn kind_identifiers_are_stable_strings() {
        assert_eq!(
            PassKind::MandatoryDiagnostic.as_str(),
            "mandatory-diagnostic"
        );
        assert_eq!(PassKind::MandatoryLowering.as_str(), "mandatory-lowering");
        assert_eq!(PassKind::Optional.as_str(), "optional");
    }

    #[test]
    fn fusability_identifiers_are_stable_strings() {
        assert!(Fusability::Fusable.is_fusable());
        assert!(!Fusability::Barrier.is_fusable());
        assert_eq!(Fusability::Fusable.as_str(), "fusable");
        assert_eq!(Fusability::Barrier.as_str(), "barrier");
    }
}

//! Demand-shaped Croquis semantic facts.
//!
//! Croquis is the semantic study tools *ask for*, not a pipeline checkpoint
//! every consumer must finish. These types name the individual facts a lane can
//! request so Patina, Canon, and the Ateliers each pay only for what they need
//! — e.g. a lint rule asks for bindings and directives without forcing scope or
//! dependency-edge collection. This is the atlas-side request shape; it stays
//! lighter than a query engine and never builds render semantics.

use super::PlateFamily;

/// A single semantic fact a lane can demand from Croquis.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CroquisFact {
    /// Setup/template binding metadata.
    Bindings,
    /// Lexical scopes and their relations.
    Scopes,
    /// Resolved components used by the template.
    Components,
    /// Directives applied across the template.
    Directives,
    /// `v-bind`/`useCssVars` CSS variable usage.
    CssVars,
    /// Cross-block / cross-file dependency edges.
    DependencyEdges,
    /// Dialect/version facts (which `SourceAtlasCoordinate` applies).
    Dialect,
}

impl CroquisFact {
    /// Known facts in stable observation order.
    pub const KNOWN: [Self; 7] = [
        Self::Bindings,
        Self::Scopes,
        Self::Components,
        Self::Directives,
        Self::CssVars,
        Self::DependencyEdges,
        Self::Dialect,
    ];

    /// Every Croquis fact is a [`PlateFamily::Semantic`] product, so requesting
    /// one never pulls a render plate.
    pub const fn family(self) -> PlateFamily {
        PlateFamily::Semantic
    }

    /// Counter used when a lane requests this fact.
    pub const fn profile_counter(self) -> &'static str {
        match self {
            Self::Bindings => "atelier.profile.croquis.bindings",
            Self::Scopes => "atelier.profile.croquis.scopes",
            Self::Components => "atelier.profile.croquis.components",
            Self::Directives => "atelier.profile.croquis.directives",
            Self::CssVars => "atelier.profile.croquis.css_vars",
            Self::DependencyEdges => "atelier.profile.croquis.dependency_edges",
            Self::Dialect => "atelier.profile.croquis.dialect",
        }
    }

    const fn bit(self) -> u8 {
        match self {
            Self::Bindings => 1 << 0,
            Self::Scopes => 1 << 1,
            Self::Components => 1 << 2,
            Self::Directives => 1 << 3,
            Self::CssVars => 1 << 4,
            Self::DependencyEdges => 1 << 5,
            Self::Dialect => 1 << 6,
        }
    }
}

/// A compact set of requested Croquis facts.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct CroquisFactSet {
    bits: u8,
}

impl CroquisFactSet {
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn from_fact(fact: CroquisFact) -> Self {
        Self { bits: fact.bit() }
    }

    pub const fn with(mut self, fact: CroquisFact) -> Self {
        self.bits |= fact.bit();
        self
    }

    pub const fn contains(self, fact: CroquisFact) -> bool {
        (self.bits & fact.bit()) != 0
    }

    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    pub fn iter(self) -> impl Iterator<Item = CroquisFact> {
        CroquisFact::KNOWN
            .into_iter()
            .filter(move |fact| self.contains(*fact))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_croquis_fact_is_a_semantic_product() {
        for fact in CroquisFact::KNOWN {
            assert_eq!(fact.family(), PlateFamily::Semantic);
        }
    }

    #[test]
    fn fact_sets_deduplicate_and_iterate_in_known_order() {
        let set = CroquisFactSet::empty()
            .with(CroquisFact::Directives)
            .with(CroquisFact::Bindings)
            .with(CroquisFact::Directives);

        assert!(set.contains(CroquisFact::Bindings));
        assert!(set.contains(CroquisFact::Directives));
        assert!(!set.contains(CroquisFact::DependencyEdges));

        let facts: std::vec::Vec<_> = set.iter().collect();
        assert_eq!(facts, [CroquisFact::Bindings, CroquisFact::Directives]);
    }

    #[test]
    fn fact_counters_are_stable() {
        assert_eq!(
            CroquisFact::Bindings.profile_counter(),
            "atelier.profile.croquis.bindings"
        );
        assert_eq!(
            CroquisFact::DependencyEdges.profile_counter(),
            "atelier.profile.croquis.dependency_edges"
        );
    }
}

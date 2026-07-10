//! Neutral Source Atlas plate families.
//!
//! Families group plates by the job they do in the toolchain, matching the
//! `Plate Families` table in `docs/content/architecture/source-atlas.md`.
//! Classifying a request by family lets a lane reason about cost rules — for
//! example "Patina does not pay for Render plates" — from the route it already
//! built, without hard-coding each plate individually.

/// The family a Source Atlas plate, source, or target belongs to.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PlateFamily {
    /// Source files and blocks: SFC, template, script, setup, style, JSX/TSX.
    Source,
    /// Source-faithful syntax surfaces such as Relief and OXC AST refs.
    Syntax,
    /// Shared semantic study: Croquis bindings, scopes, directives, deps, and
    /// dialect/version facts.
    Semantic,
    /// Tool-facing projections such as Virtual TS or inspector views.
    Projection,
    /// Render semantics: the Rendu plate.
    Render,
    /// Target product surfaces: DOM/VDOM, SSR, Vapor, JSX/TSX emit.
    Target,
    /// Finishing artifacts: AtelierOutput, maps, diagnostics, payloads.
    Finish,
}

impl PlateFamily {
    /// Known families in stable observation order.
    pub const KNOWN: [Self; 7] = [
        Self::Source,
        Self::Syntax,
        Self::Semantic,
        Self::Projection,
        Self::Render,
        Self::Target,
        Self::Finish,
    ];

    /// Counter used when a lane observes work in this family.
    pub const fn profile_counter(self) -> &'static str {
        match self {
            Self::Source => "atelier.profile.family.source",
            Self::Syntax => "atelier.profile.family.syntax",
            Self::Semantic => "atelier.profile.family.semantic",
            Self::Projection => "atelier.profile.family.projection",
            Self::Render => "atelier.profile.family.render",
            Self::Target => "atelier.profile.family.target",
            Self::Finish => "atelier.profile.family.finish",
        }
    }

    const fn bit(self) -> u8 {
        match self {
            Self::Source => 1 << 0,
            Self::Syntax => 1 << 1,
            Self::Semantic => 1 << 2,
            Self::Projection => 1 << 3,
            Self::Render => 1 << 4,
            Self::Target => 1 << 5,
            Self::Finish => 1 << 6,
        }
    }
}

/// A compact set of the plate families a lane touched.
///
/// Built by folding the requests a lane already recorded, so asking "did this
/// lane touch a Render plate?" stays allocation-free and adds no analysis to the
/// hot path.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct PlateFamilySet {
    bits: u8,
}

impl PlateFamilySet {
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn from_family(family: PlateFamily) -> Self {
        Self { bits: family.bit() }
    }

    pub const fn with(mut self, family: PlateFamily) -> Self {
        self.bits |= family.bit();
        self
    }

    pub const fn contains(self, family: PlateFamily) -> bool {
        (self.bits & family.bit()) != 0
    }

    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    pub fn iter(self) -> impl Iterator<Item = PlateFamily> {
        PlateFamily::KNOWN
            .into_iter()
            .filter(move |family| self.contains(*family))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn family_set_collects_distinct_families_in_stable_order() {
        let set = PlateFamilySet::empty()
            .with(PlateFamily::Target)
            .with(PlateFamily::Source)
            // Re-adding a family is idempotent.
            .with(PlateFamily::Source);

        assert!(set.contains(PlateFamily::Source));
        assert!(set.contains(PlateFamily::Target));
        assert!(!set.contains(PlateFamily::Render));

        let collected: std::vec::Vec<_> = set.iter().collect();
        assert_eq!(collected, [PlateFamily::Source, PlateFamily::Target]);
    }

    #[test]
    fn empty_family_set_touches_nothing() {
        let set = PlateFamilySet::empty();
        assert!(set.is_empty());
        assert_eq!(set.iter().count(), 0);
        for family in PlateFamily::KNOWN {
            assert!(!set.contains(family));
        }
    }

    #[test]
    fn each_request_kind_reports_its_plate_family() {
        use crate::{
            SourceAtlasCoordinate, SourceAtlasPlate, SourceAtlasRequest, SourceAtlasSource,
            SourceAtlasTarget,
        };

        // A source entry is a Source-family request.
        assert_eq!(
            SourceAtlasRequest::Source(SourceAtlasSource::Tsx).family(),
            PlateFamily::Source
        );
        // A plate request delegates to the plate's own family.
        assert_eq!(
            SourceAtlasRequest::Plate(SourceAtlasPlate::Rendu).family(),
            PlateFamily::Render
        );
        // A target product is a Target-family request.
        assert_eq!(
            SourceAtlasRequest::Target(SourceAtlasTarget::Ssr).family(),
            PlateFamily::Target
        );
        // A version/dialect coordinate is a Semantic fact, not a render request.
        assert_eq!(
            SourceAtlasRequest::Coordinate(SourceAtlasCoordinate::Vapor).family(),
            PlateFamily::Semantic
        );
    }
}

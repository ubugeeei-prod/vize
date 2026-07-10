//! Vue-era compatibility coordinates for Source Atlas.
//!
//! A coordinate is the single capability fact a lane resolves once near the
//! source/semantic layer and then borrows. Tracks `v0`/`v1`/`v2`/`v2.7`/`v3`
//! and the Vapor capability layer so Patina, Canon, Maestro, and the Ateliers
//! agree on one dialect rather than each crate rediscovering version rules.

use super::SourceAtlasFallback;
use vize_carton::config::VueVersion;

/// Compatibility coordinate attached to atlas facts.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SourceAtlasCoordinate {
    Vue(VueVersion),
    Vapor,
}

impl SourceAtlasCoordinate {
    /// Known coordinates in stable observation order: every Vue line newest
    /// first, then the Vapor capability layer.
    pub const KNOWN: [Self; 7] = [
        Self::Vue(VueVersion::V3),
        Self::Vue(VueVersion::V2_7),
        Self::Vue(VueVersion::V2),
        Self::Vue(VueVersion::V1),
        Self::Vue(VueVersion::V0_11),
        Self::Vue(VueVersion::V0_10),
        Self::Vapor,
    ];

    /// Build a coordinate from the configured Vue language version.
    pub const fn from_vue_version(version: VueVersion) -> Self {
        Self::Vue(version)
    }

    /// The Vue dialect this coordinate names, or `None` for the Vapor
    /// capability layer (which is not a Vue version by itself).
    pub const fn vue_version(self) -> Option<VueVersion> {
        match self {
            Self::Vue(version) => Some(version),
            Self::Vapor => None,
        }
    }

    /// Whether this coordinate is the Vapor capability layer.
    pub const fn is_vapor(self) -> bool {
        matches!(self, Self::Vapor)
    }

    /// The compatibility *era* shorthand for this coordinate.
    ///
    /// Eras are the `v0`/`v1`/`v2`/`v2.7`/`v3`/`vapor` lanes the architecture
    /// page reasons about. The two Vue 0.x lines (`V0_10` and the 0.11-era
    /// `V0_11`) fold into one `v0` era so no crate re-derives that grouping;
    /// tooling that must still tell 0.10 from the 0.11 rewrite reads
    /// [`vue_version`](Self::vue_version).
    pub const fn era(self) -> &'static str {
        match self {
            Self::Vue(VueVersion::V3) => "v3",
            Self::Vue(VueVersion::V2_7) => "v2.7",
            Self::Vue(VueVersion::V2) => "v2",
            Self::Vue(VueVersion::V1) => "v1",
            Self::Vue(VueVersion::V0_11) | Self::Vue(VueVersion::V0_10) => "v0",
            Self::Vapor => "vapor",
        }
    }

    /// Counter used when this compatibility coordinate is active.
    pub const fn profile_counter(self) -> &'static str {
        match self {
            Self::Vue(VueVersion::V3) => "atelier.profile.dialect.vue3",
            Self::Vue(VueVersion::V2_7) => "atelier.profile.dialect.vue2_7",
            Self::Vue(VueVersion::V2) => "atelier.profile.dialect.vue2",
            Self::Vue(VueVersion::V1) => "atelier.profile.dialect.vue1",
            Self::Vue(VueVersion::V0_11) => "atelier.profile.dialect.vue0_11",
            Self::Vue(VueVersion::V0_10) => "atelier.profile.dialect.vue0_10",
            Self::Vapor => "atelier.profile.capability.vapor",
        }
    }

    /// Whether this coordinate names a degraded (pre-Vue-3) compatibility line.
    ///
    /// Resolving the coordinate once and asking it here keeps every lane in
    /// agreement about which dialect is legacy, rather than each crate
    /// rediscovering the rule.
    pub const fn is_legacy(self) -> bool {
        matches!(self, Self::Vue(version) if version.is_legacy())
    }

    /// The compatibility fallback this coordinate implies, if any.
    ///
    /// Legacy Vue dialects require a compatibility path, which lanes record as
    /// [`SourceAtlasFallback::LegacySyntaxCompatibility`] so degraded
    /// compilation stays observable instead of silent. Vue 3 and the Vapor
    /// capability layer imply no fallback.
    pub const fn compatibility_fallback(self) -> Option<SourceAtlasFallback> {
        if self.is_legacy() {
            Some(SourceAtlasFallback::LegacySyntaxCompatibility)
        } else {
            None
        }
    }
}

impl From<VueVersion> for SourceAtlasCoordinate {
    fn from(version: VueVersion) -> Self {
        Self::from_vue_version(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_vue_0x_lines_fold_into_one_v0_era() {
        // 0.10 and the 0.11 rewrite are distinct dialects, but share the `v0`
        // compatibility era so lanes reason about one pre-1.0 lane.
        assert_eq!(SourceAtlasCoordinate::from(VueVersion::V0_10).era(), "v0");
        assert_eq!(SourceAtlasCoordinate::from(VueVersion::V0_11).era(), "v0");
        // The distinction is still recoverable through the Vue version.
        assert_eq!(
            SourceAtlasCoordinate::from(VueVersion::V0_10).vue_version(),
            Some(VueVersion::V0_10)
        );
    }

    #[test]
    fn every_coordinate_reports_its_era() {
        let eras: std::vec::Vec<_> = SourceAtlasCoordinate::KNOWN
            .into_iter()
            .map(SourceAtlasCoordinate::era)
            .collect();
        assert_eq!(eras, ["v3", "v2.7", "v2", "v1", "v0", "v0", "vapor"]);
    }

    #[test]
    fn vapor_is_a_capability_layer_not_a_vue_version() {
        assert!(SourceAtlasCoordinate::Vapor.is_vapor());
        assert_eq!(SourceAtlasCoordinate::Vapor.vue_version(), None);
        assert_eq!(SourceAtlasCoordinate::Vapor.era(), "vapor");
        assert!(!SourceAtlasCoordinate::Vapor.is_legacy());

        let v3 = SourceAtlasCoordinate::from(VueVersion::V3);
        assert!(!v3.is_vapor());
        assert_eq!(v3.vue_version(), Some(VueVersion::V3));
    }

    #[test]
    fn legacy_eras_carry_a_compatibility_fallback_and_v3_does_not() {
        for coordinate in SourceAtlasCoordinate::KNOWN {
            let expects_fallback = coordinate.is_legacy();
            assert_eq!(
                coordinate.compatibility_fallback().is_some(),
                expects_fallback,
                "{coordinate:?} fallback should match its legacy status"
            );
            // Only pre-Vue-3 Vue lines are legacy; v3 and vapor are not.
            let is_legacy_era = matches!(coordinate.era(), "v0" | "v1" | "v2" | "v2.7");
            assert_eq!(expects_fallback, is_legacy_era, "{coordinate:?}");
        }
    }
}

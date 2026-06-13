//! Internal Source Atlas vocabulary.
//!
//! These types name the plates and coordinates that Vize tools can request
//! without implying that every plate must be built. Keep this layer `Copy`,
//! allocation-free, and cheap enough to thread through profile/fallback facts.

use vize_carton::config::VueVersion;

/// A demandable plate in the Vize Source Atlas.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SourceAtlasPlate {
    Sfc,
    Template,
    Script,
    Style,
    Relief,
    Croquis,
    VirtualTs,
    Rendu,
    AtelierOutput,
    SourceMap,
}

impl SourceAtlasPlate {
    /// Known plates in stable observation order.
    pub const KNOWN: [Self; 10] = [
        Self::Sfc,
        Self::Template,
        Self::Script,
        Self::Style,
        Self::Relief,
        Self::Croquis,
        Self::VirtualTs,
        Self::Rendu,
        Self::AtelierOutput,
        Self::SourceMap,
    ];

    /// Counter used when a product lane requests or observes this plate.
    pub const fn profile_counter(self) -> &'static str {
        match self {
            Self::Sfc => "atelier.profile.source.sfc",
            Self::Template => "atelier.profile.source.template",
            Self::Script => "atelier.profile.source.script",
            Self::Style => "atelier.profile.source.style",
            Self::Relief => "atelier.profile.plate.relief",
            Self::Croquis => "atelier.profile.plate.croquis",
            Self::VirtualTs => "atelier.profile.plate.virtual_ts",
            Self::Rendu => "atelier.profile.plate.rendu",
            Self::AtelierOutput => "atelier.profile.plate.atelier_output",
            Self::SourceMap => "atelier.profile.plate.source_map",
        }
    }

    const fn bit(self) -> u16 {
        match self {
            Self::Sfc => 1 << 0,
            Self::Template => 1 << 1,
            Self::Script => 1 << 2,
            Self::Style => 1 << 3,
            Self::Relief => 1 << 4,
            Self::Croquis => 1 << 5,
            Self::VirtualTs => 1 << 6,
            Self::Rendu => 1 << 7,
            Self::AtelierOutput => 1 << 8,
            Self::SourceMap => 1 << 9,
        }
    }
}

/// A compact set of requested Source Atlas plates.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct SourceAtlasPlateSet {
    bits: u16,
}

impl SourceAtlasPlateSet {
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn from_plate(plate: SourceAtlasPlate) -> Self {
        Self { bits: plate.bit() }
    }

    pub const fn with(mut self, plate: SourceAtlasPlate) -> Self {
        self.bits |= plate.bit();
        self
    }

    pub const fn contains(self, plate: SourceAtlasPlate) -> bool {
        (self.bits & plate.bit()) != 0
    }

    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    pub fn iter(self) -> impl Iterator<Item = SourceAtlasPlate> {
        SourceAtlasPlate::KNOWN
            .into_iter()
            .filter(move |plate| self.contains(*plate))
    }
}

/// A target lane that may consume atlas plates.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SourceAtlasTarget {
    Dom,
    Ssr,
    Vapor,
}

impl SourceAtlasTarget {
    /// Counter used when this target lane is active.
    pub const fn profile_counter(self) -> &'static str {
        match self {
            Self::Dom => "atelier.profile.target.dom",
            Self::Ssr => "atelier.profile.target.ssr",
            Self::Vapor => "atelier.profile.target.vapor",
        }
    }
}

/// Compatibility coordinate attached to atlas facts.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SourceAtlasCoordinate {
    Vue(VueVersion),
    Vapor,
}

impl SourceAtlasCoordinate {
    /// Build a coordinate from the configured Vue language version.
    pub const fn from_vue_version(version: VueVersion) -> Self {
        Self::Vue(version)
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
}

impl From<VueVersion> for SourceAtlasCoordinate {
    fn from(version: VueVersion) -> Self {
        Self::from_vue_version(version)
    }
}

/// A reason Vize could not project one requested atlas plate directly.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SourceAtlasFallback {
    LegacyLineScanner,
    SourceMapFragmentUnavailable,
    SourceMapCompositionSkipped,
    VirtualTsSkipped,
    UnsupportedVaporShape,
    VaporSsr,
    CustomRendererMismatch,
    LegacySyntaxCompatibility,
    CacheBypass,
}

impl SourceAtlasFallback {
    /// Counter used when this fallback reason is observed.
    pub const fn profile_counter(self) -> &'static str {
        match self {
            Self::LegacyLineScanner => "atelier.fallback.legacy_line_scanner",
            Self::SourceMapFragmentUnavailable => {
                "atelier.fallback.source_map.fragment_unavailable"
            }
            Self::SourceMapCompositionSkipped => "atelier.fallback.source_map.composition_skipped",
            Self::VirtualTsSkipped => "atelier.fallback.virtual_ts.skipped",
            Self::UnsupportedVaporShape => "atelier.fallback.vapor.unsupported_shape",
            Self::VaporSsr => "atelier.fallback.vapor_ssr",
            Self::CustomRendererMismatch => "atelier.fallback.custom_renderer_mismatch",
            Self::LegacySyntaxCompatibility => "atelier.fallback.legacy_syntax_compatibility",
            Self::CacheBypass => "atelier.fallback.cache_bypass",
        }
    }
}

/// A requested atlas fact, separate from the cost of constructing the plate.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SourceAtlasRequest {
    Plate(SourceAtlasPlate),
    Target(SourceAtlasTarget),
    Coordinate(SourceAtlasCoordinate),
}

impl SourceAtlasRequest {
    /// Counter used to observe this request in profile output.
    pub const fn profile_counter(self) -> &'static str {
        match self {
            Self::Plate(plate) => plate.profile_counter(),
            Self::Target(target) => target.profile_counter(),
            Self::Coordinate(coordinate) => coordinate.profile_counter(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vue_coordinates_reuse_config_version_order() {
        let counters: std::vec::Vec<_> = VueVersion::ALL
            .into_iter()
            .map(|version| SourceAtlasCoordinate::from(version).profile_counter())
            .collect();

        assert_eq!(
            counters,
            [
                "atelier.profile.dialect.vue3",
                "atelier.profile.dialect.vue2_7",
                "atelier.profile.dialect.vue2",
                "atelier.profile.dialect.vue1",
                "atelier.profile.dialect.vue0_11",
                "atelier.profile.dialect.vue0_10",
            ]
        );
    }

    #[test]
    fn vapor_is_a_capability_coordinate_not_a_vue_version() {
        assert_eq!(
            SourceAtlasCoordinate::Vapor.profile_counter(),
            "atelier.profile.capability.vapor"
        );
    }

    #[test]
    fn requests_delegate_to_their_plate_family() {
        assert_eq!(
            SourceAtlasRequest::Plate(SourceAtlasPlate::Rendu).profile_counter(),
            "atelier.profile.plate.rendu"
        );
        assert_eq!(
            SourceAtlasRequest::Target(SourceAtlasTarget::Ssr).profile_counter(),
            "atelier.profile.target.ssr"
        );
        assert_eq!(
            SourceAtlasRequest::Coordinate(SourceAtlasCoordinate::from(VueVersion::V2_7))
                .profile_counter(),
            "atelier.profile.dialect.vue2_7"
        );
    }

    #[test]
    fn plate_sets_deduplicate_and_iterate_in_known_order() {
        let set = SourceAtlasPlateSet::empty()
            .with(SourceAtlasPlate::SourceMap)
            .with(SourceAtlasPlate::Sfc)
            .with(SourceAtlasPlate::SourceMap)
            .with(SourceAtlasPlate::VirtualTs);

        assert!(set.contains(SourceAtlasPlate::Sfc));
        assert!(set.contains(SourceAtlasPlate::SourceMap));
        assert!(!set.contains(SourceAtlasPlate::Rendu));

        let plates: std::vec::Vec<_> = set.iter().collect();
        assert_eq!(
            plates,
            [
                SourceAtlasPlate::Sfc,
                SourceAtlasPlate::VirtualTs,
                SourceAtlasPlate::SourceMap,
            ]
        );
    }

    #[test]
    fn fallback_reasons_keep_existing_counter_names_stable() {
        assert_eq!(
            SourceAtlasFallback::VaporSsr.profile_counter(),
            "atelier.fallback.vapor_ssr"
        );
        assert_eq!(
            SourceAtlasFallback::SourceMapCompositionSkipped.profile_counter(),
            "atelier.fallback.source_map.composition_skipped"
        );
    }
}

//! Internal Source Atlas vocabulary.
//!
//! These types name the plates and coordinates that Vize tools can request
//! without implying that every plate must be built. Keep this layer `Copy`,
//! allocation-free, and cheap enough to thread through profile/fallback facts.

use vize_carton::config::VueVersion;

/// A demandable plate in the Vize Source Atlas.
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
}

/// A target lane that may consume atlas plates.
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

/// A requested atlas fact, separate from the cost of constructing the plate.
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
}

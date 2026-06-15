//! Internal Source Atlas vocabulary.
//!
//! These types name the plates and coordinates that Vize tools can request
//! without implying that every plate must be built. Keep this layer `Copy`,
//! allocation-free, and cheap enough to thread through profile/fallback facts.

mod fallback;
mod family;
mod registry;
mod report;
mod route;

pub use fallback::{SourceAtlasFallback, SourceAtlasFallbackSet};
pub use family::{PlateFamily, PlateFamilySet};
pub use registry::{SourceAtlasLane, SourceAtlasRegistry};
pub use report::{render_fallback_legend, render_fallback_report, render_registry_report};
pub use route::{SourceAtlasRoute, SourceAtlasSource, SourceAtlasSourceSet, SourceAtlasTargetSet};

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

    /// The plate family this plate belongs to.
    pub const fn family(self) -> PlateFamily {
        match self {
            Self::Sfc | Self::Template | Self::Script | Self::Style => PlateFamily::Source,
            Self::Relief => PlateFamily::Syntax,
            Self::Croquis => PlateFamily::Semantic,
            Self::VirtualTs => PlateFamily::Projection,
            Self::Rendu => PlateFamily::Render,
            Self::AtelierOutput | Self::SourceMap => PlateFamily::Finish,
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
    Vdom,
    Sfc,
    Ssr,
    Vapor,
    Jsx,
    Tsx,
    VirtualTs,
    Diagnostics,
    SourceMap,
    Vitrine,
}

impl SourceAtlasTarget {
    /// Known target lanes in stable observation order.
    pub const KNOWN: [Self; 11] = [
        Self::Dom,
        Self::Vdom,
        Self::Sfc,
        Self::Ssr,
        Self::Vapor,
        Self::Jsx,
        Self::Tsx,
        Self::VirtualTs,
        Self::Diagnostics,
        Self::SourceMap,
        Self::Vitrine,
    ];

    /// Counter used when this target lane is active.
    pub const fn profile_counter(self) -> &'static str {
        match self {
            Self::Dom => "atelier.profile.target.dom",
            Self::Vdom => "atelier.profile.target.vdom",
            Self::Sfc => "atelier.profile.target.sfc",
            Self::Ssr => "atelier.profile.target.ssr",
            Self::Vapor => "atelier.profile.target.vapor",
            Self::Jsx => "atelier.profile.target.jsx",
            Self::Tsx => "atelier.profile.target.tsx",
            Self::VirtualTs => "atelier.profile.target.virtual_ts",
            Self::Diagnostics => "atelier.profile.target.diagnostics",
            Self::SourceMap => "atelier.profile.target.source_map",
            Self::Vitrine => "atelier.profile.target.vitrine",
        }
    }

    /// Targets always belong to the `Target` plate family.
    pub const fn family(self) -> PlateFamily {
        PlateFamily::Target
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

/// A requested atlas fact, separate from the cost of constructing the plate.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SourceAtlasRequest {
    Source(SourceAtlasSource),
    Plate(SourceAtlasPlate),
    Target(SourceAtlasTarget),
    Coordinate(SourceAtlasCoordinate),
}

impl SourceAtlasRequest {
    /// Counter used to observe this request in profile output.
    pub const fn profile_counter(self) -> &'static str {
        match self {
            Self::Source(source) => source.profile_counter(),
            Self::Plate(plate) => plate.profile_counter(),
            Self::Target(target) => target.profile_counter(),
            Self::Coordinate(coordinate) => coordinate.profile_counter(),
        }
    }

    /// The plate family this request belongs to.
    ///
    /// A version/dialect coordinate is a [`PlateFamily::Semantic`] fact (it
    /// lives with bindings and directives in the `Plate Families` table), so a
    /// lane that only attaches a coordinate is not treated as touching a Render
    /// plate.
    pub const fn family(self) -> PlateFamily {
        match self {
            Self::Source(_) => PlateFamily::Source,
            Self::Plate(plate) => plate.family(),
            Self::Target(_) => PlateFamily::Target,
            Self::Coordinate(_) => PlateFamily::Semantic,
        }
    }
}

#[cfg(test)]
mod tests;

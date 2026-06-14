//! Multi-source and multi-target Source Atlas routes.

use super::{SourceAtlasCoordinate, SourceAtlasPlate, SourceAtlasPlateSet, SourceAtlasTarget};

/// Source entry points that can enter the atlas.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SourceAtlasSource {
    Sfc,
    VueTemplate,
    Script,
    ScriptSetup,
    Style,
    CustomBlock,
    Js,
    Ts,
    Jsx,
    Tsx,
    Css,
    Html,
}

impl SourceAtlasSource {
    /// Known source entries in stable observation order.
    pub const KNOWN: [Self; 12] = [
        Self::Sfc,
        Self::VueTemplate,
        Self::Script,
        Self::ScriptSetup,
        Self::Style,
        Self::CustomBlock,
        Self::Js,
        Self::Ts,
        Self::Jsx,
        Self::Tsx,
        Self::Css,
        Self::Html,
    ];

    /// Counter used when this source lane enters the atlas.
    pub const fn profile_counter(self) -> &'static str {
        match self {
            Self::Sfc => "atelier.profile.input.sfc",
            Self::VueTemplate => "atelier.profile.input.vue_template",
            Self::Script => "atelier.profile.input.script",
            Self::ScriptSetup => "atelier.profile.input.script_setup",
            Self::Style => "atelier.profile.input.style",
            Self::CustomBlock => "atelier.profile.input.custom_block",
            Self::Js => "atelier.profile.input.js",
            Self::Ts => "atelier.profile.input.ts",
            Self::Jsx => "atelier.profile.input.jsx",
            Self::Tsx => "atelier.profile.input.tsx",
            Self::Css => "atelier.profile.input.css",
            Self::Html => "atelier.profile.input.html",
        }
    }

    const fn bit(self) -> u16 {
        match self {
            Self::Sfc => 1 << 0,
            Self::VueTemplate => 1 << 1,
            Self::Script => 1 << 2,
            Self::ScriptSetup => 1 << 3,
            Self::Style => 1 << 4,
            Self::CustomBlock => 1 << 5,
            Self::Js => 1 << 6,
            Self::Ts => 1 << 7,
            Self::Jsx => 1 << 8,
            Self::Tsx => 1 << 9,
            Self::Css => 1 << 10,
            Self::Html => 1 << 11,
        }
    }
}

/// Compact source-entry set.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct SourceAtlasSourceSet {
    bits: u16,
}

impl SourceAtlasSourceSet {
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn with(mut self, source: SourceAtlasSource) -> Self {
        self.bits |= source.bit();
        self
    }

    pub const fn contains(self, source: SourceAtlasSource) -> bool {
        (self.bits & source.bit()) != 0
    }

    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    pub fn iter(self) -> impl Iterator<Item = SourceAtlasSource> {
        SourceAtlasSource::KNOWN
            .into_iter()
            .filter(move |source| self.contains(*source))
    }
}

/// Compact target-lane set.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct SourceAtlasTargetSet {
    bits: u16,
}

impl SourceAtlasTargetSet {
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn with(mut self, target: SourceAtlasTarget) -> Self {
        self.bits |= target_bit(target);
        self
    }

    pub const fn contains(self, target: SourceAtlasTarget) -> bool {
        (self.bits & target_bit(target)) != 0
    }

    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }
}

/// A cheap route request through the Source Atlas.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct SourceAtlasRoute {
    pub sources: SourceAtlasSourceSet,
    pub targets: SourceAtlasTargetSet,
    pub plates: SourceAtlasPlateSet,
    pub coordinate: Option<SourceAtlasCoordinate>,
}

impl SourceAtlasRoute {
    pub const fn empty() -> Self {
        Self {
            sources: SourceAtlasSourceSet::empty(),
            targets: SourceAtlasTargetSet::empty(),
            plates: SourceAtlasPlateSet::empty(),
            coordinate: None,
        }
    }

    pub const fn with_source(mut self, source: SourceAtlasSource) -> Self {
        self.sources = self.sources.with(source);
        self
    }

    pub const fn with_target(mut self, target: SourceAtlasTarget) -> Self {
        self.targets = self.targets.with(target);
        self
    }

    pub const fn with_plate(mut self, plate: SourceAtlasPlate) -> Self {
        self.plates = self.plates.with(plate);
        self
    }

    pub const fn with_coordinate(mut self, coordinate: SourceAtlasCoordinate) -> Self {
        self.coordinate = Some(coordinate);
        self
    }
}

const fn target_bit(target: SourceAtlasTarget) -> u16 {
    match target {
        SourceAtlasTarget::Dom => 1 << 0,
        SourceAtlasTarget::Vdom => 1 << 1,
        SourceAtlasTarget::Sfc => 1 << 2,
        SourceAtlasTarget::Ssr => 1 << 3,
        SourceAtlasTarget::Vapor => 1 << 4,
        SourceAtlasTarget::Jsx => 1 << 5,
        SourceAtlasTarget::Tsx => 1 << 6,
        SourceAtlasTarget::VirtualTs => 1 << 7,
        SourceAtlasTarget::Diagnostics => 1 << 8,
        SourceAtlasTarget::SourceMap => 1 << 9,
        SourceAtlasTarget::Vitrine => 1 << 10,
    }
}

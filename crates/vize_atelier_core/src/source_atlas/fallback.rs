//! Source Atlas fallback facts.

/// A reason Vize could not project one requested atlas plate directly.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SourceAtlasFallback {
    /// A source-map lane was requested, but the producing Atelier did not
    /// provide a fragment to register.
    SourceMapFragmentUnavailable,
    /// A valid fragment exists, but full SFC map composition has not happened
    /// at the current assembly boundary.
    SourceMapCompositionSkipped,
    /// A typecheck or editor lane did not need, or could not produce, Virtual
    /// TS for the requested source shape.
    VirtualTsSkipped,
    /// Vapor was requested for a render shape outside the current Vapor Atelier
    /// capability set.
    UnsupportedVaporShape,
    /// SSR was requested together with Vapor, but the SFC compiler had to use
    /// the standard SSR Atelier because Vapor SSR is not implemented yet.
    VaporSsr,
    /// A custom renderer request could not be satisfied by the selected target
    /// Atelier without changing semantics.
    CustomRendererMismatch,
    /// Legacy Vue syntax required a compatibility path rather than the default
    /// modern Vue projection.
    LegacySyntaxCompatibility,
    /// A cacheable lane intentionally bypassed cache reuse.
    CacheBypass,
}

impl SourceAtlasFallback {
    /// Known fallback facts in stable observation order.
    pub const KNOWN: [Self; 8] = [
        Self::SourceMapFragmentUnavailable,
        Self::SourceMapCompositionSkipped,
        Self::VirtualTsSkipped,
        Self::UnsupportedVaporShape,
        Self::VaporSsr,
        Self::CustomRendererMismatch,
        Self::LegacySyntaxCompatibility,
        Self::CacheBypass,
    ];

    /// Counter used when this fallback reason is observed.
    pub const fn profile_counter(self) -> &'static str {
        match self {
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

    /// A short, human-facing explanation of why this fallback was taken.
    ///
    /// This is what turns a fallback counter into a first-class fact: a tool can
    /// show *why* it stayed on a legacy bridge or omitted a map without a
    /// developer reconstructing the decision from output strings.
    pub const fn description(self) -> &'static str {
        match self {
            Self::SourceMapFragmentUnavailable => {
                "the producing Atelier emitted no source-map fragment to register"
            }
            Self::SourceMapCompositionSkipped => {
                "a valid map fragment exists but full SFC composition was deferred"
            }
            Self::VirtualTsSkipped => "Virtual TS was not needed or not produced for this source",
            Self::UnsupportedVaporShape => {
                "the template shape is outside the current Vapor capability set"
            }
            Self::VaporSsr => "Vapor SSR is unimplemented, so standard SSR output was used",
            Self::CustomRendererMismatch => {
                "the selected target Atelier could not satisfy the custom renderer request"
            }
            Self::LegacySyntaxCompatibility => "a pre-Vue-3 dialect required a compatibility path",
            Self::CacheBypass => "a cacheable lane intentionally bypassed cache reuse",
        }
    }

    /// Resolve a profile counter name back to its fallback fact.
    ///
    /// Lets a report consumer (such as the curator profile renderer) annotate a
    /// recorded counter with the typed reason and its [`description`].
    ///
    /// [`description`]: Self::description
    pub fn from_profile_counter(counter: &str) -> Option<Self> {
        Self::KNOWN
            .into_iter()
            .find(|fallback| fallback.profile_counter() == counter)
    }

    const fn bit(self) -> u16 {
        match self {
            Self::SourceMapFragmentUnavailable => 1 << 0,
            Self::SourceMapCompositionSkipped => 1 << 1,
            Self::VirtualTsSkipped => 1 << 2,
            Self::UnsupportedVaporShape => 1 << 3,
            Self::VaporSsr => 1 << 4,
            Self::CustomRendererMismatch => 1 << 5,
            Self::LegacySyntaxCompatibility => 1 << 6,
            Self::CacheBypass => 1 << 7,
        }
    }
}

/// Compact set of fallback facts observed by an atlas lane.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct SourceAtlasFallbackSet {
    bits: u16,
}

impl SourceAtlasFallbackSet {
    /// Build an empty fallback set.
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// Build a set containing one fallback reason.
    pub const fn from_fallback(fallback: SourceAtlasFallback) -> Self {
        Self {
            bits: fallback.bit(),
        }
    }

    /// Add one fallback reason, preserving the existing reasons.
    pub const fn with(mut self, fallback: SourceAtlasFallback) -> Self {
        self.bits |= fallback.bit();
        self
    }

    /// Return whether a fallback reason is present.
    pub const fn contains(self, fallback: SourceAtlasFallback) -> bool {
        (self.bits & fallback.bit()) != 0
    }

    /// Return whether no fallback reason was observed.
    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    /// Iterate fallbacks in stable profile/reporting order.
    pub fn iter(self) -> impl Iterator<Item = SourceAtlasFallback> {
        SourceAtlasFallback::KNOWN
            .into_iter()
            .filter(move |fallback| self.contains(*fallback))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reasons_round_trip_through_their_counter_and_carry_a_description() {
        for fallback in SourceAtlasFallback::KNOWN {
            // The counter name resolves back to the same typed reason.
            assert_eq!(
                SourceAtlasFallback::from_profile_counter(fallback.profile_counter()),
                Some(fallback)
            );
            // Every reason explains itself.
            assert!(
                !fallback.description().is_empty(),
                "{fallback:?} should describe itself"
            );
        }
        assert_eq!(
            SourceAtlasFallback::from_profile_counter("atelier.fallback.not_a_real_reason"),
            None
        );
    }
}

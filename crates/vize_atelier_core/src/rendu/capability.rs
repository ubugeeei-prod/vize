//! Single render capability facts exposed by a Rendu root.
//!
//! [`RenduCapabilities`] is the compact set a Rendu root carries. This module
//! names one fact at a time so a target Atelier can ask "is this shape
//! renderable for me?" without inspecting the raw target bitset, coordinate, or
//! custom-renderer flag directly.

use super::RenduCapabilities;
use crate::source_atlas::{SourceAtlasCoordinate, SourceAtlasFallback, SourceAtlasTarget};

/// A single render capability fact.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum RenduCapability {
    /// Output can target the VDOM/DOM render lane.
    Dom,
    /// Output can target server-side rendering.
    Ssr,
    /// Output can target the Vapor render lane.
    Vapor,
    /// A custom (non-DOM) renderer is in use.
    CustomRenderer,
    /// A versioned-syntax (dialect) coordinate is attached.
    VersionedSyntax(SourceAtlasCoordinate),
}

/// How a Vapor render route resolves for a capability set.
///
/// This is the shared classification the Vapor lane consumes and fallback
/// reporting agrees with, so `Rendu -> Vapor IR -> Vapor output` does not
/// re-derive the route rule inline.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VaporSupport {
    /// The shape renders on the Vapor target as-is.
    Renderable,
    /// Vapor was requested together with SSR; the SFC compiler degrades to the
    /// standard SSR Atelier because Vapor SSR is unimplemented.
    DegradesToSsr,
    /// The template shape is outside the current Vapor capability set.
    Unsupported,
}

impl VaporSupport {
    /// The atlas fallback this Vapor outcome records, if any.
    pub const fn fallback(self) -> Option<SourceAtlasFallback> {
        match self {
            Self::Renderable => None,
            Self::DegradesToSsr => Some(SourceAtlasFallback::VaporSsr),
            Self::Unsupported => Some(SourceAtlasFallback::UnsupportedVaporShape),
        }
    }

    /// Whether the Vapor target renders this shape without degrading.
    pub const fn is_renderable(self) -> bool {
        matches!(self, Self::Renderable)
    }
}

impl RenduCapabilities {
    /// Whether a single capability fact holds for this set.
    pub fn supports(self, capability: RenduCapability) -> bool {
        match capability {
            RenduCapability::Dom => {
                self.targets.contains(SourceAtlasTarget::Dom)
                    || self.targets.contains(SourceAtlasTarget::Vdom)
            }
            RenduCapability::Ssr => self.targets.contains(SourceAtlasTarget::Ssr),
            RenduCapability::Vapor => self.targets.contains(SourceAtlasTarget::Vapor),
            RenduCapability::CustomRenderer => self.custom_renderer,
            RenduCapability::VersionedSyntax(coordinate) => self.coordinate == Some(coordinate),
        }
    }

    /// The fallback a Vapor render route implies for this capability set.
    ///
    /// Vapor SSR is not supported, so a route that asks for both the Vapor and
    /// SSR targets degrades to standard SSR; the mismatch is reported as
    /// [`SourceAtlasFallback::VaporSsr`] so it stays observable. This is the one
    /// shared decision the Vapor lane consumes instead of re-deriving the rule
    /// inline. Returns `None` when the capability set is renderable as-is.
    pub fn vapor_route_fallback(self) -> Option<SourceAtlasFallback> {
        if self.supports(RenduCapability::Vapor) && self.supports(RenduCapability::Ssr) {
            Some(SourceAtlasFallback::VaporSsr)
        } else {
            None
        }
    }

    /// Classify how a Vapor render route resolves for this capability set.
    ///
    /// `shape_renderable` is whether the Vapor Atelier could lower the template
    /// shape (Vapor reports this by succeeding or failing). The single rule:
    /// an unrenderable shape is [`VaporSupport::Unsupported`], a Vapor+SSR route
    /// degrades to standard SSR ([`VaporSupport::DegradesToSsr`]), and anything
    /// else is [`VaporSupport::Renderable`]. The Vapor lane and `AtelierFallback`
    /// reporting consume this instead of re-deriving the rule.
    pub fn vapor_support(self, shape_renderable: bool) -> VaporSupport {
        if !shape_renderable {
            VaporSupport::Unsupported
        } else if self.vapor_route_fallback() == Some(SourceAtlasFallback::VaporSsr) {
            VaporSupport::DegradesToSsr
        } else {
            VaporSupport::Renderable
        }
    }

    /// Visit each capability fact present, in stable order.
    ///
    /// Allocation-free, mirroring `SourceAtlasRegistry::visit_requests`, so it
    /// stays cheap enough for inspector/profile lanes.
    pub fn visit_facts(self, mut visit: impl FnMut(RenduCapability)) {
        if self.supports(RenduCapability::Dom) {
            visit(RenduCapability::Dom);
        }
        if self.supports(RenduCapability::Ssr) {
            visit(RenduCapability::Ssr);
        }
        if self.supports(RenduCapability::Vapor) {
            visit(RenduCapability::Vapor);
        }
        if self.custom_renderer {
            visit(RenduCapability::CustomRenderer);
        }
        if let Some(coordinate) = self.coordinate {
            visit(RenduCapability::VersionedSyntax(coordinate));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_carton::config::VueVersion;

    #[test]
    fn vdom_target_satisfies_the_dom_capability() {
        let caps = RenduCapabilities::empty().with_target(SourceAtlasTarget::Vdom);
        assert!(caps.supports(RenduCapability::Dom));
        assert!(!caps.supports(RenduCapability::Ssr));
        assert!(!caps.supports(RenduCapability::Vapor));
    }

    #[test]
    fn custom_renderer_and_coordinate_are_capability_facts() {
        let coordinate = SourceAtlasCoordinate::from(VueVersion::V2_7);
        let caps = RenduCapabilities::empty()
            .with_custom_renderer(true)
            .with_coordinate(coordinate);
        assert!(caps.supports(RenduCapability::CustomRenderer));
        assert!(caps.supports(RenduCapability::VersionedSyntax(coordinate)));
        assert!(!caps.supports(RenduCapability::VersionedSyntax(
            SourceAtlasCoordinate::Vapor
        )));
    }

    #[test]
    fn vapor_plus_ssr_route_reports_a_vapor_ssr_fallback() {
        let vapor_ssr = RenduCapabilities::empty()
            .with_target(SourceAtlasTarget::Vapor)
            .with_target(SourceAtlasTarget::Ssr);
        assert_eq!(
            vapor_ssr.vapor_route_fallback(),
            Some(SourceAtlasFallback::VaporSsr)
        );

        // Vapor-only and SSR-only routes are renderable as-is.
        let vapor_only = RenduCapabilities::empty().with_target(SourceAtlasTarget::Vapor);
        assert_eq!(vapor_only.vapor_route_fallback(), None);
        let ssr_only = RenduCapabilities::empty().with_target(SourceAtlasTarget::Ssr);
        assert_eq!(ssr_only.vapor_route_fallback(), None);
    }

    #[test]
    fn vapor_support_classifies_renderable_degraded_and_unsupported() {
        let vapor = RenduCapabilities::empty().with_target(SourceAtlasTarget::Vapor);
        // A renderable Vapor-only shape needs no fallback.
        assert_eq!(vapor.vapor_support(true), VaporSupport::Renderable);
        assert!(vapor.vapor_support(true).is_renderable());
        assert_eq!(vapor.vapor_support(true).fallback(), None);

        // An unrenderable shape reports UnsupportedVaporShape regardless of route.
        assert_eq!(vapor.vapor_support(false), VaporSupport::Unsupported);
        assert_eq!(
            vapor.vapor_support(false).fallback(),
            Some(SourceAtlasFallback::UnsupportedVaporShape)
        );

        // Vapor + SSR degrades to standard SSR even when the shape renders.
        let vapor_ssr = vapor.with_target(SourceAtlasTarget::Ssr);
        assert_eq!(vapor_ssr.vapor_support(true), VaporSupport::DegradesToSsr);
        assert_eq!(
            vapor_ssr.vapor_support(true).fallback(),
            Some(SourceAtlasFallback::VaporSsr)
        );
        // An unrenderable shape wins over the route degrade.
        assert_eq!(vapor_ssr.vapor_support(false), VaporSupport::Unsupported);
    }

    #[test]
    fn visit_facts_yields_present_capabilities_in_stable_order() {
        let coordinate = SourceAtlasCoordinate::Vapor;
        let caps = RenduCapabilities::empty()
            .with_target(SourceAtlasTarget::Dom)
            .with_target(SourceAtlasTarget::Vapor)
            .with_custom_renderer(true)
            .with_coordinate(coordinate);

        let mut facts = std::vec::Vec::new();
        caps.visit_facts(|fact| facts.push(fact));

        assert_eq!(
            facts,
            [
                RenduCapability::Dom,
                RenduCapability::Vapor,
                RenduCapability::CustomRenderer,
                RenduCapability::VersionedSyntax(coordinate),
            ]
        );
    }

    #[test]
    fn empty_capabilities_visit_nothing() {
        let mut count = 0usize;
        RenduCapabilities::empty().visit_facts(|_| count += 1);
        assert_eq!(count, 0);
    }
}

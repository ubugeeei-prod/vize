//! Source-map registration marks for the Source Atlas.
//!
//! A registration names already-produced map material and the generated Rendu
//! range it covers. It intentionally borrows the fragment instead of composing
//! maps here, so consumers can observe source-map readiness without paying for
//! an extra output pass.

use crate::{
    rendu::RenduRange,
    source_atlas::{
        SourceAtlasFallback, SourceAtlasPlate, SourceAtlasRoute, SourceAtlasSource,
        SourceAtlasTarget,
    },
};

/// The generated section covered by a source-map registration.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SourceMapRegistrationSection {
    Module,
    TemplateRender,
    Script,
    Style,
    CustomBlock,
}

impl SourceMapRegistrationSection {
    /// Counter used when this generated section is registered.
    pub const fn profile_counter(self) -> &'static str {
        match self {
            Self::Module => "atelier.profile.source_map.section.module",
            Self::TemplateRender => "atelier.profile.source_map.section.template_render",
            Self::Script => "atelier.profile.source_map.section.script",
            Self::Style => "atelier.profile.source_map.section.style",
            Self::CustomBlock => "atelier.profile.source_map.section.custom_block",
        }
    }
}

/// Composition status for a source-map registration.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SourceMapRegistrationState {
    Composed,
    Deferred(SourceAtlasFallback),
    Omitted(SourceAtlasFallback),
}

impl SourceMapRegistrationState {
    pub const fn is_composed(self) -> bool {
        matches!(self, Self::Composed)
    }

    pub const fn fallback(self) -> Option<SourceAtlasFallback> {
        match self {
            Self::Composed => None,
            Self::Deferred(reason) | Self::Omitted(reason) => Some(reason),
        }
    }
}

/// Borrowed source-map material attached to a generated Rendu range.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SourceMapRegistration<'a> {
    pub route: SourceAtlasRoute,
    pub section: SourceMapRegistrationSection,
    pub source_name: Option<&'a str>,
    pub generated: RenduRange,
    pub fragment: Option<&'a str>,
    pub state: SourceMapRegistrationState,
}

impl<'a> SourceMapRegistration<'a> {
    pub const fn new(
        route: SourceAtlasRoute,
        section: SourceMapRegistrationSection,
        generated: RenduRange,
        fragment: Option<&'a str>,
        state: SourceMapRegistrationState,
    ) -> Self {
        Self {
            route,
            section,
            source_name: None,
            generated,
            fragment,
            state,
        }
    }

    pub const fn for_template_fragment(
        generated: RenduRange,
        fragment: &'a str,
        state: SourceMapRegistrationState,
    ) -> Self {
        Self::new(
            SourceAtlasRoute::empty()
                .with_source(SourceAtlasSource::VueTemplate)
                .with_target(SourceAtlasTarget::SourceMap)
                .with_plate(SourceAtlasPlate::SourceMap),
            SourceMapRegistrationSection::TemplateRender,
            generated,
            Some(fragment),
            state,
        )
    }

    pub const fn with_source_name(mut self, source_name: &'a str) -> Self {
        self.source_name = Some(source_name);
        self
    }

    pub const fn has_fragment(self) -> bool {
        self.fragment.is_some()
    }

    pub const fn is_composed(self) -> bool {
        self.state.is_composed()
    }

    pub const fn fallback(self) -> Option<SourceAtlasFallback> {
        self.state.fallback()
    }

    pub const fn generated_len(self) -> usize {
        self.generated.end.saturating_sub(self.generated.start)
    }
}

/// Outcome of composing an SFC source map from registered output sections.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SfcMapComposition<'a> {
    /// A single template-render fragment is the whole generated map, so it can
    /// be surfaced directly as the SFC source map.
    Composed(&'a str),
    /// Composition is valid but deferred to a later script/template/style
    /// assembly plate.
    Deferred(SourceAtlasFallback),
    /// No registered fragment was available to compose.
    Omitted(SourceAtlasFallback),
}

impl<'a> SfcMapComposition<'a> {
    /// The composed map, when composition succeeded.
    pub const fn composed_map(self) -> Option<&'a str> {
        match self {
            Self::Composed(map) => Some(map),
            Self::Deferred(_) | Self::Omitted(_) => None,
        }
    }

    /// The fallback reason recorded for a deferred or omitted composition.
    pub const fn fallback(self) -> Option<SourceAtlasFallback> {
        match self {
            Self::Composed(_) => None,
            Self::Deferred(reason) | Self::Omitted(reason) => Some(reason),
        }
    }
}

/// Decide whether registered sections compose into one SFC source map.
///
/// This is the single authority for the composition rule, instead of each
/// caller hand-deciding composed-vs-skipped. Composition is only correct today
/// for a single template-render fragment that covers the whole module (a
/// template-only SFC): then the fragment *is* the final map. A module that also
/// registers script/style sections still needs a composition plate and is
/// reported as [`Deferred`](SfcMapComposition::Deferred); a module with no
/// fragment is [`Omitted`](SfcMapComposition::Omitted). It composes from
/// already-registered sections and never rescans generated JavaScript.
pub fn compose_sfc_source_map<'a>(
    registrations: &[SourceMapRegistration<'a>],
) -> SfcMapComposition<'a> {
    let mut with_fragment = registrations.iter().filter(|reg| reg.has_fragment());
    let Some(first) = with_fragment.next() else {
        return SfcMapComposition::Omitted(SourceAtlasFallback::SourceMapFragmentUnavailable);
    };
    // A lone template-render fragment is the entire module map.
    if with_fragment.next().is_none()
        && matches!(first.section, SourceMapRegistrationSection::TemplateRender)
        && let Some(fragment) = first.fragment
    {
        return SfcMapComposition::Composed(fragment);
    }
    SfcMapComposition::Deferred(SourceAtlasFallback::SourceMapCompositionSkipped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_registrations_carry_route_and_range() {
        let registration = SourceMapRegistration::for_template_fragment(
            RenduRange::new(12, 48),
            "{\"version\":3}",
            SourceMapRegistrationState::Composed,
        )
        .with_source_name("template.vue");

        assert!(registration.has_fragment());
        assert!(registration.is_composed());
        assert_eq!(registration.generated_len(), 36);
        assert_eq!(registration.source_name, Some("template.vue"));
        assert!(
            registration
                .route
                .sources
                .contains(SourceAtlasSource::VueTemplate)
        );
        assert!(
            registration
                .route
                .targets
                .contains(SourceAtlasTarget::SourceMap)
        );
        assert!(
            registration
                .route
                .plates
                .contains(SourceAtlasPlate::SourceMap)
        );
    }

    #[test]
    fn deferred_registrations_report_their_fallback() {
        let registration = SourceMapRegistration::for_template_fragment(
            RenduRange::new(0, 10),
            "{}",
            SourceMapRegistrationState::Deferred(SourceAtlasFallback::SourceMapCompositionSkipped),
        );

        assert!(!registration.is_composed());
        assert_eq!(
            registration.fallback(),
            Some(SourceAtlasFallback::SourceMapCompositionSkipped)
        );
    }

    fn template_fragment(fragment: &str) -> SourceMapRegistration<'_> {
        SourceMapRegistration::for_template_fragment(
            RenduRange::new(0, 20),
            fragment,
            SourceMapRegistrationState::Composed,
        )
    }

    fn script_fragment(fragment: &str) -> SourceMapRegistration<'_> {
        SourceMapRegistration::new(
            SourceAtlasRoute::empty().with_source(SourceAtlasSource::Script),
            SourceMapRegistrationSection::Script,
            RenduRange::new(0, 10),
            Some(fragment),
            SourceMapRegistrationState::Composed,
        )
    }

    #[test]
    fn a_lone_template_fragment_composes_into_the_module_map() {
        let regs = [template_fragment("{\"version\":3}")];
        let composition = compose_sfc_source_map(&regs);
        assert_eq!(composition.composed_map(), Some("{\"version\":3}"));
        assert_eq!(composition.fallback(), None);
    }

    #[test]
    fn no_registered_fragment_is_an_omitted_composition() {
        let composition = compose_sfc_source_map(&[]);
        assert_eq!(composition.composed_map(), None);
        assert_eq!(
            composition.fallback(),
            Some(SourceAtlasFallback::SourceMapFragmentUnavailable)
        );
    }

    #[test]
    fn template_plus_script_fragments_defer_composition() {
        // Two sections still need a real composition plate, so the rule reports
        // a deferred composition rather than surfacing a half-composed map.
        let regs = [
            template_fragment("{\"version\":3,\"file\":\"t\"}"),
            script_fragment("{\"version\":3,\"file\":\"s\"}"),
        ];
        let composition = compose_sfc_source_map(&regs);
        assert_eq!(composition.composed_map(), None);
        assert_eq!(
            composition.fallback(),
            Some(SourceAtlasFallback::SourceMapCompositionSkipped)
        );
    }
}

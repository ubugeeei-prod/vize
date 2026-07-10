//! Source-map bookkeeping for SFC output assembly.

use crate::compile_template::TemplateBlockCompileResult;
use vize_atelier_core::{
    atelier_output::AtelierFallback,
    source_map::{SourceMapRegistration, SourceMapRegistrationState},
};
use vize_carton::profiler::global_profiler;

#[derive(Clone, Copy)]
pub(crate) enum SourceMapComposition {
    /// The map fragment can be surfaced as the final SFC map for this compile
    /// shape.
    Composed,
    /// The map fragment is still valid but cannot be surfaced until a later
    /// script/template/style composition plate exists.
    Skipped,
}

/// Record the template source-map artifact requested by SFC output assembly.
///
/// This function does not compose maps. It observes the map fragment and the
/// generated output range already attached to `TemplateBlockCompileResult`,
/// then records the provider observation directly.
/// That makes both skipped composition and unavailable fragments visible
/// without forcing normal compile or lint paths to pay for a full source-map
/// composition pass.
pub(crate) fn record_template_source_map_fact(
    template_output: &TemplateBlockCompileResult,
    composition: SourceMapComposition,
) {
    let Some(registration) = template_source_map_registration(template_output, composition) else {
        // No fragment was produced (DOM without source maps, or SSR/Vapor
        // which emit no template fragment yet). Report the omission instead of
        // losing the intent silently.
        record_missing_template_fragment();
        return;
    };

    let profiler = global_profiler();
    profiler.record_counter("atelier.profile.template_source_map_fragments", 1);
    record_source_map_observation(registration.fallback());
    profiler.record_counter("atelier.profile.source_map.registrations", 1);
    profiler.record_counter(
        "atelier.profile.source_map.generated_bytes",
        usize_to_counter(registration.generated_len()),
    );
    profiler.record_counter(registration.section.profile_counter(), 1);
}

/// Report that a template lane produced no source-map fragment to compose.
///
/// Mirrors the registered-fragment path so an unavailable fragment is an
/// observable `SourceMapFragmentUnavailable` fact on the `SourceMap` lane
/// rather than a silent gap.
fn record_missing_template_fragment() {
    record_source_map_observation(Some(AtelierFallback::SourceMapFragmentUnavailable));
}

/// Emit the compatibility profile counters for a source-map registration.
///
/// Keeping this helper lane-shaped matters for #1634: source maps are not a
/// codegen afterthought. The graph-native path records the same information on
/// its source-map product outcome and execution trace.
fn record_source_map_observation(fallback: Option<AtelierFallback>) {
    let profiler = global_profiler();
    profiler.record_counter("atelier.profile.lane.source_map", 1);
    profiler.record_counter("atelier.profile.input.vue_template", 1);
    profiler.record_counter("atelier.profile.target.source_map", 1);
    profiler.record_counter("atelier.profile.plate.source_map", 1);
    if let Some(fallback) = fallback {
        profiler.record_counter(fallback.profile_counter(), 1);
    }
}

fn template_source_map_registration(
    template_output: &TemplateBlockCompileResult,
    composition: SourceMapComposition,
) -> Option<SourceMapRegistration<'_>> {
    template_output.source_map_registration(source_map_registration_state(composition))
}

fn source_map_registration_state(composition: SourceMapComposition) -> SourceMapRegistrationState {
    match composition {
        SourceMapComposition::Composed => SourceMapRegistrationState::Composed,
        SourceMapComposition::Skipped => {
            SourceMapRegistrationState::Deferred(AtelierFallback::SourceMapCompositionSkipped)
        }
    }
}

fn usize_to_counter(value: usize) -> u64 {
    value.try_into().unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::output_module::{AtelierModuleSections, AtelierOutputMaps};
    use vize_carton::String;

    fn template_result_without_map() -> TemplateBlockCompileResult {
        TemplateBlockCompileResult {
            code: String::from("function render() {}\n"),
            warnings: std::vec::Vec::new(),
            sections: None,
            module_sections: Some(AtelierModuleSections::from_chunk_lengths(0, 0, 20, 0)),
            maps: AtelierOutputMaps::default(),
        }
    }

    fn template_result_with_map() -> TemplateBlockCompileResult {
        TemplateBlockCompileResult {
            code: String::from("import { h } from \"vue\"\n\nfunction render() {}\n"),
            warnings: std::vec::Vec::new(),
            sections: None,
            module_sections: Some(AtelierModuleSections::from_chunk_lengths(24, 0, 20, 0)),
            maps: AtelierOutputMaps::from_source_map(Some(String::from("{\"version\":3}"))),
        }
    }

    #[test]
    fn template_source_map_registration_marks_generated_render_range() {
        let output = template_result_with_map();
        let registration =
            template_source_map_registration(&output, SourceMapComposition::Composed)
                .expect("source-map fragment should register");

        assert_eq!(
            registration.generated,
            output.module_sections.unwrap().functions
        );
        assert_eq!(registration.source_name, Some("template.vue"));
        assert!(registration.is_composed());
        assert_eq!(
            registration.source,
            vize_atelier_core::source_map::SourceMapSource::VueTemplate
        );
    }

    #[test]
    fn templates_without_a_fragment_have_no_registration() {
        let output = template_result_without_map();
        assert!(
            template_source_map_registration(&output, SourceMapComposition::Composed).is_none(),
            "a template without a map fragment should not register one"
        );
    }

    #[test]
    fn missing_fragment_uses_the_output_owned_fallback_reason() {
        assert_eq!(
            AtelierFallback::SourceMapFragmentUnavailable.profile_counter(),
            "atelier.profile.fallback.source_map_fragment_unavailable"
        );
    }

    #[test]
    fn skipped_template_source_maps_keep_deferred_reason() {
        let output = template_result_with_map();
        let registration = template_source_map_registration(&output, SourceMapComposition::Skipped)
            .expect("source-map fragment should register");

        assert_eq!(
            registration.fallback(),
            Some(AtelierFallback::SourceMapCompositionSkipped)
        );
    }

    #[test]
    fn template_only_registration_composes_into_a_module_map() {
        use vize_atelier_core::source_map::compose_sfc_source_map;

        // A template-only SFC registers a single template-render fragment, so
        // the shared composition authority surfaces it as the whole module map.
        let output = template_result_with_map();
        let registration = output
            .source_map_registration(SourceMapRegistrationState::Composed)
            .expect("template fragment should register");

        let composition = compose_sfc_source_map(&[registration]);
        assert_eq!(composition.composed_map(), Some("{\"version\":3}"));
        assert_eq!(composition.fallback(), None);
    }

    #[test]
    fn a_template_without_a_fragment_omits_composition() {
        use vize_atelier_core::source_map::compose_sfc_source_map;

        let output = template_result_without_map();
        let composition = match output.source_map_registration(SourceMapRegistrationState::Composed)
        {
            Some(registration) => compose_sfc_source_map(&[registration]),
            None => compose_sfc_source_map(&[]),
        };
        assert_eq!(composition.composed_map(), None);
        assert_eq!(
            composition.fallback(),
            Some(AtelierFallback::SourceMapFragmentUnavailable)
        );
    }
}

//! Source-map bookkeeping for SFC output assembly.

use crate::compile_template::TemplateBlockCompileResult;
use vize_atelier_core::{
    source_atlas::{SourceAtlasFallback, SourceAtlasRegistry},
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

/// Record the template source-map plate requested by SFC output assembly.
///
/// This function does not compose maps. It observes the map fragment and the
/// generated Rendu range already attached to `TemplateBlockCompileResult`,
/// then records that registration through the Source Atlas `SourceMap` lane.
/// That makes skipped composition visible without forcing normal compile or
/// lint paths to pay for a full source-map composition pass.
pub(crate) fn record_template_source_map_fact(
    template_output: &TemplateBlockCompileResult,
    composition: SourceMapComposition,
) {
    let Some(registration) = template_source_map_registration(template_output, composition) else {
        return;
    };

    let profiler = global_profiler();
    profiler.record_counter("atelier.profile.template_source_map_fragments", 1);
    let mut registry = SourceAtlasRegistry::source_map().with_route(registration.route);
    if let Some(fallback) = registration.fallback() {
        registry = registry.with_fallback(fallback);
    }
    record_source_map_registry(registry);
    profiler.record_counter("atelier.profile.source_map.registrations", 1);
    profiler.record_counter(
        "atelier.profile.source_map.generated_bytes",
        usize_to_counter(registration.generated_len()),
    );
    profiler.record_counter(registration.section.profile_counter(), 1);
}

/// Emit the common Source Atlas counters for a source-map registration.
///
/// Keeping this helper lane-shaped matters for #1634: source maps are not a
/// codegen afterthought. They are a requested plate with the same source,
/// target, coordinate, and fallback vocabulary as compiler, linter, and
/// typechecker lanes.
fn record_source_map_registry(registry: SourceAtlasRegistry) {
    let profiler = global_profiler();
    profiler.record_counter(registry.lane.profile_counter(), 1);
    registry.visit_requests(|request| {
        profiler.record_counter(request.profile_counter(), 1);
    });
    for fallback in registry.fallbacks.iter() {
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
            SourceMapRegistrationState::Deferred(SourceAtlasFallback::SourceMapCompositionSkipped)
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
    use vize_atelier_core::source_atlas::{SourceAtlasSource, SourceAtlasTarget};
    use vize_carton::String;

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
    }

    #[test]
    fn skipped_template_source_maps_keep_deferred_reason() {
        let output = template_result_with_map();
        let registration = template_source_map_registration(&output, SourceMapComposition::Skipped)
            .expect("source-map fragment should register");

        assert_eq!(
            registration.fallback(),
            Some(SourceAtlasFallback::SourceMapCompositionSkipped)
        );
    }
}

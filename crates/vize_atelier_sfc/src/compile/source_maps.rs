//! Source-map bookkeeping for SFC output assembly.

use crate::compile_template::TemplateBlockCompileResult;
use vize_atelier_core::{
    rendu::RenduRange,
    source_atlas::{SourceAtlasFallback, SourceAtlasPlate, SourceAtlasTarget},
    source_map::{SourceMapRegistration, SourceMapRegistrationState},
};
use vize_carton::profiler::global_profiler;

#[derive(Clone, Copy)]
pub(crate) enum SourceMapComposition {
    Composed,
    Skipped,
}

pub(crate) fn record_template_source_map_fact(
    template_output: &TemplateBlockCompileResult,
    composition: SourceMapComposition,
) {
    let Some(registration) = template_source_map_registration(template_output, composition) else {
        return;
    };

    let profiler = global_profiler();
    profiler.record_counter("atelier.profile.template_source_map_fragments", 1);
    profiler.record_counter(SourceAtlasPlate::SourceMap.profile_counter(), 1);
    profiler.record_counter(SourceAtlasTarget::SourceMap.profile_counter(), 1);
    profiler.record_counter("atelier.profile.source_map.registrations", 1);
    profiler.record_counter(
        "atelier.profile.source_map.generated_bytes",
        usize_to_counter(registration.generated_len()),
    );
    profiler.record_counter(registration.section.profile_counter(), 1);
    if let Some(fallback) = registration.fallback() {
        profiler.record_counter(fallback.profile_counter(), 1);
    }
}

fn template_source_map_registration(
    template_output: &TemplateBlockCompileResult,
    composition: SourceMapComposition,
) -> Option<SourceMapRegistration<'_>> {
    let fragment = template_output.source_map_fragment()?;
    Some(
        SourceMapRegistration::for_template_fragment(
            template_source_map_range(template_output),
            fragment,
            source_map_registration_state(composition),
        )
        .with_source_name("template.vue"),
    )
}

fn source_map_registration_state(composition: SourceMapComposition) -> SourceMapRegistrationState {
    match composition {
        SourceMapComposition::Composed => SourceMapRegistrationState::Composed,
        SourceMapComposition::Skipped => {
            SourceMapRegistrationState::Deferred(SourceAtlasFallback::SourceMapCompositionSkipped)
        }
    }
}

fn template_source_map_range(template_output: &TemplateBlockCompileResult) -> RenduRange {
    template_output
        .module_sections
        .map(|sections| sections.functions)
        .unwrap_or_else(|| RenduRange::new(0, template_output.code.len()))
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

//! Source-map bookkeeping for SFC output assembly.

use crate::compile_template::TemplateBlockCompileResult;
use vize_atelier_core::source_atlas::{SourceAtlasFallback, SourceAtlasPlate};
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
    if template_output.source_map_fragment().is_none() {
        return;
    }

    let profiler = global_profiler();
    profiler.record_counter("atelier.profile.template_source_map_fragments", 1);
    profiler.record_counter(SourceAtlasPlate::SourceMap.profile_counter(), 1);
    if matches!(composition, SourceMapComposition::Skipped) {
        profiler.record_counter(
            SourceAtlasFallback::SourceMapCompositionSkipped.profile_counter(),
            1,
        );
    }
}

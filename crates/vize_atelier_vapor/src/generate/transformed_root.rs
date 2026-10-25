//! Generate from an already transformed shared root without losing its origins.

use vize_atelier_core::{RootNode, options::BindingMetadata};
use vize_carton::{Allocator, String};
use vize_l0::{profile, profiler::global_profiler};

use super::{
    VaporGenerateExperimentalOptions, VaporGenerateOptions, VaporGenerateResult,
    generate_vapor_with_spans, spans::VaporSourceSpans,
};

/// Lower and emit a host-supplied, already transformed render root through the
/// shared Vapor IR pipeline. Its locations index the complete original source.
///
/// Unlike generating from a bare IR, this preserves static template and
/// structural-unit origins alongside expression locations. With maps disabled,
/// no origin collector is built. Scope attributes are inserted while lowering,
/// before template escaping and mapping, rather than rewriting generated text.
#[doc(hidden)]
pub fn generate_vapor_from_transformed_root<'a>(
    allocator: &'a Allocator,
    root: &RootNode<'a>,
    source: &'a str,
    binding_metadata: Option<&BindingMetadata>,
    scope_id: Option<&str>,
    options: VaporGenerateOptions,
    experimental: VaporGenerateExperimentalOptions<'_>,
) -> (VaporGenerateResult, Vec<String>, Vec<String>) {
    let (ir, diagnostics, warnings, templates) = profile!(
        "atelier.vapor.transformed_root.lower",
        crate::lower::transform_to_ir_with_spans(
            allocator,
            root,
            source,
            scope_id,
            experimental.source_map,
        )
    );
    let spans = profile!(
        "atelier.vapor.transformed_root.origins",
        templates.map(|templates| VaporSourceSpans::collect(root, templates))
    );
    let profiler = global_profiler();
    if profiler.is_enabled() {
        profiler.record_counter_enabled("atelier.vapor.transformed_root.roots", 1);
        profiler.record_counter_enabled(
            "atelier.vapor.transformed_root.origin_collections",
            u64::from(spans.is_some()),
        );
    }
    let generated = profile!(
        "atelier.vapor.transformed_root.generate",
        generate_vapor_with_spans(&ir, binding_metadata, options, experimental, spans.as_ref())
    );
    (generated, diagnostics, warnings)
}

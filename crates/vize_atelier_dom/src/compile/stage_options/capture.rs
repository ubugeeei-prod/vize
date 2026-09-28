//! Stage observation at the DOM product's L2 selection boundary.

use super::{emit_l2_captured, l2_binding_table_for, l2_emit_options};
use crate::options::DomCompilerOptions;
use vize_atelier_core::codegen::CodegenResultWithSections;
use vize_atelier_core::options::{CodegenOptions, CustomElementMatcher};
use vize_atelier_core::walk_probe::WalkCounts;
use vize_l0::dump::capture::CaptureSink;
use vize_l0::{Allocator, profile};

#[expect(
    clippy::too_many_arguments,
    reason = "independent compile inputs and capture"
)]
pub(in crate::compile) fn try_emit_l2_captured<C: CaptureSink>(
    allocator: &Allocator,
    source: &str,
    options: &DomCompilerOptions,
    codegen: &CodegenOptions,
    custom_elements: &CustomElementMatcher,
    hoisted_scope_id: Option<&str>,
    experimental_component_name: Option<&str>,
    pre_s2_walks: Option<WalkCounts>,
    capture: &mut C,
) -> Option<CodegenResultWithSections> {
    let binding_table = l2_binding_table_for(options);
    let emit_options = l2_emit_options(
        options,
        codegen,
        custom_elements,
        binding_table.as_ref(),
        hoisted_scope_id,
        experimental_component_name,
    )?;
    profile!(
        "atelier.dom.template.s2_codegen",
        emit_l2_captured(
            allocator,
            source,
            options.dialect,
            &emit_options,
            pre_s2_walks,
            false,
            capture,
        )
    )
    .ok()
}

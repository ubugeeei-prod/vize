//! Stage observation at the DOM product's L2 selection boundary.

use super::{l2_binding_table_for, l2_codegen_sections, l2_emit_options};
use crate::options::DomCompilerOptions;
use vize_atelier_core::codegen::{CodegenResult, CodegenResultWithSections};
use vize_atelier_core::options::{CodegenOptions, CustomElementMatcher};
use vize_atelier_core::walk_probe::WalkCounts;
use vize_l0::dump::capture::CaptureSink;
use vize_l0::{Allocator, profile, profiler::global_profiler};
use vize_l1_to_l2::{DomEmitOptions, EmitError, LegacyCaps};

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
    if !C::RECORDING {
        return super::try_emit_l2(
            allocator,
            source,
            options,
            codegen,
            custom_elements,
            hoisted_scope_id,
            experimental_component_name,
            pre_s2_walks,
        );
    }
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

/// Emit one DOM module through L2, with the SFC-only slot check when requested.
/// The product L2 emitter with a compile-time selected stage capture sink.
pub(in crate::compile) fn emit_l2_captured<C: CaptureSink>(
    allocator: &Allocator,
    source: &str,
    dialect: vize_l0::config::VueVersion,
    options: &DomEmitOptions<'_>,
    pre_s2_walks: Option<WalkCounts>,
    strict_slot_params: bool,
    capture: &mut C,
) -> Result<CodegenResultWithSections, EmitError> {
    if !C::RECORDING {
        return super::emit_l2(
            allocator,
            source,
            dialect,
            options,
            pre_s2_walks,
            strict_slot_params,
        );
    }
    let caps = LegacyCaps::for_version(dialect);
    let profiler = global_profiler();
    let emit = if profiler.is_enabled() {
        let observed = vize_l1_to_l2::emit_dom_source_observed_with_options_captured(
            allocator,
            source,
            caps,
            options,
            strict_slot_params,
            capture,
        )?;
        let budget = observed.budget;
        // P2-12b observes the compiler path that actually produced this DOM
        // module. The regular entry point keeps the observer uninstantiated,
        // preserving the no-observer cost law outside explicit profiling.
        profiler.record_counter_enabled("davinci.s2_dom.files", 1);
        profiler.record_counter_enabled(
            "davinci.s2_dom.transform.walks",
            u64::from(budget.transform.walks),
        );
        profiler.record_counter_enabled(
            "davinci.s2_dom.transform.passes",
            u64::from(budget.transform.passes),
        );
        profiler.record_counter_enabled("davinci.s2_dom.emit.walks", u64::from(budget.emit_walks));
        profiler
            .record_counter_enabled("davinci.s2_dom.emit.visits", u64::from(budget.emit_visits));
        profiler.record_counter_enabled(
            "davinci.s2_dom.total.walks",
            u64::from(budget.total_walks()),
        );
        let (pre_s2_walks, pre_s2_visits) = pre_s2_walks.map_or((0, 0), |counts| {
            (counts.total_walks(), counts.total_visits())
        });
        profiler.record_counter_enabled("davinci.s2_dom.pre_s2.walks", pre_s2_walks);
        profiler.record_counter_enabled("davinci.s2_dom.pre_s2.visits", pre_s2_visits);
        profiler.record_counter_enabled(
            "davinci.s2_dom.build.walks",
            pre_s2_walks + u64::from(budget.total_walks()),
        );
        observed.emit
    } else {
        vize_l1_to_l2::emit_dom_source_with_options_captured(
            allocator,
            source,
            caps,
            options,
            strict_slot_params,
            capture,
        )?
    };
    Ok(CodegenResultWithSections {
        result: CodegenResult {
            code: emit.code,
            preamble: emit.preamble,
            map: None,
        },
        // L2 records the same structural render-module boundaries as the
        // shipped emitter so SFC assembly can slice either lane identically.
        sections: Some(l2_codegen_sections(emit.sections)),
    })
}

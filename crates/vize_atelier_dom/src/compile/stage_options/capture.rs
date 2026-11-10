//! Stage observation at the DOM product's L2 selection boundary.

use super::{l2_binding_table_for, l2_codegen_sections, l2_emit_options};
use crate::options::DomCompilerOptions;
use vize_atelier_core::codegen::{CodegenResult, CodegenResultWithSections};
use vize_atelier_core::options::{CodegenOptions, CustomElementMatcher, TemplateSyntaxMode};
use vize_atelier_core::walk_probe::WalkCounts;
use vize_l0::dump::capture::CaptureSink;
use vize_l0::{Allocator, profile, profiler::active_profiler_is_enabled};
use vize_l4::targets::dom::artifact::{DomEmitOptions, EmitError};

pub(in crate::compile) struct SlotEmitPolicy {
    pub(in crate::compile) strict_slot_params: bool,
    pub(in crate::compile) no_slotted: bool,
    pub(in crate::compile) development: bool,
}

#[expect(
    clippy::too_many_arguments,
    reason = "independent compile inputs and capture"
)]
pub(in crate::compile) fn try_emit_l2_captured<C: CaptureSink>(
    allocator: &Allocator,
    source: &str,
    template_syntax: TemplateSyntaxMode,
    options: &DomCompilerOptions,
    codegen: &CodegenOptions,
    custom_elements: &CustomElementMatcher,
    hoisted_scope_id: Option<&str>,
    experimental_component_name: Option<&str>,
    pre_s2_walks: Option<WalkCounts>,
    strict_slot_params: bool,
    no_slotted: bool,
    development: bool,
    capture: &mut C,
) -> Option<CodegenResultWithSections> {
    if !C::RECORDING {
        return super::try_emit_l2(
            allocator,
            source,
            template_syntax,
            options,
            codegen,
            custom_elements,
            hoisted_scope_id,
            experimental_component_name,
            pre_s2_walks,
            strict_slot_params,
            no_slotted,
            development,
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
            template_syntax,
            &emit_options,
            pre_s2_walks,
            SlotEmitPolicy {
                strict_slot_params,
                no_slotted,
                development,
            },
            capture,
            &mut None,
        )
    )
    .ok()
}

/// Emit one DOM module through L2, with the SFC-only slot check when requested.
/// The product L2 emitter with a compile-time selected stage capture sink.
#[expect(
    clippy::too_many_arguments,
    reason = "existing emission inputs and optional borrowed validation"
)]
pub(in crate::compile) fn emit_l2_captured<C: CaptureSink>(
    allocator: &Allocator,
    source: &str,
    dialect: vize_l0::config::VueVersion,
    template_syntax: TemplateSyntaxMode,
    options: &DomEmitOptions<'_>,
    pre_s2_walks: Option<WalkCounts>,
    policy: SlotEmitPolicy,
    capture: &mut C,
    preflight: &mut Option<&mut dyn vize_l4::targets::dom::artifact::SurfacePreflight>,
) -> Result<CodegenResultWithSections, EmitError> {
    let SlotEmitPolicy {
        strict_slot_params,
        no_slotted,
        development,
    } = policy;
    if let Some(preflight) = preflight.as_deref_mut() {
        let observe = active_profiler_is_enabled();
        let observed =
            vize_l4::targets::dom::artifact::emit_dom_source_with_preflight_environment_and_syntax(
                allocator,
                source,
                super::dom_compat_caps(dialect, options.prefix_identifiers),
                options,
                strict_slot_params,
                no_slotted,
                preflight,
                observe,
                development,
                template_syntax,
                capture,
            );
        let (emit, budget) = if C::RECORDING {
            observed
        } else {
            super::noting_refusal(observed)
        }?;
        if let Some(budget) = budget {
            super::record_l2_budget(budget, pre_s2_walks);
        }
        return Ok(CodegenResultWithSections {
            result: CodegenResult {
                code: emit.code,
                preamble: emit.preamble,
                map: None,
            },
            sections: Some(l2_codegen_sections(emit.sections)),
        });
    }
    if !C::RECORDING {
        return super::noting_refusal(super::emit_l2(
            allocator,
            source,
            dialect,
            template_syntax,
            options,
            pre_s2_walks,
            strict_slot_params,
            no_slotted,
            development,
        ));
    }
    let caps = super::dom_compat_caps(dialect, options.prefix_identifiers);
    let emit = if active_profiler_is_enabled() {
        let observed =
            vize_l4::targets::dom::artifact::emit_dom_source_observed_with_options_captured_slot_scope_environment_and_syntax(
                allocator,
                source,
                caps,
                options,
                strict_slot_params,
                no_slotted,
                development,
                template_syntax,
                capture,
            )?;
        super::record_l2_budget(observed.budget, pre_s2_walks);
        observed.emit
    } else {
        vize_l4::targets::dom::artifact::emit_dom_source_with_options_captured_slot_scope_environment_and_syntax(
            allocator,
            source,
            caps,
            options,
            strict_slot_params,
            no_slotted,
            development,
            template_syntax,
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

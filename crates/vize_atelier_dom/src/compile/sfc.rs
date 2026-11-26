use super::{
    pipeline::{self, DomCompilePipelineOptions},
    selection, stage_options,
};
use crate::options::DomCompilerOptions;
use vize_atelier_core::{
    CompilerError, RootNode,
    codegen::{CodegenResult, CodegenResultWithSections},
    options::{
        CodegenExperimentalOptions, CodegenOptions, CustomElementMatcher, TemplateSyntaxMode,
    },
};
use vize_l0::dump::capture::{CaptureOutcome, CaptureSink, NoCapture};
use vize_l0::{Allocator, String, profile};

mod p_end;
mod selector;

#[expect(clippy::too_many_arguments, reason = "independent compile inputs")]
pub(super) fn compile_template_inner_for_sfc_with_sections<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
    codegen_experimental_options: CodegenExperimentalOptions,
    no_slotted: bool,
) -> (Vec<CompilerError>, CodegenResultWithSections) {
    compile_template_inner_for_sfc_with_sections_captured(
        allocator,
        source,
        options,
        template_syntax,
        hoisted_scope_id,
        custom_elements,
        codegen_options,
        codegen_experimental_options,
        no_slotted,
        true,
        &mut NoCapture,
        None,
    )
}

/// Whether a template source is one the shipped parser passes without a
/// notice the native levels cannot reproduce, so a lane that skips that
/// parser may compile it. This is the source test of the SFC fast path,
/// shared with the SSR direct lane.
pub(super) fn direct_template_source(
    source: &str,
    directive_contracts: &vize_l0::config::DirectiveContracts,
) -> bool {
    selector::l2_sfc_fast_path_supported_source(source)
        && !stage_options::source_may_contain_patterned_template_syntax(source)
        && !stage_options::source_may_contain_vize_directive_comment(source)
        && !(!directive_contracts.is_empty()
            && stage_options::source_may_bind_declared_directive(source, directive_contracts))
}

#[expect(
    clippy::too_many_arguments,
    reason = "independent compile inputs and capture"
)]
pub(super) fn compile_template_inner_for_sfc_with_sections_captured<'a, C: CaptureSink>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
    codegen_experimental_options: CodegenExperimentalOptions,
    no_slotted: bool,
    development: bool,
    capture: &mut C,
    mut preflight: Option<&mut dyn vize_l4::targets::dom::artifact::SurfacePreflight>,
) -> (Vec<CompilerError>, CodegenResultWithSections) {
    let codegen_opts = stage_options::codegen_options(&options, codegen_options.clone());
    let experimental_self_component = codegen_experimental_options.self_component;
    // A refusal here is recorded by the shared pipeline below, which
    // re-derives it from the same inputs; only the fast path's own outcomes
    // are recorded in this function.
    let use_l2_emit = stage_options::l2_emit_supported(
        &options,
        &codegen_opts,
        &custom_elements,
        template_syntax,
        stage_options::unprojectable_croquis(&options),
        pipeline::L2EmitSelection::RequireSections,
        experimental_self_component,
    ) && !stage_options::source_may_contain_patterned_template_syntax(source)
        && !stage_options::source_may_contain_vize_directive_comment(source)
        && !(!options.directive_contracts.is_empty()
            && stage_options::source_may_bind_declared_directive(
                source,
                &options.directive_contracts,
            ));

    let force_compat_sections = if use_l2_emit
        && !codegen_opts.source_map
        && selector::l2_sfc_fast_path_supported_source(source)
    {
        let binding_table = stage_options::l2_binding_table_for(&options);
        let l2_options = stage_options::l2_emit_options(
            &options,
            &codegen_opts,
            &custom_elements,
            binding_table.as_ref(),
            hoisted_scope_id.as_deref(),
            codegen_experimental_options.component_name.as_deref(),
        );
        if let Some(l2_options) = l2_options
            && let Ok(result) = profile!(
                "atelier.dom.template.s2_codegen_sfc_fast",
                stage_options::emit_l2_captured(
                    allocator,
                    source,
                    options.dialect,
                    template_syntax,
                    &l2_options,
                    None,
                    stage_options::SlotEmitPolicy {
                        strict_slot_params: true,
                        no_slotted,
                        development,
                    },
                    capture,
                    &mut preflight,
                )
            )
        {
            selection::record(Ok(()));
            capture.finish(|| CaptureOutcome::Accepted);
            return (Vec::new(), result);
        }
        true
    } else {
        // The shared pipeline hands L2 the same slot-outlet flag as the fast
        // path; L2 decodes character references in expressions itself.
        false
    };

    let mut pipeline_options = if force_compat_sections {
        DomCompilePipelineOptions::after_refusal_with_experimental_options(
            custom_elements,
            codegen_options,
            codegen_experimental_options,
        )
    } else {
        DomCompilePipelineOptions::require_sections_with_experimental_options(
            custom_elements,
            codegen_options,
            codegen_experimental_options,
        )
    };
    pipeline_options.no_slotted = no_slotted;
    // L2 runs before the shipped transform reports malformed slot parameters,
    // and the map composed into the component's must describe the module a
    // plain compile emits, not select another one.
    pipeline_options.sfc_assembly = true;
    pipeline_options.development = development;

    // This is the actual compatibility parse entry, not a final emitter
    // selection: the shared pipeline may still emit L2. A native-checked
    // invocation retains its result across late refusal without another scan.
    if let Some(preflight) = preflight
        && !preflight.check(None)
    {
        return (
            Vec::new(),
            CodegenResultWithSections {
                result: CodegenResult {
                    code: String::default(),
                    preamble: String::default(),
                    map: None,
                },
                sections: None,
            },
        );
    }
    let (_, errors, result) = super::inner::compile_template_inner_with_sections_captured(
        allocator,
        source,
        options,
        template_syntax,
        hoisted_scope_id,
        pipeline_options,
        capture,
    );
    (errors, result)
}

#[expect(clippy::too_many_arguments, reason = "independent compile inputs")]
pub(super) fn compile_template_inner_for_sfc<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
    codegen_experimental_options: CodegenExperimentalOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    let mut pipeline_options =
        DomCompilePipelineOptions::require_sections_compat_with_experimental_options(
            custom_elements,
            codegen_options,
            codegen_experimental_options,
        );
    // The SFC pipeline: a requested map is the one composed into the
    // component's, as the selected SFC compile composes it.
    pipeline_options.sfc_assembly = true;
    let (root, errors, result) = super::inner::compile_template_inner_with_sections_captured(
        allocator,
        source,
        options,
        template_syntax,
        hoisted_scope_id,
        pipeline_options,
        &mut NoCapture,
    );
    (root, errors, result.into_result())
}

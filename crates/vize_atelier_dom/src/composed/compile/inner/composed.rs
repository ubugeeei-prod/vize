// DO NOT EDIT. Generated from crates/vize_atelier_dom/src/compile/inner/composed.rs by `node tools/commands/generate/composed-copies.ts`.
// Edit the source and run the command again.
//! The compile of a template the SFC assembly maps.
//!
//! The SFC assembly composes the template's render map into the component's,
//! so the map must describe the module a compile without one emits: it is
//! additive and never selects another emitter. This compile makes the plain
//! compile's L2 attempt and keeps its module and its diagnostics; the shipped
//! transform and the compatibility generator then run
//! only to supply the map. A compile without a map never reaches this file.

use vize_atelier_core::composed::{
    CompilerError, RootNode,
    codegen::{
        CodegenResult, CodegenResultWithSections, locate_for_aliases,
        generate_with_sections_and_experimental_options_slot_scope_and_environment,
    },
    lane::transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id,
    options::{
        CodegenExperimentalOptions, CodegenOptions, CustomElementMatcher, TemplateSyntaxMode,
    },
    parser::parse_with_options_custom_elements_and_template_syntax,
};
use vize_l0::dump::capture::{CaptureOutcome, CaptureSink};
use vize_l0::{Allocator, String, profile, profiler::with_active_profiler};

use crate::composed::compile::selection::{self, DomLegacyReason};
use crate::composed::compile::{pipeline::DomCompilePipelineOptions, stage_options};
use crate::composed::options::DomCompilerOptions;

/// [`super::compile_template_inner_with_sections_captured`] for an SFC
/// template compiled with a map: the module a compile without one emits,
/// with the map composed into the component's.
#[inline(never)]
pub(in crate::composed::compile) fn compile_composed_template_with_sections_captured<'a, C: CaptureSink>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    pipeline_options: DomCompilePipelineOptions,
    capture: &mut C,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResultWithSections) {
    let DomCompilePipelineOptions {
        custom_elements,
        codegen_options,
        codegen_experimental_options,
        l2_emit_selection,
        no_slotted,
        strict_slot_params,
        development,
    } = pipeline_options;
    let parser_opts = stage_options::parser_options(&options);

    let (mut root, errors) = profile!(
        "atelier.dom.template.parse",
        parse_with_options_custom_elements_and_template_syntax(
            allocator,
            source,
            parser_opts,
            custom_elements.clone(),
            template_syntax,
        )
    );

    // Recoverable parser diagnostics ride along without gating codegen.
    if errors.iter().any(|e| !e.is_recoverable()) {
        selection::record(Err(DomLegacyReason::ParseError));
        capture.finish(|| CaptureOutcome::Rejected(String::from("parse-error")));
        let result = CodegenResult {
            code: String::default(),
            preamble: String::default(),
            map: None,
        };
        let result = CodegenResultWithSections {
            result,
            sections: None,
        };
        return (root, errors.to_vec(), result);
    }

    let has_croquis = stage_options::unprojectable_croquis(&options);
    let codegen_opts = stage_options::codegen_options(&options, codegen_options);
    let template_syntax_quirks = template_syntax.is_quirks();
    let l2_refusal = stage_options::l2_emit_refusal(
        &options,
        &codegen_opts,
        &custom_elements,
        template_syntax,
        has_croquis,
        l2_emit_selection,
        codegen_experimental_options.self_component,
    )
    .or_else(|| {
        stage_options::source_may_contain_patterned_template_syntax(source)
            .then_some(DomLegacyReason::PatternedSource)
    })
    .or_else(|| {
        stage_options::source_may_contain_vize_directive_comment(source)
            .then_some(DomLegacyReason::DirectiveComment)
    })
    .or_else(|| {
        (!options.directive_contracts.is_empty()
            && stage_options::source_may_bind_declared_directive(
                source,
                &options.directive_contracts,
            ))
        .then_some(DomLegacyReason::DeclaredDirective)
    });
    let inputs = MapInputs {
        custom_elements,
        template_syntax_quirks,
        hoisted_scope_id,
        codegen_opts,
        codegen_experimental_options,
        no_slotted,
        development,
    };
    match l2_refusal {
        None => {
            if let Some(native) = stage_options::try_emit_l2_captured(
                allocator,
                source,
                template_syntax,
                &options,
                &inputs.codegen_opts,
                &inputs.custom_elements,
                inputs.hoisted_scope_id.as_deref(),
                inputs
                    .codegen_experimental_options
                    .component_name
                    .as_deref(),
                None,
                strict_slot_params,
                no_slotted,
                development,
                capture,
            ) {
                selection::record(Ok(()));
                capture.finish(|| CaptureOutcome::Accepted);
                // The module and its diagnostics are the plain compile's.
                let (_, compat) = compat_map(allocator, &mut root, source, options, inputs);
                return (
                    root,
                    errors.to_vec(),
                    with_verified_map(native, compat),
                );
            }
            selection::record(Err(DomLegacyReason::EmitRefused));
        }
        Some(reason) => selection::record(Err(reason)),
    }

    let mut errors = errors.to_vec();
    capture.finish(|| {
        CaptureOutcome::Legacy(String::from(
            l2_refusal.unwrap_or(DomLegacyReason::EmitRefused).id(),
        ))
    });
    let (transform_errors, result) = compat_map(allocator, &mut root, source, options, inputs);
    // Transform diagnostics ride along with the parse errors, as in a plain
    // compile.
    errors.extend(transform_errors);
    (root, errors, result)
}

/// What the composed generator needs besides the tree.
struct MapInputs {
    custom_elements: CustomElementMatcher,
    template_syntax_quirks: bool,
    hoisted_scope_id: Option<String>,
    codegen_opts: CodegenOptions,
    codegen_experimental_options: CodegenExperimentalOptions,
    no_slotted: bool,
    development: bool,
}

/// Run the shipped transform over `root` and the composed generator over the
/// result, returning the transform's diagnostics and the generated module.
fn compat_map<'a>(
    allocator: &'a Allocator,
    root: &mut RootNode<'a>,
    source: &'a str,
    options: DomCompilerOptions,
    inputs: MapInputs,
) -> (Vec<CompilerError>, CodegenResultWithSections) {
    let transform_opts = stage_options::transform_options(&options);
    // Park the summary on the allocator so it shares the allocator lifetime.
    let script_tables = options
        .script_tables
        .map(|tables| allocator.alloc_owned(tables));
    let transform_errors = profile!(
        "atelier.dom.template.transform",
        transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id(
            allocator,
            root,
            transform_opts,
            script_tables,
            inputs.custom_elements,
            inputs.template_syntax_quirks,
            inputs.hoisted_scope_id,
        )
    );
    locate_for_aliases(root, source);
    let composed = profile!(
        "atelier.dom.template.codegen_compat",
        generate_with_sections_and_experimental_options_slot_scope_and_environment(
            root,
            inputs.codegen_opts,
            inputs.codegen_experimental_options,
            inputs.no_slotted,
            inputs.development,
        )
    );
    (
        transform_errors.into_iter().collect(),
        composed,
    )
}

/// The module of an additive mapped compile: `native`, carrying the composed
/// map only when the compatibility generator emits the same module. A
/// disagreement leaves the module unmapped instead of replacing it.
fn with_verified_map(
    mut native: CodegenResultWithSections,
    composed: CodegenResultWithSections,
) -> CodegenResultWithSections {
    let same = native.result.preamble == composed.result.preamble
        && native.result.code == composed.result.code
        && native.sections == composed.sections;
    if same {
        native.result.map = composed.result.map;
    } else {
        // Beside the selection counter, which still records `accepted`.
        with_active_profiler(|profiler| {
            if profiler.is_enabled() {
                profiler.record_counter_enabled("l2.dom.source_map.unmapped", 1);
            }
        });
    }
    native
}

use vize_atelier_core::{
    CompilerError, RootNode,
    codegen::{
        CodegenResult, CodegenResultWithSections, generate_with_sfc_member_columns,
        locate_for_aliases,
    },
    lane::transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id,
    options::TemplateSyntaxMode,
    parser::parse_with_options_custom_elements_and_template_syntax,
    walk_probe::WalkCounts,
};
use vize_l0::dump::capture::{CaptureOutcome, CaptureSink};
use vize_l0::{Allocator, String, profile, profiler::active_profiler_is_enabled};

use crate::compile::selection::{self, DomLegacyReason};
use crate::compile::{pipeline::DomCompilePipelineOptions, source_map, stage_options};
use crate::options::DomCompilerOptions;

pub(in crate::compile) fn compile_template_inner_with_sections_captured<'a, C: CaptureSink>(
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
        additive_source_map,
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

    // Parser-level diagnostics that are recoverable (e.g. duplicate
    // attribute — Vue keeps the first and continues) must NOT gate
    // codegen, or downstream callers see a 0-byte module reported as a
    // success. (#958) The recoverable diagnostics still ride along in
    // the returned errors vec so the caller can surface them as
    // warnings or test for parity.
    let fatal_count = errors.iter().filter(|e| !e.is_recoverable()).count();
    if fatal_count > 0 {
        selection::record(Err(DomLegacyReason::ParseError));
        capture.finish(|| CaptureOutcome::Rejected(String::from("parse-error")));
        let codegen_result = CodegenResult {
            code: String::default(),
            preamble: String::default(),
            map: None,
        };
        return (
            root,
            errors.to_vec(),
            CodegenResultWithSections {
                result: codegen_result,
                sections: None,
            },
        );
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
    let use_l2_emit = l2_refusal.is_none();
    let l2_custom_elements = custom_elements.clone();
    if use_l2_emit && !codegen_opts.source_map {
        if let Some(result) = stage_options::try_emit_l2_captured(
            allocator,
            source,
            template_syntax,
            &options,
            &codegen_opts,
            &l2_custom_elements,
            hoisted_scope_id.as_deref(),
            codegen_experimental_options.component_name.as_deref(),
            None,
            strict_slot_params,
            no_slotted,
            development,
            capture,
        ) {
            selection::record(Ok(()));
            capture.finish(|| CaptureOutcome::Accepted);
            return (root, errors.to_vec(), result);
        }
        selection::record(Err(DomLegacyReason::EmitRefused));
    } else if let Some(reason) = l2_refusal {
        selection::record(Err(reason));
    } else if additive_source_map {
        // Only a mapped compile whose L2 emit is not refused reaches this arm:
        // the two above take every compile without a map. The map is composed
        // into a module it must not change, so this makes the plain compile's
        // L2 attempt and keeps its module and its diagnostics. The shipped
        // transform and the compatibility generator then run only to supply
        // the map.
        if let Some(native) = stage_options::try_emit_l2_captured(
            allocator,
            source,
            template_syntax,
            &options,
            &codegen_opts,
            &l2_custom_elements,
            hoisted_scope_id.as_deref(),
            codegen_experimental_options.component_name.as_deref(),
            None,
            strict_slot_params,
            no_slotted,
            development,
            capture,
        ) {
            selection::record(Ok(()));
            capture.finish(|| CaptureOutcome::Accepted);
            let transform_opts = stage_options::transform_options(&options);
            let script_tables = options
                .script_tables
                .map(|tables| allocator.alloc_owned(tables));
            profile!(
                "atelier.dom.template.transform",
                transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id(
                    allocator,
                    &mut root,
                    transform_opts,
                    script_tables,
                    custom_elements,
                    template_syntax_quirks,
                    hoisted_scope_id,
                )
            );
            locate_for_aliases(&mut root, source);
            let compat = profile!(
                "atelier.dom.template.codegen_sourcemap_compat",
                generate_with_sfc_member_columns(
                    &root,
                    codegen_opts,
                    codegen_experimental_options,
                    no_slotted,
                    development,
                    additive_source_map,
                )
            );
            let mapped = source_map::with_verified_map(native, compat);
            return (root, errors.to_vec(), mapped);
        }
        selection::record(Err(DomLegacyReason::EmitRefused));
    }

    // An additive mapped compile attempted L2 above, as a plain compile does.
    let l2_emit_after_transform = (use_l2_emit && codegen_opts.source_map && !additive_source_map)
        .then(|| (options.clone(), hoisted_scope_id.clone()));
    let experimental_component_name = codegen_experimental_options.component_name.clone();
    let transform_opts = stage_options::transform_options(&options);
    // Park the summary on the allocator so it shares the allocator lifetime.
    let script_tables = options
        .script_tables
        .map(|tables| allocator.alloc_owned(tables));
    let profiling_l2 = use_l2_emit && active_profiler_is_enabled();
    let template_walk_before = profiling_l2.then(WalkCounts::snapshot);
    let transform_errors = profile!(
        "atelier.dom.template.transform",
        transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id(
            allocator,
            &mut root,
            transform_opts,
            script_tables,
            custom_elements,
            template_syntax_quirks,
            hoisted_scope_id,
        )
    );
    let template_walks = template_walk_before.map(|before| WalkCounts::snapshot().since(before));

    // Surface transform diagnostics (e.g. invalid expressions) alongside
    // parse errors instead of dropping them — the official compiler reports
    // both through the same `errors` channel.
    let mut errors = errors.to_vec();
    errors.extend(transform_errors);

    let l2_emit = l2_emit_after_transform.and_then(|(options, hoisted_scope_id)| {
        stage_options::try_emit_l2_captured(
            allocator,
            source,
            template_syntax,
            &options,
            &codegen_opts,
            &l2_custom_elements,
            hoisted_scope_id.as_deref(),
            experimental_component_name.as_deref(),
            template_walks,
            strict_slot_params,
            no_slotted,
            development,
            capture,
        )
    });
    let codegen_result = match l2_emit {
        // Only a mapped compile emits L2 after the transform.
        Some(result) => {
            locate_for_aliases(&mut root, source);
            source_map::attach_compat_map(
                &root,
                &codegen_opts,
                codegen_experimental_options,
                no_slotted,
                development,
                result,
            )
            .finish(capture)
        }
        None => {
            if codegen_opts.source_map {
                locate_for_aliases(&mut root, source);
                // An additive compile recorded its refusal before the transform.
                if use_l2_emit && !additive_source_map {
                    selection::record(Err(DomLegacyReason::EmitRefused));
                }
            }
            capture.finish(|| {
                CaptureOutcome::Legacy(String::from(
                    l2_refusal.unwrap_or(DomLegacyReason::EmitRefused).id(),
                ))
            });
            profile!(
                "atelier.dom.template.codegen_compat",
                generate_with_sfc_member_columns(
                    &root,
                    codegen_opts,
                    codegen_experimental_options,
                    no_slotted,
                    development,
                    additive_source_map,
                )
            )
        }
    };

    (root, errors, codegen_result)
}

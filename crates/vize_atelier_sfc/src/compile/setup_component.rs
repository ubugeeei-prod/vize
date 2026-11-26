//! Case 3 of SFC compilation: `<script setup>`, with an optional `<script>`.
//!
//! [`analyze`] does the target-neutral work: the setup's parse and facts, the
//! macro type information (imported type resolution and the declaration
//! merge), the macro analysis and its validation, and the setup binding
//! tables. [`emit`] does one target's work on top: the template-dependent
//! bindings, the template compile and the script assembly. A multi-target
//! compile analyzes once and emits per target; [`emit`] changes only the
//! context's `macros` and `bindings`, which such a compile restores between
//! targets.

use vize_atelier_core::lane::ScriptTables;
use vize_carton::{String, profile};
use vize_l0::dump::capture::StageCapture;

use crate::compile_script::lazy_hydration::LazyHydrationTransform;
use crate::compile_script::props::{is_valid_identifier, validate_macro_scope_for_descriptor};
use crate::compile_script::{SetupTrace, TemplateParts, compile_script_setup_inline_with_context};
use crate::compile_template::{
    compile_template_block_vapor, compile_template_block_with_capture, extract_template_parts,
    slice_template_parts, template_parts_full,
};
use crate::module_map::Runs;
use crate::script::ScriptCompileContext;
use crate::types::{
    BindingMetadata, BindingType, SfcCompileResult, SfcError, SfcScriptBlock,
    TemplateCompileOptions,
};

use super::bindings::{merge_normal_script_bindings, register_setup_enum_bindings};
use super::diagnostics::create_v_model_reactive_const_warning;
use super::helpers::{demote_v_model_reactive_const_bindings, is_ts_lang, trim_trailing_newlines};
use super::output_module::{finalize_output_mode, rewrite_client_render_for_sfc_main};
use super::prelude::{Outputs, Prelude, Target};
use super::setup_facts::SetupCompileAnalysis;
use super::{bindings, module_trace, normal_script, props_rest, script_context, setup_exports};
use super::{stage_capture, template_context::SfcTemplateContext};

/// The prepared texts of both script blocks.
pub(super) struct SetupSources {
    /// The `<script>` content kept beside the setup (types, imports, etc.).
    pub(super) normal_script_content: Option<String>,
    pub(super) lazy_hydration: Option<LazyHydrationTransform>,
    /// The `<script setup>` content every later stage reads.
    pub(super) content: String,
}

/// The source-map provenance of [`SetupSources`], which a target consumes.
#[derive(Clone, Default)]
pub(super) struct SetupRuns {
    pub(super) normal: Option<Runs>,
    pub(super) setup: Option<Runs>,
}

impl SetupSources {
    pub(super) fn new(
        prelude: &Prelude<'_>,
        script_setup: &SfcScriptBlock<'_>,
    ) -> (Self, SetupRuns) {
        let codegen_options = prelude.codegen_options;
        // Extract normal script content if present (for type definitions, imports, etc.)
        // When both <script> and <script setup> exist, normal script content should be preserved
        // (except for export default which is handled by script setup)
        // With a source map requested, each piece also carries its `.vue` provenance.
        let (normal_script_content, normal) = normal_script::extract_for_setup(
            prelude.descriptor,
            prelude.is_ts,
            codegen_options.source_map,
        );
        let (lazy_hydration, content, setup) = module_trace::prepared_script(
            script_setup,
            codegen_options.source_map,
            prelude.nuxt_page_meta,
            codegen_options.runtime_module_name.as_str(),
        );
        (
            Self {
                normal_script_content,
                lazy_hydration,
                content,
            },
            SetupRuns { normal, setup },
        )
    }
}

/// The target-neutral analysis of the setup.
pub(super) struct AnalyzedSetup<'a> {
    pub(super) setup: SetupCompileAnalysis<'a>,
    pub(super) source_is_ts: bool,
    pub(super) ctx: ScriptCompileContext,
    pub(super) script_bindings: BindingMetadata,
    pub(super) script_tables: ScriptTables,
    pub(super) dependencies: Vec<String>,
}

/// Analyze the setup. `is_vapor` holds for every target that emits from it.
pub(super) fn analyze<'a>(
    setup_arena: &'a vize_l0::Allocator,
    prelude: &Prelude<'_>,
    script_setup: &SfcScriptBlock<'_>,
    content: &'a str,
    template: &TemplateCompileOptions,
    is_vapor: bool,
) -> Result<AnalyzedSetup<'a>, SfcError> {
    let descriptor = prelude.descriptor;
    let setup = profile!(
        "atelier.sfc.script_setup.facts",
        SetupCompileAnalysis::new(setup_arena, content, (descriptor, template))
    )?;
    let setup_program = setup.program();
    setup_exports::validate(setup_program, content, descriptor)?;

    // ScriptCompileContext: macro span info and TypeScript type resolution.
    let source_is_ts = is_ts_lang(script_setup.lang.as_deref());
    let script_context::CollectedContext {
        context: mut ctx,
        imports: imported_types,
        dependencies,
    } = script_context::collect(
        descriptor,
        content,
        prelude.source_filename,
        prelude.type_project,
    )?;
    profile!(
        "atelier.sfc.script_context.analyze",
        ctx.analyze_for_target(setup_program, content, is_vapor)
    );
    validate_macro_scope_for_descriptor(&ctx, setup_program, descriptor)?;
    script_context::validate(&ctx, &imported_types, descriptor)?;

    let (mut script_bindings, script_tables) = setup.tables()?;
    if let Some(program) = setup_program {
        register_setup_enum_bindings(&mut script_bindings, program);
    }
    props_rest::merge_props_bindings(&mut script_bindings, &ctx);
    Ok(AnalyzedSetup {
        setup,
        source_is_ts,
        ctx,
        script_bindings,
        script_tables,
        dependencies,
    })
}

/// What one target's [`emit`] consumes.
pub(super) struct SetupInputs {
    pub(super) outputs: Outputs,
    pub(super) runs: SetupRuns,
    pub(super) script_bindings: BindingMetadata,
    pub(super) script_tables: ScriptTables,
    pub(super) dependencies: Vec<String>,
}

/// The setup analysis one target reads.
pub(super) struct SetupView<'s, 'a> {
    pub(super) script_setup: &'s SfcScriptBlock<'s>,
    pub(super) sources: &'s SetupSources,
    pub(super) setup: &'s SetupCompileAnalysis<'a>,
    pub(super) source_is_ts: bool,
}

/// Compile one target from the analyzed setup. Changes `ctx.macros` and
/// `ctx.bindings` and nothing else of the context.
pub(super) fn emit(
    prelude: &Prelude<'_>,
    target: &Target<'_>,
    view: SetupView<'_, '_>,
    ctx: &mut ScriptCompileContext,
    SetupInputs {
        outputs:
            Outputs {
                css,
                mut warnings,
                macro_artifacts,
            },
        runs:
            SetupRuns {
                normal: normal_runs,
                setup: mut setup_runs,
            },
        mut script_bindings,
        script_tables,
        dependencies: type_dependencies,
    }: SetupInputs,
    mut capture: Option<&mut StageCapture>,
) -> Result<SfcCompileResult, SfcError> {
    let SetupView {
        script_setup,
        sources,
        setup,
        source_is_ts,
    } = view;
    let descriptor = prelude.descriptor;
    let options = target.options;
    let codegen_options = prelude.codegen_options;
    let compiled_styles = &prelude.styles.compiled;
    let scope_id = prelude.styles.scope_id.as_str();
    let template_context: &SfcTemplateContext<'_> = &prelude.template_context;
    let (is_vapor, script_output) = (target.is_vapor, target.script_output);
    let script_setup_content = &sources.content;
    let setup_program = setup.program();
    let mut errors = Vec::new();
    let mut code = String::default();

    let mut synthetic_inline_dollar_emit = ctx.macros.define_emits.as_ref().is_some_and(|emits| {
        bindings::register_setup_emit_binding(
            &mut script_bindings,
            emits.binding_name.as_deref(),
            script_output.separates_template(),
        )
    });

    // Register bindings from normal <script> block.
    // When both <script> and <script setup> exist, top-level imports and
    // declarations from the normal script are accessible in the template.
    // This enables proper component resolution (e.g., `import { Form as PForm }`)
    // and identifier prefix resolution (avoiding incorrect `_ctx.` prefix).
    if let Some(script) = descriptor.script.as_ref() {
        let normal_script_bindings = bindings::ordinary(&script.content, script.lang.as_deref());
        merge_normal_script_bindings(&mut script_bindings, &normal_script_bindings);
        merge_normal_script_bindings(&mut ctx.bindings, &normal_script_bindings);
        synthetic_inline_dollar_emit &= !normal_script_bindings.bindings.contains_key("$emit");
    }

    let setup_css_module_names = compiled_styles
        .css_modules
        .iter()
        .filter(|module| {
            is_valid_identifier(&module.name) && !ctx.bindings.bindings.contains_key(&module.name)
        })
        .map(|module| module.name.clone())
        .collect::<Vec<_>>();
    for module_name in &setup_css_module_names {
        script_bindings
            .bindings
            .entry(module_name.clone())
            .or_insert(BindingType::SetupConst);
        ctx.bindings
            .bindings
            .entry(module_name.clone())
            .or_insert(BindingType::SetupConst);
    }

    // When v-model demotion rewrites `const` to `let`, the rewritten content
    // replaces the original for the inline compile below and the shared
    // pre-demotion AST is no longer valid for it.
    let mut demoted_setup_content: Option<String> = None;
    if let Some(template) = &descriptor.template {
        let demote_result = profile!(
            "atelier.sfc.script_setup.demote_v_model_reactive_consts",
            demote_v_model_reactive_const_bindings(
                &template.content,
                script_setup.lang.as_deref(),
                script_setup_content,
                ctx,
                &mut script_bindings,
            )
        );

        if let Some((rewritten, demoted_ids, starts)) = demote_result {
            setup_runs = module_trace::demoted(setup_runs, script_setup_content.len(), &starts);
            for binding_name in demoted_ids {
                warnings.push(create_v_model_reactive_const_warning(
                    script_setup,
                    &binding_name,
                ));
            }
            demoted_setup_content = Some(rewritten);
        }
    }
    bindings::copy_setup_state_bindings(
        &mut ctx.bindings,
        &script_bindings,
        synthetic_inline_dollar_emit,
    );
    // Compile template with bindings (if present) to get the render function
    let template_result = if let Some(template) = &descriptor.template {
        let template_allocator = vize_carton::pool::acquire();
        if is_vapor {
            Some(profile!(
                "atelier.sfc.template.vapor",
                compile_template_block_vapor(
                    &template_allocator,
                    template,
                    &options.template,
                    prelude.custom_elements,
                    template_context.block(
                        false,
                        Some(&bindings::for_vapor(&script_bindings, setup_program)),
                        None,
                    ),
                    prelude.template_syntax,
                    codegen_options,
                    capture.as_deref_mut(),
                )
            ))
        } else {
            // Also pass scope IDs to the client template compiler. Vue's runtime
            // normally propagates __scopeId, but wrapper components such as NuxtLink
            // can otherwise lose parent scoped attrs before the final DOM root.
            let mut template_opts = options.template.clone();
            if prelude.auto_ssr_css_vars {
                crate::css::transform::vars::inject_ssr_values(
                    &mut template_opts,
                    &descriptor.css_vars,
                    scope_id,
                    Some(&script_bindings),
                );
            }
            Some(profile!(
                "atelier.sfc.template.compile",
                compile_template_block_with_capture(
                    &template_allocator,
                    template,
                    &template_opts,
                    prelude.custom_elements,
                    template_context.block(
                        !script_output.separates_template(),
                        Some(&script_bindings),
                        Some(script_tables),
                    ),
                    prelude.template_syntax,
                    codegen_options,
                    capture.as_deref_mut(),
                )
            ))
        }
    } else {
        None
    };

    if let Some(Ok(template_output)) = &template_result {
        warnings.extend(template_output.warnings.clone());
    }

    // Extract template parts for inline mode (imports, hoisted, preamble, render_body)
    let (
        template_imports,
        template_hoisted,
        template_render_fn,
        template_render_fn_name,
        template_preamble,
        render_body,
    ) = match &template_result {
        Some(Ok(template_output)) => {
            let template_code = &template_output.code;
            if is_vapor || options.template.ssr || script_output.separates_template() {
                let (imports, hoisted, render_fn, render_fn_name) = profile!(
                    "atelier.sfc.template.extract_parts_full",
                    template_parts_full(template_output)
                );
                (
                    imports,
                    hoisted,
                    render_fn,
                    render_fn_name,
                    String::default(),
                    String::default(),
                )
            } else {
                let (imports, hoisted, preamble, body, render_fn_name) = profile!(
                    "atelier.sfc.template.extract_parts",
                    match &template_output.sections {
                        // Emission-recorded offsets: slice the sections out
                        // directly instead of re-scanning the module.
                        Some(sections) => slice_template_parts(template_code, sections),
                        None => extract_template_parts(template_code),
                    }
                );
                (
                    imports,
                    hoisted,
                    String::default(),
                    render_fn_name,
                    preamble,
                    body,
                )
            }
        }
        Some(Err(e)) => {
            errors.push(e.clone());
            stage_capture::unavailable(capture, "template-error");
            (
                String::default(),
                String::default(),
                String::default(),
                "",
                String::default(),
                String::default(),
            )
        }
        None => (
            String::default(),
            String::default(),
            String::default(),
            "",
            String::default(),
            String::default(),
        ),
    };

    // In separate client output the template function shares module scope with
    // the user's imports and declarations. Keep its name distinct from common
    // script bindings such as `import { render } from "./lib"`.
    let (template_render_fn, template_render_fn_name) = if script_output.separates_template()
        && !is_vapor
        && !options.template.ssr
        && !template_render_fn.is_empty()
    {
        (
            rewrite_client_render_for_sfc_main(&template_render_fn),
            "_sfc_render",
        )
    } else {
        (template_render_fn, template_render_fn_name)
    };

    // When demotion rewrote the content, the shared AST is stale for the
    // rewritten text; the inline compiler re-parses in that rare case.
    let (setup_content_for_inline, setup_program_for_inline) = match demoted_setup_content.as_ref()
    {
        Some(content) => (content.as_str(), None),
        None => (script_setup_content.as_str(), setup_program),
    };

    // Assemble script setup with either an inline render (`inline_template`) or
    // a separate render function plus a marked setup-state object (module mode),
    // sharing the same analyzed script context, macro lowering, and runtime props.
    let mut setup_trace = setup_runs.map(|runs| SetupTrace::new(runs, normal_runs));
    let script_result = profile!(
        "atelier.sfc.script_setup.inline_compile",
        compile_script_setup_inline_with_context(
            ctx,
            setup_content_for_inline,
            setup_program_for_inline,
            prelude.component_name,
            prelude.is_ts,
            source_is_ts,
            is_vapor,
            TemplateParts {
                imports: &template_imports,
                hoisted: &template_hoisted,
                render_fn: &template_render_fn,
                render_fn_name: template_render_fn_name,
                preamble: &template_preamble,
                render_body: &render_body,
                render_is_block: is_vapor,
                hydrates_as_vapor: options.template.vapor,
            },
            sources.normal_script_content.as_deref(),
            &descriptor.css_vars,
            &compiled_styles.css_modules,
            &setup_css_module_names,
            scope_id,
            scope_id,
            prelude.filename.ends_with(".art.vue"),
            options.template.is_prod,
            &crate::compile_script::runtime_imports::RuntimeModule::new(
                codegen_options.runtime_module_name.as_str()
            ),
            setup_trace.as_mut(),
        )
    )?;

    // The shared script-setup compiler generates the complete module, including
    // imports, hoists, component options, setup bindings, and render attachment.
    if let Some(transform) = sources.lazy_hydration.as_ref() {
        code.push_str(&transform.preamble);
    }
    let (script_at, output) = (code.len(), setup_trace.map(|trace| trace.output));
    code.push_str(&script_result.code);

    let rewrite = finalize_output_mode(&mut code, &mut warnings, options, codegen_options);
    trim_trailing_newlines(&mut code);

    Ok(SfcCompileResult {
        map: module_trace::module_map(
            &code,
            output,
            script_at,
            rewrite,
            descriptor,
            prelude.filename,
        ),
        code,
        css,
        errors,
        warnings,
        bindings: script_result.bindings,
        macro_artifacts,
        dependencies: type_dependencies,
    })
}

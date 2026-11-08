use std::borrow::Cow;

use vize_carton::{String, profile};

use crate::compile_script::erasure::{Erasure, elide_generated_imports};
use crate::compile_script::runtime_imports::RuntimeModule;
use crate::module_map::{Runs, edit_runs, replace_traced};
use crate::script::{
    ScriptCompileContext, transform_define_expose_destructured_props,
    transform_destructured_props_with_edits,
};
use crate::types::{CssModuleMapping, SfcError};

use super::super::super::props::WithDefaultsValues;
use super::super::super::{
    ScriptCompileResult, TemplateParts,
    function_mode::helpers::{collect_runtime_identifier_references, parse_setup_body},
    import_utils::{extract_import_identifiers, import_block_has_local_from},
};
use super::{
    component_output::emit_component_definition,
    hoist::separate_hoisted_consts,
    model::{ModelInfo, build_model_props_emits, collect_model_infos},
    preamble::{AuthoredHead, emit_preamble},
    props::build_props_emits,
    render::{SetupBindingInputs, emit_render_return},
    setup_emit::{css_var_values, emit_setup_body},
    trace::{SetupTrace, Tracer},
};

#[expect(clippy::too_many_arguments, reason = "independent compile inputs")]
pub(super) fn compile_script_setup_inline_body(
    mut ctx: ScriptCompileContext,
    component_name: &str,
    is_ts: bool,
    source_is_ts: bool,
    is_vapor: bool,
    template: TemplateParts<'_>,
    css_vars: &[Cow<'_, str>],
    css_modules: &[CssModuleMapping],
    setup_css_module_names: &[String],
    scope_id: &str,
    css_vars_id: &str,
    is_prod: bool,
    user_imports: Vec<String>,
    ts_declarations: Vec<String>,
    setup_code: String,
    mut output: Vec<u8>,
    preserved_normal_script: Option<String>,
    needs_merge_defaults: bool,
    mut with_defaults_values: Option<WithDefaultsValues>,
    has_define_model: bool,
    needs_merge_models: bool,
    has_define_slots: bool,
    needs_vapor_setup_context: bool,
    vapor_render_alias: Option<String>,
    is_async: bool,
    runtime: &RuntimeModule<'_>,
    trace: Option<&mut SetupTrace>,
) -> Result<ScriptCompileResult, SfcError> {
    let mut tracer = Tracer(trace);
    let rest_proxy = ctx
        .macros
        .props_destructure
        .as_ref()
        .map(|destructure| destructure.rest_proxy_declaration())
        .transpose()?
        .flatten();
    if rest_proxy.is_some() {
        runtime.push_import(&mut output, "createPropsRestProxy as _createPropsRestProxy");
    }
    // JavaScript from a TypeScript source: erase each authored piece, then
    // elide the imports the module does not use as values.
    let mut erasure = (source_is_ts && !is_ts).then(Erasure::default);
    if let Some(erasure) = erasure.as_mut() {
        erasure.macro_arguments(&mut ctx);
        if let Some(defaults) = with_defaults_values.as_mut() {
            for value in defaults.static_values.values_mut() {
                *value = erasure.expression(value);
            }
            erasure.optional_expression(&mut defaults.runtime_expression);
        }
    }
    transform_define_expose_destructured_props(&mut ctx)?;
    let has_css_vars = !css_vars.is_empty();
    let has_css_modules = !setup_css_module_names.is_empty();
    let needs_prop_type = false;

    let props_emits_buf = profile!(
        "atelier.script_inline.build_props_emits",
        build_props_emits(
            &ctx,
            is_ts,
            needs_prop_type,
            needs_merge_defaults,
            with_defaults_values.as_ref(),
            is_prod,
        )
    );

    let mut model_infos: Vec<ModelInfo> = profile!(
        "atelier.script_inline.collect_model_infos",
        collect_model_infos(&ctx)
    );
    if let Some(erasure) = erasure.as_mut() {
        for (_, _, _, options, runtime_options, _) in &mut model_infos {
            erasure.optional_expression(options);
            erasure.optional_expression(runtime_options);
        }
    }

    let model_props_emits_buf = profile!(
        "atelier.script_inline.build_model_props_emits",
        build_model_props_emits(
            &ctx,
            &model_infos,
            is_ts,
            needs_prop_type,
            needs_merge_defaults,
            with_defaults_values.as_ref(),
            is_prod,
        )
    );

    let transformed_setup: String = if let Some(ref destructure) = ctx.macros.props_destructure {
        let (code, edits) = profile!(
            "atelier.script_inline.transform_props_destructure",
            transform_destructured_props_with_edits(&setup_code, destructure)
        )?;
        if let Some(trace) = tracer.trace() {
            trace.setup_code = edits.map_or_else(Runs::default, |edits| {
                edit_runs(setup_code.len(), &edits).compose(&trace.setup_code)
            });
        }
        code
    } else {
        setup_code
    };
    let transformed_setup = match erasure.as_mut() {
        Some(erasure) => {
            let (code, runs) = erasure.statements(&transformed_setup, tracer.trace().is_some());
            if let Some(trace) = tracer.trace() {
                trace.setup_code =
                    runs.map_or_else(Runs::default, |runs| runs.compose(&trace.setup_code));
            }
            code
        }
        None => transformed_setup,
    };

    // One parse of the setup body serves hoisting and runtime references.
    let setup_allocator = oxc_allocator::Allocator::default();
    let setup_parse = parse_setup_body(&setup_allocator, &transformed_setup);
    let (hoisted, body) = profile!(
        "atelier.script_inline.separate_hoisted",
        separate_hoisted_consts(&transformed_setup, &setup_parse, &ctx)
    );
    let segment_runs = |trace: &SetupTrace, segments: &[(String, usize)]| -> Vec<Runs> {
        let slice = |(text, offset): &(String, usize)| trace.setup_code.slice(*offset, text.len());
        segments.iter().map(slice).collect()
    };
    let mut hoisted_runs = Vec::new();
    if let Some(trace) = tracer.trace() {
        hoisted_runs = segment_runs(trace, &hoisted);
        trace.body = segment_runs(trace, &body);
    }
    let hoisted_lines: Vec<String> = hoisted.into_iter().map(|(text, _)| text).collect();
    let setup_body_lines: Vec<String> = body.into_iter().map(|(text, _)| text).collect();
    let css_var_values = css_var_values(&ctx, css_vars, source_is_ts);

    let mut head = AuthoredHead::collect(
        &user_imports,
        preserved_normal_script.as_ref(),
        is_ts,
        &mut tracer,
    );
    let setup_return_imports = head.import_texts();
    let runtime_used_identifiers = if template.render_body.is_empty()
        && !template.render_fn.is_empty()
        && !setup_return_imports.is_empty()
    {
        Some(collect_runtime_identifier_references(&setup_parse))
    } else {
        None
    };
    // The render or setup-state return closes `setup`; it has no provenance.
    let mut render_return = Vec::new();
    emit_render_return(
        &mut render_return,
        &template,
        SetupBindingInputs {
            imports: &setup_return_imports,
            runtime_used_identifiers: runtime_used_identifiers.as_ref(),
        },
        is_ts,
        is_vapor,
        is_prod,
        vapor_render_alias.as_deref(),
        &ctx,
    );

    let (ts_declarations, declaration_runs) = {
        let runs = tracer
            .trace()
            .map(|trace| std::mem::take(&mut trace.ts_declarations))
            .unwrap_or_default();
        match erasure.as_mut() {
            Some(erasure) if !ts_declarations.is_empty() => {
                erase_declarations(erasure, &ts_declarations, &runs, tracer.trace().is_some())
            }
            _ => (ts_declarations, runs),
        }
    };

    if let Some(erasure) = erasure.as_mut() {
        if let Some((body, _)) = &head.normal_body
            && !body.trim().is_empty()
        {
            erasure.javascript(body);
        }
        // Nuxt rewrites `useSeoMeta` to `useHead` after this compile, so an
        // explicit `useHead` import must survive.
        let calls_use_seo_meta = transformed_setup.contains("useSeoMeta(")
            || head
                .normal_body
                .as_ref()
                .is_some_and(|(body, _)| body.contains("useSeoMeta("));
        if calls_use_seo_meta
            && setup_return_imports
                .iter()
                .any(|import| import_block_has_local_from(import, "#imports", "useHead"))
        {
            erasure.note("useHead");
        }
        let locals: Vec<Vec<String>> = setup_return_imports
            .iter()
            .map(|import| extract_import_identifiers(import))
            .collect();
        // SAFETY: the render return is ASCII glue around UTF-8 template code.
        let render_return = unsafe { std::str::from_utf8_unchecked(&render_return) };
        for generated in [render_return, template.hoisted, template.render_fn]
            .into_iter()
            .filter(|piece| !piece.is_empty())
        {
            erasure.javascript(generated);
        }
        // Erased macro arguments count where they are emitted.
        let mut options =
            Vec::with_capacity(props_emits_buf.len() + model_props_emits_buf.len() + 6);
        options.extend_from_slice(b"({\n");
        options.extend_from_slice(&props_emits_buf);
        options.extend_from_slice(&model_props_emits_buf);
        options.extend_from_slice(b"})");
        // SAFETY: macro arguments and ASCII glue are UTF-8.
        erasure.javascript(unsafe { std::str::from_utf8_unchecked(&options) });
        let macros = &ctx.macros;
        for call in [&macros.define_options, &macros.define_expose]
            .into_iter()
            .flatten()
        {
            erasure.javascript_expression(&call.args);
        }
        let model_options = model_infos.iter().filter_map(|info| info.4.as_ref());
        for value in css_var_values.iter().chain(model_options) {
            erasure.javascript_expression(value);
        }
        if erasure.is_complete() {
            head.elide(erasure, &locals);
        }
    }

    let generated_imports_end = emit_preamble(
        &mut output,
        &template,
        &head,
        &ts_declarations,
        &declaration_runs,
        needs_merge_defaults,
        has_define_model,
        needs_merge_models,
        has_define_slots,
        has_css_vars,
        has_css_modules,
        needs_vapor_setup_context,
        vapor_render_alias.as_deref(),
        is_vapor,
        is_ts,
        is_async,
        runtime,
        &mut tracer,
    );

    if !hoisted_lines.is_empty() {
        ensure_blank_line(&mut output);
    }
    for (index, line) in hoisted_lines.iter().enumerate() {
        tracer.copy(output.len(), hoisted_runs.get(index));
        output.extend_from_slice(line.as_bytes());
        output.push(b'\n');
    }

    let component_state = emit_component_definition(
        &mut output,
        &ctx,
        component_name,
        is_ts,
        is_vapor,
        is_async,
        needs_prop_type,
        needs_vapor_setup_context,
        head.has_default_export,
        &props_emits_buf,
        &model_props_emits_buf,
        &template,
        vapor_render_alias.as_deref(),
        css_modules,
        erasure.as_mut(),
    );

    if let Some(declaration) = rest_proxy {
        if let Some(erasure) = erasure.as_mut() {
            erasure.javascript(&declaration);
        }
        output.extend_from_slice(declaration.as_bytes());
        output.push(b'\n');
    }

    emit_setup_body(
        &mut output,
        &ctx,
        &model_infos,
        &setup_body_lines,
        source_is_ts,
        is_ts,
        is_vapor,
        is_async,
        css_vars,
        &css_var_values,
        scope_id,
        css_vars_id,
        is_prod,
        has_css_vars,
        setup_css_module_names,
        erasure.as_mut(),
        &mut tracer,
    );

    output.push(b'\n');
    output.extend_from_slice(render_return.as_slice());

    output.extend_from_slice(b"}\n");
    output.push(b'\n');
    if is_vapor && (component_state.has_options || head.has_default_export) {
        output.extend_from_slice(b"}))\n");
    } else if component_state.has_options || head.has_default_export || is_ts || is_vapor {
        output.extend_from_slice(b"})\n");
    } else {
        output.extend_from_slice(b"}\n");
    }

    // SAFETY: `output` is assembled from UTF-8 source slices, OXC-generated
    // strings, and ASCII glue emitted by this compiler. The buffer type is bytes
    // only because we need cheap `extend_from_slice` during script assembly. No
    // raw non-UTF-8 bytes are ever appended, so validating the whole script again
    // would only add work to the hot SFC compile path.
    #[expect(clippy::disallowed_types, reason = "unchecked UTF-8")]
    let output_str: std::string::String =
        unsafe { std::string::String::from_utf8_unchecked(output) };

    let final_code: String = if is_ts {
        annotate_event_params(&output_str, &mut tracer)
    } else if let Some(erasure) = erasure.as_ref().filter(|erasure| erasure.is_complete()) {
        match elide_generated_imports(&output_str, generated_imports_end, erasure) {
            Some((code, runs)) => {
                if let Some(trace) = tracer.trace() {
                    trace.output = runs.compose(&trace.output);
                }
                code
            }
            None => output_str.into(),
        }
    } else {
        output_str.into()
    };

    Ok(ScriptCompileResult {
        code: final_code,
        bindings: Some(ctx.bindings),
    })
}

/// Hoisted runtime declarations (static enums) as one erased block.
fn erase_declarations(
    erasure: &mut Erasure,
    declarations: &[String],
    runs: &[Runs],
    trace: bool,
) -> (Vec<String>, Vec<Runs>) {
    let mut joined = String::default();
    let mut joined_runs = Runs::default();
    for (index, declaration) in declarations.iter().enumerate() {
        if index > 0 {
            joined.push('\n');
        }
        if let Some(runs) = runs.get(index) {
            joined_runs.append(joined.len(), runs);
        }
        joined.push_str(declaration);
    }
    let (code, erased_runs) = erasure.statements(&joined, trace);
    let code: String = code.trim_end().into();
    let runs = erased_runs
        .filter(|_| !runs.is_empty())
        .map(|erased| erased.slice(0, code.len()).compose(&joined_runs));
    (vec![code], runs.into_iter().collect())
}

/// TypeScript output types the `$event` parameter of inline handlers.
fn annotate_event_params(code: &str, tracer: &mut Tracer<'_>) -> String {
    const EXPRESSION: (&str, &str) = ("$event => (", "($event: any) => (");
    const BLOCK: (&str, &str) = ("$event => {", "($event: any) => {");
    let Some(trace) = tracer.trace() else {
        let code = code.replace(EXPRESSION.0, EXPRESSION.1);
        return code.replace(BLOCK.0, BLOCK.1).into();
    };
    let (first, first_runs) = replace_traced(code, EXPRESSION.0, EXPRESSION.1);
    let (second, second_runs) = replace_traced(&first, BLOCK.0, BLOCK.1);
    trace.output = second_runs.compose(&first_runs.compose(&trace.output));
    second
}

fn ensure_blank_line(output: &mut Vec<u8>) {
    match output.as_slice() {
        bytes if bytes.ends_with(b"\n\n") => {}
        bytes if bytes.ends_with(b"\n") => output.push(b'\n'),
        _ => output.extend_from_slice(b"\n\n"),
    }
}

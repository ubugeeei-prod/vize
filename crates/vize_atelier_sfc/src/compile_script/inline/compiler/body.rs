use std::borrow::Cow;

use vize_carton::{String, profile};

use crate::module_map::{Runs, edit_runs, replace_traced};
use crate::script::{ScriptCompileContext, transform_destructured_props_with_edits};
use crate::types::{CssModuleMapping, SfcError};

use super::super::super::props::WithDefaultsValues;
use super::super::super::{
    ScriptCompileResult, TemplateParts,
    function_mode::helpers::collect_runtime_identifier_references,
    import_utils::import_block_has_local_from,
    typescript::{transform_typescript_to_js, transform_typescript_to_js_traced},
};
use super::{
    component_output::emit_component_definition,
    hoist::separate_hoisted_consts,
    model::{ModelInfo, build_model_props_emits, collect_model_infos},
    preamble::emit_preamble,
    props::build_props_emits,
    render::{SetupBindingInputs, emit_render_return},
    setup_emit::emit_setup_body,
    trace::{SetupTrace, Tracer},
};

#[expect(clippy::too_many_arguments, reason = "independent compile inputs")]
pub(super) fn compile_script_setup_inline_body(
    ctx: ScriptCompileContext,
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
    mut output: vize_carton::Vec<u8>,
    preserved_normal_script: Option<String>,
    needs_merge_defaults: bool,
    with_defaults_values: Option<WithDefaultsValues>,
    has_define_model: bool,
    needs_merge_models: bool,
    has_define_slots: bool,
    needs_vapor_setup_context: bool,
    vapor_render_alias: Option<String>,
    is_async: bool,
    trace: Option<&mut SetupTrace>,
) -> Result<ScriptCompileResult, SfcError> {
    let mut tracer = Tracer(trace);
    let has_css_vars = !css_vars.is_empty();
    let has_css_modules = !setup_css_module_names.is_empty();
    let needs_prop_type = false;
    let preamble = emit_preamble(
        &mut output,
        &template,
        &user_imports,
        &ts_declarations,
        preserved_normal_script.as_ref(),
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
        &mut tracer,
    );

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

    let model_infos: Vec<ModelInfo> = profile!(
        "atelier.script_inline.collect_model_infos",
        collect_model_infos(&ctx)
    );

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

    let (hoisted, body) = profile!(
        "atelier.script_inline.separate_hoisted",
        separate_hoisted_consts(&transformed_setup, &ctx)
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
        preamble.has_default_export,
        &props_emits_buf,
        &model_props_emits_buf,
        &template,
        vapor_render_alias.as_deref(),
        css_modules,
    );

    emit_setup_body(
        &mut output,
        &ctx,
        &model_infos,
        &setup_body_lines,
        source_is_ts,
        is_ts,
        is_async,
        css_vars,
        scope_id,
        css_vars_id,
        is_prod,
        has_css_vars,
        setup_css_module_names,
        &mut tracer,
    );

    output.push(b'\n');
    let runtime_used_identifiers = if template.render_body.is_empty()
        && !template.render_fn.is_empty()
        && !preamble.setup_return_imports.is_empty()
    {
        Some(collect_runtime_identifier_references(&transformed_setup))
    } else {
        None
    };
    emit_render_return(
        &mut output,
        &template,
        SetupBindingInputs {
            imports: &preamble.setup_return_imports,
            runtime_used_identifiers: runtime_used_identifiers.as_ref(),
        },
        is_ts,
        is_vapor,
        vapor_render_alias.as_deref(),
        &ctx,
    );

    output.extend_from_slice(b"}\n");
    output.push(b'\n');
    if is_vapor && (component_state.has_options || preamble.has_default_export) {
        output.extend_from_slice(b"}))\n");
    } else if component_state.has_options || preamble.has_default_export || is_ts || is_vapor {
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
        unsafe { std::string::String::from_utf8_unchecked(output.into_iter().collect()) };

    let final_code: String = if is_ts {
        annotate_event_params(&output_str, &mut tracer)
    } else if !source_is_ts {
        output_str.into()
    } else {
        let mut code = output_str;
        if should_preserve_nuxt_use_head_import(&code) {
            code.push_str("\nvoid useHead;\n");
        }
        profile!(
            "atelier.script_inline.ts_to_js",
            match tracer.trace() {
                Some(trace) => {
                    let (js, runs) = transform_typescript_to_js_traced(&code);
                    trace.output = runs.compose(&trace.output);
                    js
                }
                None => transform_typescript_to_js(&code),
            }
        )
    };

    Ok(ScriptCompileResult {
        code: final_code,
        bindings: Some(ctx.bindings),
    })
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

fn should_preserve_nuxt_use_head_import(code: &str) -> bool {
    code.contains("useSeoMeta(") && import_block_has_local_from(code, "#imports", "useHead")
}

fn ensure_blank_line(output: &mut vize_carton::Vec<u8>) {
    match output.as_slice() {
        bytes if bytes.ends_with(b"\n\n") => {}
        bytes if bytes.ends_with(b"\n") => output.push(b'\n'),
        _ => output.extend_from_slice(b"\n\n"),
    }
}

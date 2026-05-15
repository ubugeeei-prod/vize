use std::borrow::Cow;

use vize_carton::{profile, String};

use crate::script::{transform_destructured_props, ScriptCompileContext};
use crate::types::SfcError;

use super::super::super::{
    typescript::transform_typescript_to_js, ScriptCompileResult, TemplateParts,
};
use super::{
    component_output::emit_component_definition,
    hoist::separate_hoisted_consts,
    model::{build_model_props_emits, collect_model_infos},
    preamble::emit_preamble,
    props::build_props_emits,
    render::emit_render_return,
    setup_emit::emit_setup_body,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_script_setup_inline_body(
    ctx: ScriptCompileContext,
    component_name: &str,
    is_ts: bool,
    source_is_ts: bool,
    is_vapor: bool,
    template: TemplateParts<'_>,
    css_vars: &[Cow<'_, str>],
    scope_id: &str,
    user_imports: Vec<String>,
    ts_declarations: Vec<String>,
    setup_code: String,
    mut output: vize_carton::Vec<u8>,
    preserved_normal_script: Option<String>,
    needs_merge_defaults: bool,
    has_define_model: bool,
    has_define_slots: bool,
    needs_vapor_setup_context: bool,
    vapor_render_alias: Option<String>,
    is_async: bool,
) -> Result<ScriptCompileResult, SfcError> {
    let has_css_vars = !css_vars.is_empty();
    let needs_prop_type = false;
    let preamble = emit_preamble(
        &mut output,
        &template,
        &user_imports,
        &ts_declarations,
        preserved_normal_script.as_ref(),
        needs_merge_defaults,
        has_define_model,
        has_define_slots,
        has_css_vars,
        needs_vapor_setup_context,
        vapor_render_alias.as_deref(),
        is_vapor,
        is_ts,
        is_async,
    );

    let props_emits_buf = profile!(
        "atelier.script_inline.build_props_emits",
        build_props_emits(&ctx, is_ts, needs_prop_type, needs_merge_defaults)
    );

    let model_infos: Vec<(String, String, Option<String>)> = profile!(
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
        )
    );

    let transformed_setup: String = if let Some(ref destructure) = ctx.macros.props_destructure {
        profile!(
            "atelier.script_inline.transform_props_destructure",
            transform_destructured_props(&setup_code, destructure)
        )
    } else {
        setup_code
    };

    let (hoisted_lines, setup_body_lines) = profile!(
        "atelier.script_inline.separate_hoisted",
        separate_hoisted_consts(&transformed_setup, &ctx)
    );

    for line in &hoisted_lines {
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
        has_css_vars,
    );

    output.push(b'\n');
    emit_render_return(
        &mut output,
        &template,
        &preamble.setup_return_imports,
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

    #[allow(clippy::disallowed_types)]
    let output_str: std::string::String =
        unsafe { std::string::String::from_utf8_unchecked(output.into_iter().collect()) };

    let final_code: String = if is_ts || !source_is_ts {
        let mut code = output_str;
        if is_ts {
            code = code.replace("$event => (", "($event: any) => (");
            code = code.replace("$event => { ", "($event: any) => { ");
        }
        code.into()
    } else {
        profile!(
            "atelier.script_inline.ts_to_js",
            transform_typescript_to_js(&output_str)
        )
    };

    Ok(ScriptCompileResult {
        code: final_code,
        bindings: Some(ctx.bindings),
    })
}

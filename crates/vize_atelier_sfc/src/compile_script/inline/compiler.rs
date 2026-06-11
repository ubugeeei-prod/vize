//! Core inline script compilation logic.
//!
//! Contains the main `compile_script_setup_inline` function that handles
//! compilation of `<script setup>` with inline template mode.

mod await_transform;
mod body;
mod component_output;
mod hoist;
mod model;
mod parser;
mod preamble;
mod props;
mod render;
mod setup_emit;

use std::borrow::Cow;

use vize_carton::{String, ToCompactString, profile};

use crate::script::ScriptCompileContext;
use crate::types::{CssModuleMapping, SfcError};

use super::super::function_mode::contains_top_level_await;
use super::super::lazy_hydration::transform_lazy_hydration_macros;
use super::super::props::validate_props_destructure_default_types;
use super::super::{ScriptCompileResult, TemplateParts};
use body::compile_script_setup_inline_body;
use parser::parse_script_content;
use render::build_vapor_render_alias;

/// Compile script setup with inline template (Vue's inline template mode)
#[allow(clippy::too_many_arguments)]
pub fn compile_script_setup_inline(
    content: &str,
    component_name: &str,
    is_ts: bool,
    source_is_ts: bool,
    is_vapor: bool,
    template: TemplateParts<'_>,
    normal_script_content: Option<&str>,
    css_vars: &[Cow<'_, str>],
    scope_id: &str,
    filename: Option<&str>,
) -> Result<ScriptCompileResult, SfcError> {
    let transformed = transform_lazy_hydration_macros(content);
    let content = transformed
        .as_ref()
        .map(|result| result.code.as_str())
        .unwrap_or(content);
    let ctx = build_script_setup_context(content, normal_script_content, filename, source_is_ts);
    let mut result = compile_script_setup_inline_with_context(
        ctx,
        content,
        component_name,
        is_ts,
        source_is_ts,
        is_vapor,
        template,
        normal_script_content,
        css_vars,
        &[],
        &[],
        scope_id,
        scope_id,
        false,
    )?;
    if let Some(transformed) = transformed {
        let mut code = transformed.preamble;
        code.push_str(&result.code);
        result.code = code;
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_script_setup_inline_with_context(
    ctx: ScriptCompileContext,
    content: &str,
    component_name: &str,
    is_ts: bool,
    source_is_ts: bool,
    is_vapor: bool,
    template: TemplateParts<'_>,
    normal_script_content: Option<&str>,
    css_vars: &[Cow<'_, str>],
    css_modules: &[CssModuleMapping],
    setup_css_module_names: &[String],
    scope_id: &str,
    css_vars_id: &str,
    is_prod: bool,
) -> Result<ScriptCompileResult, SfcError> {
    // Extract user imports and setup lines from script content once; await detection
    // and output assembly share the same split.
    let (user_imports, setup_lines, ts_declarations) = profile!(
        "atelier.script_inline.parse_sections",
        parse_script_content(content, is_ts)
    );
    let setup_code: String = setup_lines.join("\n").into();

    // Use arena-allocated Vec for better performance
    let bump = vize_carton::Bump::new();
    let mut output: vize_carton::Vec<u8> = vize_carton::Vec::with_capacity_in(4096, &bump);

    // Store normal script content to add AFTER TypeScript transformation
    // This preserves type definitions that would otherwise be stripped
    let preserved_normal_script = normal_script_content
        .filter(|s| !s.is_empty())
        .map(|s| s.to_compact_string());

    // Check if we need mergeDefaults import (props destructure with defaults)
    // For type-based props (defineProps<{...}>()), defaults are inlined into the prop definitions
    // so mergeDefaults is NOT needed. Only runtime-based props (defineProps([...])) need it.
    let has_props_destructure = ctx.macros.props_destructure.is_some();
    let has_type_based_props = ctx
        .macros
        .define_props
        .as_ref()
        .is_some_and(|p| p.type_args.is_some());
    let needs_merge_defaults = has_props_destructure
        && !has_type_based_props
        && ctx
            .macros
            .props_destructure
            .as_ref()
            .map(|d| d.bindings.values().any(|b| b.default.is_some()))
            .unwrap_or(false);

    validate_props_destructure_default_types(&ctx, 0, content)?;

    let has_define_model = !ctx.macros.define_models.is_empty();
    // `mergeModels` is needed when defineModel coexists with a user-declared
    // defineProps (to merge props) or defineEmits (to merge emits).
    let needs_merge_models = has_define_model
        && (ctx.macros.define_props.is_some() || ctx.macros.define_emits.is_some());
    let has_define_slots = ctx.macros.define_slots.is_some();
    let needs_vapor_setup_context = is_vapor && !template.render_fn.is_empty();
    let vapor_render_alias = needs_vapor_setup_context
        .then(|| build_vapor_render_alias(content, normal_script_content, template.render_fn));

    // withAsyncContext import comes first if needed
    let is_async = contains_top_level_await(&setup_code, source_is_ts);
    if is_async {
        if is_vapor {
            if needs_vapor_setup_context {
                output.extend_from_slice(
                    b"import { withAsyncContext as _withAsyncContext, defineVaporComponent as _defineVaporComponent, getCurrentInstance as _getCurrentInstance, proxyRefs as _proxyRefs } from 'vue'\n",
                );
            } else {
                output.extend_from_slice(
                    b"import { withAsyncContext as _withAsyncContext, defineVaporComponent as _defineVaporComponent } from 'vue'\n",
                );
            }
        } else if is_ts {
            output.extend_from_slice(
                b"import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'\n",
            );
        } else {
            output.extend_from_slice(
                b"import { withAsyncContext as _withAsyncContext } from 'vue'\n",
            );
        }
    }

    compile_script_setup_inline_body(
        ctx,
        component_name,
        is_ts,
        source_is_ts,
        is_vapor,
        template,
        css_vars,
        css_modules,
        setup_css_module_names,
        scope_id,
        css_vars_id,
        is_prod,
        user_imports,
        ts_declarations,
        setup_code,
        output,
        preserved_normal_script,
        needs_merge_defaults,
        has_define_model,
        needs_merge_models,
        has_define_slots,
        needs_vapor_setup_context,
        vapor_render_alias,
        is_async,
    )
}

fn build_script_setup_context(
    content: &str,
    normal_script_content: Option<&str>,
    filename: Option<&str>,
    source_is_ts: bool,
) -> ScriptCompileContext {
    let mut ctx = profile!(
        "atelier.script_inline.context.new",
        ScriptCompileContext::new(content)
    );

    // Merge type definitions from normal <script> block so that
    // defineProps<TypeRef>() can resolve types defined there.
    if let Some(normal_src) = normal_script_content
        && !normal_src.is_empty()
    {
        profile!(
            "atelier.script_inline.collect_normal_types",
            ctx.collect_types_from(normal_src)
        );
    }
    if let Some(path) = filename {
        profile!(
            "atelier.script_inline.collect_setup_import_types",
            ctx.collect_imported_types_from_path(content, path, source_is_ts)
        );
        if let Some(normal_src) = normal_script_content
            && !normal_src.is_empty()
        {
            profile!(
                "atelier.script_inline.collect_normal_import_types",
                ctx.collect_imported_types_from_path(normal_src, path, source_is_ts)
            );
        }
    }
    profile!("atelier.script_inline.context.analyze", ctx.analyze());
    ctx
}

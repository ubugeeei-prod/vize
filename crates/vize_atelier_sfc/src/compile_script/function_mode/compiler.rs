//! Main compiler for function-mode script setup.
//!
//! Contains the `compile_script_setup` function which compiles `<script setup>` blocks
//! in function mode, where the setup function returns bindings for use by a separate
//! render function.

mod output;
mod returned;

use vize_carton::{String, ToCompactString};

use crate::script::{ScriptCompileContext, transform_destructured_props};
use crate::types::SfcError;

use self::output::{
    collect_model_names, emit_emits_definition, emit_expose, emit_model_bindings,
    emit_props_definition,
};
use self::returned::build_returned_bindings;
use super::super::ScriptCompileResult;
use super::super::artifacts::erase_artifact_macro_statements;
use super::super::lazy_hydration::transform_lazy_hydration_macros;
use super::super::props::validate_props_destructure_default_types;
use super::super::statement_sections::{ScriptSections, extract_script_sections};
use super::super::typescript::transform_typescript_to_js;
use super::helpers::{collect_runtime_identifier_references, is_reserved_word};
use super::imports::dedupe_imports;

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_script_setup_from_source(
    content: &str,
    component_name: &str,
    is_vapor: bool,
    preserve_types: bool,
    source_is_ts: bool,
    template_content: Option<&str>,
    normal_script_content: Option<&str>,
    filename: Option<&str>,
) -> Result<ScriptCompileResult, SfcError> {
    super::super::record_legacy_from_source_compile();
    let lazy_hydration_transform = transform_lazy_hydration_macros(content);
    let content = lazy_hydration_transform
        .as_ref()
        .map(|result| result.code.as_str())
        .unwrap_or(content);
    let erased_content = erase_artifact_macro_statements(content);
    let content = erased_content
        .as_ref()
        .map(|content| content.as_str())
        .unwrap_or(content);

    let mut ctx =
        super::source::build_context(content, normal_script_content, filename, source_is_ts);
    ctx.analyze();
    let sections = extract_script_sections(content, source_is_ts)
        .unwrap_or_else(|| fallback_script_sections(content));
    let mut result = compile_script_setup_with_context(
        content,
        component_name,
        is_vapor,
        preserve_types,
        source_is_ts,
        template_content,
        ctx,
        sections,
    )?;
    if let Some(transform) = lazy_hydration_transform {
        let mut code = transform.preamble;
        code.push_str(&result.code);
        result.code = code;
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_script_setup_with_context(
    content: &str,
    component_name: &str,
    is_vapor: bool,
    preserve_types: bool,
    source_is_ts: bool,
    template_content: Option<&str>,
    mut ctx: ScriptCompileContext,
    sections: ScriptSections,
) -> Result<ScriptCompileResult, SfcError> {
    validate_props_destructure_default_types(&ctx, 0, content)?;

    // Use arena-allocated Vec for better performance
    let bump = vize_carton::Bump::new();
    let mut output: vize_carton::Vec<u8> = vize_carton::Vec::with_capacity_in(4096, &bump);

    // Check if we have props destructure
    let has_props_destructure = ctx.macros.props_destructure.is_some();

    let (imports, setup_lines, _) = sections;

    // Check if we need PropType import (type-based defineProps in non-vapor TS mode)
    let needs_prop_type = preserve_types
        && !is_vapor
        && ctx
            .macros
            .define_props
            .as_ref()
            .is_some_and(|p| p.type_args.is_some());

    // Add Vapor-specific import or defineComponent import
    if is_vapor {
        output.extend_from_slice(
            b"import { defineVaporComponent as _defineVaporComponent } from 'vue'\n",
        );
    } else if needs_prop_type {
        output.extend_from_slice(
            b"import { defineComponent as _defineComponent, type PropType } from 'vue'\n",
        );
    } else {
        output.extend_from_slice(b"import { defineComponent as _defineComponent } from 'vue'\n");
    }

    // Add mergeDefaults import if props destructure has defaults
    let needs_merge_defaults = has_props_destructure
        && ctx
            .macros
            .props_destructure
            .as_ref()
            .map(|d| d.bindings.values().any(|b| b.default.is_some()))
            .unwrap_or(false);
    if needs_merge_defaults {
        output.extend_from_slice(b"import { mergeDefaults as _mergeDefaults } from 'vue'\n");
    }

    // Add useSlots import if defineSlots was used
    let has_define_slots = ctx.macros.define_slots.is_some();
    if has_define_slots {
        output.extend_from_slice(b"import { useSlots as _useSlots } from 'vue'\n");
    }

    // Add useModel import if defineModel was used
    let has_define_model = !ctx.macros.define_models.is_empty();
    if has_define_model {
        output.extend_from_slice(b"import { useModel as _useModel } from 'vue'\n");
    }

    // Output imports (filtering out type-only imports + dedupe)
    let deduped_imports = dedupe_imports(&imports, false);
    for import in &deduped_imports {
        output.extend_from_slice(import.as_bytes());
    }

    output.push(b'\n');

    // Add comment for props destructure
    if has_props_destructure {
        output.extend_from_slice(b"// Reactive Props Destructure (Vue 3.5+)\n\n");
    }

    // Start __sfc__ definition
    if is_vapor {
        output.extend_from_slice(b"const __sfc__ = /*@__PURE__*/_defineVaporComponent({\n");
    } else {
        output.extend_from_slice(b"const __sfc__ = /*@__PURE__*/_defineComponent({\n");
    }
    output.extend_from_slice(b"  __name: '");
    output.extend_from_slice(component_name.as_bytes());
    output.extend_from_slice(b"',\n");

    // Props definition - handle both regular defineProps and destructure
    emit_props_definition(
        &mut output,
        &ctx,
        has_props_destructure,
        needs_prop_type,
        preserve_types,
    );

    // Collect model names for props and emits
    let model_names: Vec<String> = collect_model_names(&ctx);

    // Add model props if defineModel was used (and no defineProps)
    if !model_names.is_empty() && ctx.macros.define_props.is_none() && !has_props_destructure {
        output.extend_from_slice(b"  props: {\n");
        for model_name in &model_names {
            output.extend_from_slice(b"    \"");
            output.extend_from_slice(model_name.as_bytes());
            output.extend_from_slice(b"\": {},\n");
        }
        output.extend_from_slice(b"  },\n");
    }

    // Emits definition - combine defineEmits and defineModel
    emit_emits_definition(&mut output, &ctx, &model_names);

    // Prepare setup code and detect top-level await (async setup)
    let setup_code = setup_lines.join("\n");
    let has_top_level_await = super::helpers::contains_top_level_await(&setup_code, source_is_ts);

    // Setup function
    if has_top_level_await {
        output.extend_from_slice(b"  async setup(__props, { expose: __expose, emit: __emit }) {\n");
    } else {
        output.extend_from_slice(b"  setup(__props, { expose: __expose, emit: __emit }) {\n");
    }

    // Always call __expose() - Vue runtime requires this for proper component initialization
    // If defineExpose has args, use those; otherwise call with no args
    emit_expose(&mut output, &ctx);

    // Collect emit binding name for inclusion in __returned__
    let emit_binding_name: Option<String> = ctx
        .macros
        .define_emits
        .as_ref()
        .and_then(|m| m.binding_name.as_deref().map(String::from));

    // defineEmits binding: const emit = __emit
    if let Some(ref binding_name) = emit_binding_name {
        output.extend_from_slice(b"  const ");
        output.extend_from_slice(binding_name.as_bytes());
        output.extend_from_slice(b" = __emit\n");
    }

    // defineProps binding: const props = __props (only if not destructured).
    // Function mode has a separate render function, so this binding remains
    // part of `__returned__` and is visible through `$setup`.
    if !has_props_destructure
        && let Some(ref props_macro) = ctx.macros.define_props
        && let Some(ref binding_name) = props_macro.binding_name
    {
        output.extend_from_slice(b"  const ");
        output.extend_from_slice(binding_name.as_bytes());
        output.extend_from_slice(b" = __props\n");
    }

    // defineSlots binding: const slots = _useSlots()
    if let Some(ref slots_macro) = ctx.macros.define_slots
        && let Some(ref binding_name) = slots_macro.binding_name
    {
        output.extend_from_slice(b"  const ");
        output.extend_from_slice(binding_name.as_bytes());
        output.extend_from_slice(b" = _useSlots()\n");
    }

    // defineModel bindings: const model = _useModel(__props, 'modelValue')
    // Collect model binding names for __returned__
    let model_binding_names = emit_model_bindings(&mut output, &ctx);

    let transformed_setup: String = if let Some(ref destructure) = ctx.macros.props_destructure {
        transform_destructured_props(&setup_code, destructure)?
    } else {
        setup_code.into()
    };

    // Indent the setup code
    for line in transformed_setup.lines() {
        if !line.trim().is_empty() {
            output.extend_from_slice(b"  ");
            output.extend_from_slice(line.as_bytes());
        }
        output.push(b'\n');
    }

    let runtime_used_identifiers = collect_runtime_identifier_references(&transformed_setup);

    // Generate __returned__ object
    let returned_bindings = build_returned_bindings(
        &mut ctx,
        has_props_destructure,
        &emit_binding_name,
        &imports,
        template_content,
        &runtime_used_identifiers,
        &model_binding_names,
    );

    let returned_props: Vec<String> = returned_bindings
        .iter()
        .map(|name| {
            if is_reserved_word(name) {
                let mut entry = String::with_capacity(name.len() * 2 + 4);
                entry.push('"');
                entry.push_str(name);
                entry.push_str("\": ");
                entry.push_str(name);
                entry
            } else {
                name.clone()
            }
        })
        .collect();

    output.extend_from_slice(b"  const __returned__ = { ");
    output.extend_from_slice(returned_props.join(", ").as_bytes());
    output.extend_from_slice(b" }\n");
    output.extend_from_slice(b"  Object.defineProperty(__returned__, '__isScriptSetup', { enumerable: false, value: true })\n");
    output.extend_from_slice(b"  return __returned__\n");

    output.extend_from_slice(b"  }\n\n");
    // Close the component definition
    if is_vapor {
        output.extend_from_slice(b"});\n"); // Close _defineVaporComponent(
    } else {
        output.extend_from_slice(b"});\n"); // Close _defineComponent(
    }

    // SAFETY: this byte buffer is a fast append target for valid UTF-8 script
    // fragments: original source ranges, OXC codegen output, and ASCII component
    // wrapper text. There is no API in this module that appends arbitrary binary
    // data, so the generated script remains valid UTF-8 by construction.
    // Avoiding a second full-buffer validation matters for large script-setup
    // blocks in function mode.
    #[allow(clippy::disallowed_types)]
    let output_str: std::string::String =
        unsafe { std::string::String::from_utf8_unchecked(output.into_iter().collect()) };

    // Transform TypeScript to JavaScript only when output is not TS.
    let final_code: String = if preserve_types {
        output_str.into()
    } else {
        transform_typescript_to_js(&output_str)
    };

    Ok(ScriptCompileResult {
        code: final_code,
        bindings: Some(ctx.bindings),
    })
}

fn fallback_script_sections(content: &str) -> ScriptSections {
    let setup_lines = content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(line.to_compact_string())
            }
        })
        .collect();
    (Vec::new(), setup_lines, Vec::new())
}

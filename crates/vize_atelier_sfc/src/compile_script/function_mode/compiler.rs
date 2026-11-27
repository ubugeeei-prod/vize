//! Main compiler for function-mode script setup.
//!
//! Contains the `compile_script_setup` function which compiles `<script setup>` blocks
//! in function mode, where the setup function returns bindings for use by a separate
//! render function.

use crate::compile_script::macros::runtime_erased_macro_names;
use vize_carton::string_builder::TextBuf;
use vize_carton::{FxHashSet, String, ToCompactString};

use crate::script::{
    PropsDestructuredBindings, ScriptCompileContext, TemplateUsedIdentifiers, define_model_name,
    resolve_template_used_identifiers, resolve_type_args, transform_destructured_props,
};
use crate::types::{BindingType, SfcError};

use super::super::ScriptCompileResult;
use super::super::artifacts::erase_artifact_macro_statements;
use super::super::import_utils::extract_import_identifiers;
use super::super::lazy_hydration::transform_lazy_hydration_macros;
use super::super::props::{
    add_null_to_runtime_type, extract_emit_names_from_type,
    normalize_destructure_default_value_with_factory as normalize_default, runtime_prop_key,
    validate_macro_scope_and_props,
};
use super::expose::emit_expose;
use super::helpers::{is_reserved_word, runtime_identifier_references};
use super::imports::{dedupe_imports, insert_imports};
use super::model::emit_model_bindings;
use super::prop_types::{prop_types_from_context, resolve_prop_type};
use crate::compile_script::erasure::Erasure;

/// Compile script setup content following Vue.js core format
pub fn compile_script_setup(
    content: &str,
    component_name: &str,
    is_vapor: bool,
    is_ts: bool,
    template_content: Option<&str>,
) -> Result<ScriptCompileResult, SfcError> {
    let runtime = super::super::runtime_imports::RuntimeModule::new("vue");
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

    let mut ctx = ScriptCompileContext::new(content);
    ctx.analyze();
    validate_macro_scope_and_props(&ctx, 0, content)?;
    let mut erasure = Erasure::for_javascript(!is_ts, &mut ctx);
    crate::script::transform_define_expose_destructured_props(&mut ctx)?;
    // Assembled in the heap buffer that becomes the returned code.
    let mut output = TextBuf::with_capacity(4096);

    // Check if we have props destructure
    let has_props_destructure = ctx.macros.props_destructure.is_some();
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

    let (imports, setup_lines, module_scope) = super::module_scope::extract(content);

    // Check if we need PropType import (type-based defineProps in non-vapor TS mode)
    let needs_prop_type = is_ts
        && !is_vapor
        && ctx
            .macros
            .define_props
            .as_ref()
            .is_some_and(|p| p.type_args.is_some());

    // Add Vapor-specific import or defineComponent import
    if is_vapor {
        runtime.push_import(&mut output, "defineVaporComponent as _defineVaporComponent");
    } else if needs_prop_type {
        runtime.push_import(
            &mut output,
            "defineComponent as _defineComponent, type PropType",
        );
    } else {
        runtime.push_import(&mut output, "defineComponent as _defineComponent");
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
        runtime.push_import(&mut output, "mergeDefaults as _mergeDefaults");
    }

    // Add useSlots import if defineSlots was used
    let has_define_slots = ctx.macros.define_slots.is_some();
    if has_define_slots {
        runtime.push_import(&mut output, "useSlots as _useSlots");
    }

    // Add useModel import if defineModel was used
    let has_define_model = !ctx.macros.define_models.is_empty();
    if has_define_model {
        runtime.push_import(&mut output, "useModel as _useModel");
    }

    // Imports (type-only ones filtered, deduplicated) are placed here last.
    let imports_at = output.len();
    super::module_scope::emit_module_scope(&mut output, &module_scope, erasure.as_mut());

    output.push('\n');

    // Add comment for props destructure
    if has_props_destructure {
        output.push_str("// Reactive Props Destructure (Vue 3.5+)\n\n");
    }

    // Start __sfc__ definition
    if is_vapor {
        output.push_str("const __sfc__ = /*@__PURE__*/_defineVaporComponent({\n");
    } else {
        output.push_str("const __sfc__ = /*@__PURE__*/_defineComponent({\n");
    }
    crate::compile_script::component_name::push_component_name_option(&mut output, component_name);

    // Props definition - handle both regular defineProps and destructure
    emit_props_definition(
        &mut output,
        &ctx,
        has_props_destructure,
        needs_prop_type,
        is_ts,
    );

    // Collect model names for props and emits
    let model_names: Vec<String> = collect_model_names(&ctx);

    // Add model props if defineModel was used (and no defineProps)
    if !model_names.is_empty() && ctx.macros.define_props.is_none() && !has_props_destructure {
        output.push_str("  props: {\n");
        for model_name in &model_names {
            output.push_str("    \"");
            output.push_str(model_name);
            output.push_str("\": {},\n");
        }
        output.push_str("  },\n");
    }

    // Emits definition - combine defineEmits and defineModel
    emit_emits_definition(&mut output, &ctx, &model_names);

    // Prepare setup code and detect top-level await (async setup)
    let setup_code = setup_lines.join("\n");
    let has_top_level_await = super::helpers::contains_top_level_await(&setup_code, is_ts);

    // Setup function
    if has_top_level_await {
        output.push_str("  async setup(__props, { expose: __expose, emit: __emit }) {\n");
    } else {
        output.push_str("  setup(__props, { expose: __expose, emit: __emit }) {\n");
    }

    // Always call __expose() - Vue runtime requires this for proper component initialization
    // If defineExpose has args, use those; otherwise call with no args
    emit_expose(&mut output, &ctx);
    if let Some(declaration) = rest_proxy {
        output.push_str("  ");
        output.push_str(&declaration);
        output.push('\n');
    }

    // Collect emit binding name for inclusion in __returned__
    let emit_binding_name: Option<String> = ctx
        .macros
        .define_emits
        .as_ref()
        .and_then(|m| m.binding_name.as_deref().map(String::from));

    // defineEmits binding: const emit = __emit
    if let Some(ref binding_name) = emit_binding_name {
        output.push_str("  const ");
        output.push_str(binding_name);
        output.push_str(" = __emit\n");
    }

    // Collect props binding for exclusion from __returned__ (props themselves shouldn't be in returned)
    let mut props_binding_names: FxHashSet<String> = FxHashSet::default();

    // defineProps binding: const props = __props (only if not destructured)
    if !has_props_destructure
        && let Some(ref props_macro) = ctx.macros.define_props
        && let Some(ref binding_name) = props_macro.binding_name
    {
        output.push_str("  const ");
        output.push_str(binding_name);
        output.push_str(" = __props\n");
        props_binding_names.insert(String::from(binding_name.as_str()));
    }

    // defineSlots binding: const slots = _useSlots()
    if let Some(ref slots_macro) = ctx.macros.define_slots
        && let Some(ref binding_name) = slots_macro.binding_name
    {
        output.push_str("  const ");
        output.push_str(binding_name);
        output.push_str(" = _useSlots()\n");
    }

    // defineModel bindings: const model = _useModel(__props, 'modelValue')
    // Collect model binding names for __returned__
    let model_binding_names = emit_model_bindings(&mut output, &ctx, erasure.as_mut());

    let transformed_setup: String = if let Some(ref destructure) = ctx.macros.props_destructure {
        transform_destructured_props(&setup_code, destructure)?
    } else {
        setup_code.into()
    };
    let transformed_setup = Erasure::erase_statements(erasure.as_mut(), transformed_setup);

    // Indent the setup code
    for line in transformed_setup.lines() {
        if !line.trim().is_empty() {
            output.push_str("  ");
            output.push_str(line);
        }
        output.push('\n');
    }

    let runtime_used_identifiers = runtime_identifier_references(&transformed_setup);

    // Generate __returned__ object
    let returned_bindings = build_returned_bindings(
        &mut ctx,
        has_props_destructure,
        &props_binding_names,
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

    output.push_str("  const __returned__ = { ");
    output.push_str(&returned_props.join(", "));
    output.push_str(" }\n");
    output.push_str("  Object.defineProperty(__returned__, '__isScriptSetup', { enumerable: false, value: true })\n");
    output.push_str("  return __returned__\n");

    output.push_str("  }\n\n");
    // Close the component definition
    if is_vapor {
        output.push_str("});\n"); // Close _defineVaporComponent(
    } else {
        output.push_str("});\n"); // Close _defineComponent(
    }

    let imports = dedupe_imports(&imports, false);
    insert_imports(&mut output, imports_at, &imports, erasure);
    let mut final_code = output.into_compact();
    if let Some(transform) = lazy_hydration_transform {
        let mut code = transform.preamble;
        code.push_str(&final_code);
        final_code = code;
    }

    Ok(ScriptCompileResult {
        code: final_code,
        bindings: Some(ctx.bindings),
    })
}

/// Emit props definition to the output buffer.
///
/// Handles regular defineProps, destructured props with defaults, and type-based props.
fn emit_props_definition(
    output: &mut TextBuf,
    ctx: &ScriptCompileContext,
    has_props_destructure: bool,
    needs_prop_type: bool,
    _is_ts: bool,
) {
    if let (true, Some(destructure)) =
        (has_props_destructure, ctx.macros.props_destructure.as_ref())
    {
        // Check if there are any defaults
        let has_defaults = destructure.bindings.values().any(|b| b.default.is_some());

        if has_defaults {
            // Use mergeDefaults format: _mergeDefaults(runtimeProps, { prop2: default })
            // Get the original props argument from defineProps (or generate from type args)
            let original_props: String = if let Some(ref props_macro) = ctx.macros.define_props {
                if let Some(ref type_args) = props_macro.type_args {
                    let prop_types = prop_types_from_context(ctx, type_args);
                    if prop_types.is_empty() {
                        if let Some(ref destructure) = ctx.macros.props_destructure {
                            destructured_props_runtime_decl(destructure)
                        } else {
                            "{}".to_compact_string()
                        }
                    } else {
                        let mut names: Vec<_> =
                            prop_types.iter().map(|(n, _)| n.as_str()).collect();
                        names.sort();
                        let mut s = String::from("{ ");
                        for (i, name) in names.iter().enumerate() {
                            if i > 0 {
                                s.push_str(", ");
                            }
                            let key = runtime_prop_key(name);
                            s.push_str(key.as_str());
                            s.push_str(": {}");
                        }
                        s.push_str(" }");
                        s
                    }
                } else if !props_macro.args.is_empty() {
                    String::from(props_macro.args.as_str())
                } else {
                    "[]".to_compact_string()
                }
            } else {
                "[]".to_compact_string()
            };

            output.push_str("  props: /*@__PURE__*/_mergeDefaults(");
            output.push_str(&original_props);
            output.push_str(", {\n");

            for (key, binding) in &destructure.bindings {
                if let Some(ref default_val) = binding.default {
                    output.push_str("  ");
                    output.push_str(key);
                    output.push_str(": ");
                    let default_val = normalize_default(default_val, binding.default_needs_factory);
                    output.push_str(&default_val);
                    output.push('\n');
                }
            }
            output.push_str("}),\n");
        } else {
            // No defaults - just use the original props array
            if let Some(ref props_macro) = ctx.macros.define_props
                && !props_macro.args.is_empty()
            {
                output.push_str("  props: ");
                output.push_str(&props_macro.args);
                output.push_str(",\n");
            } else if let Some(ref props_macro) = ctx.macros.define_props
                && props_macro.type_args.is_some()
            {
                let prop_types = props_macro
                    .type_args
                    .as_ref()
                    .map(|type_args| prop_types_from_context(ctx, type_args))
                    .unwrap_or_default();
                if prop_types.is_empty() {
                    output.push_str("  props: ");
                    output.push_str(&destructured_props_runtime_decl(destructure));
                    output.push_str(",\n");
                } else {
                    output.push_str("  props: {\n");
                    let mut names: Vec<_> = prop_types.iter().map(|(n, _)| n.as_str()).collect();
                    names.sort();
                    for name in names {
                        output.push_str("    ");
                        let key = runtime_prop_key(name);
                        output.push_str(&key);
                        output.push_str(": {},\n");
                    }
                    output.push_str("  },\n");
                }
            }
        }
    } else if let Some(ref props_macro) = ctx.macros.define_props {
        if let Some(ref type_args) = props_macro.type_args {
            // For type-based props, extract full prop definitions
            let prop_types = prop_types_from_context(ctx, type_args);
            if !prop_types.is_empty() {
                output.push_str("  props: {\n");
                // Sort props for deterministic output
                let mut sorted_props: Vec<_> = prop_types.iter().collect();
                sorted_props.sort_by(|a, b| a.0.cmp(&b.0));
                for (name, prop_type) in sorted_props {
                    let resolved_js_type = resolve_prop_type(prop_type, ctx);
                    let runtime_js_type =
                        add_null_to_runtime_type(&resolved_js_type, prop_type.nullable);
                    output.push_str("    ");
                    let key = runtime_prop_key(name);
                    output.push_str(&key);
                    output.push_str(": { type: ");
                    output.push_str(&runtime_js_type);
                    if needs_prop_type && let Some(ref ts_type) = prop_type.ts_type {
                        if resolved_js_type == "null" {
                            output.push_str(" as unknown as PropType<");
                        } else {
                            output.push_str(" as PropType<");
                        }
                        // Normalize multi-line types to single line
                        let normalized: String =
                            String::from(ts_type.split_whitespace().collect::<Vec<_>>().join(" "));
                        output.push_str(&normalized);
                        output.push('>');
                    }
                    if prop_type.optional {
                        output.push_str(", required: false");
                    }
                    if prop_type.skip_check {
                        output.push_str(", skipCheck: true");
                    }
                    output.push_str(" },\n");
                }
                output.push_str("  },\n");
            }
        } else if !props_macro.args.is_empty() {
            output.push_str("  props: ");
            output.push_str(&props_macro.args);
            output.push_str(",\n");
        }
    }
}

fn destructured_props_runtime_decl(destructure: &PropsDestructuredBindings) -> String {
    let mut decl = String::from("{ ");
    for (i, key) in destructure.keys.iter().enumerate() {
        if i > 0 {
            decl.push_str(", ");
        }
        decl.push_str(key.as_str());
        decl.push_str(": {}");
    }
    decl.push_str(" }");
    decl
}

/// Collect model names from defineModel calls.
fn collect_model_names(ctx: &ScriptCompileContext) -> Vec<String> {
    ctx.macros
        .define_models
        .iter()
        .map(|m| define_model_name(ctx.source.as_str(), m))
        .collect()
}

/// Emit emits definition to the output buffer.
///
/// Combines defineEmits and defineModel emit events.
fn emit_emits_definition(output: &mut TextBuf, ctx: &ScriptCompileContext, model_names: &[String]) {
    let mut all_emits: Vec<String> = Vec::new();

    // Add emits from defineEmits
    if let Some(ref emits_macro) = ctx.macros.define_emits {
        if let Some(ref type_args) = emits_macro.type_args {
            let resolved_type_args =
                resolve_type_args(type_args, &ctx.interfaces, &ctx.type_aliases);
            let emit_names = extract_emit_names_from_type(&resolved_type_args);
            all_emits.extend(emit_names);
        } else if !emits_macro.args.is_empty() {
            // Runtime args - we'll output separately
        }
    }

    // Add update:modelValue emits from defineModel
    for model_name in model_names {
        let mut name = String::with_capacity(7 + model_name.len());
        name.push_str("update:");
        name.push_str(model_name);
        all_emits.push(name);
    }

    // Output emits
    if !all_emits.is_empty() {
        output.push_str("  emits: [");
        for (i, name) in all_emits.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            output.push('"');
            output.push_str(name);
            output.push('"');
        }
        output.push_str("],\n");
    } else if let Some(ref emits_macro) = ctx.macros.define_emits
        && !emits_macro.args.is_empty()
    {
        output.push_str("  emits: ");
        output.push_str(&emits_macro.args);
        output.push_str(",\n");
    }
}

/// Build the list of bindings to include in `__returned__`.
///
/// Filters out compiler macros, destructured props, props bindings, and typed props.
/// Includes imported identifiers used in the template.
#[expect(clippy::too_many_arguments, reason = "independent compile inputs")]
fn build_returned_bindings(
    ctx: &mut ScriptCompileContext,
    _has_props_destructure: bool,
    props_binding_names: &FxHashSet<String>,
    emit_binding_name: &Option<String>,
    imports: &[String],
    template_content: Option<&str>,
    runtime_used_identifiers: &FxHashSet<String>,
    _model_binding_names: &[String],
) -> Vec<String> {
    // Compiler macros preset - these are compile-time only and should not be in __returned__
    let compiler_macros: FxHashSet<&str> = runtime_erased_macro_names().collect();

    // Collect destructured prop local names to exclude from __returned__
    let destructured_prop_locals: FxHashSet<String> = ctx
        .macros
        .props_destructure
        .as_ref()
        .map(|d| {
            d.bindings
                .values()
                .map(|b| String::from(b.local.as_str()))
                .collect()
        })
        .unwrap_or_default();

    // Collect prop names from type-based defineProps to exclude from __returned__
    let typed_prop_names: FxHashSet<String> = ctx
        .macros
        .define_props
        .as_ref()
        .and_then(|p| p.type_args.as_ref())
        .map(|type_args| {
            prop_types_from_context(ctx, type_args)
                .iter()
                .map(|(n, _)| String::from(n.as_str()))
                .collect()
        })
        .unwrap_or_default();

    let imported_identifier_set: FxHashSet<String> = imports
        .iter()
        .flat_map(|import| extract_import_identifiers(import).into_iter())
        .filter(|name| !compiler_macros.contains(name.as_str()))
        .collect();

    // Generate __returned__ object
    let mut returned_bindings: Vec<String> = ctx
        .bindings
        .bindings
        .keys()
        .filter(|name| {
            // Exclude compiler macros, destructured props, props bindings, and typed props
            !compiler_macros.contains(name.as_str())
                && !destructured_prop_locals.contains(*name)
                && !props_binding_names.contains(*name)
                && !typed_prop_names.contains(*name)
                && (!imported_identifier_set.contains(*name)
                    || runtime_used_identifiers.contains(*name)
                    || template_content.is_none())
        })
        .cloned()
        .collect();

    // Add emit binding to returned (it's a runtime value that should be exposed)
    if let Some(emit_name) = emit_binding_name
        && !returned_bindings.contains(emit_name)
    {
        returned_bindings.push(emit_name.clone());
    }

    returned_bindings.sort();

    // Parse template to get used identifiers
    let template_used_ids: TemplateUsedIdentifiers = if let Some(template_src) = template_content {
        let allocator = vize_carton::Allocator::new();
        let (root, _) = vize_atelier_core::parser::parse(&allocator, template_src);
        resolve_template_used_identifiers(&root)
    } else {
        TemplateUsedIdentifiers::default()
    };

    // Include imported identifiers that are used in template
    let mut all_bindings = returned_bindings.clone();
    for name in &imported_identifier_set {
        if template_content.is_none()
            || runtime_used_identifiers.contains(name)
            || template_used_ids.used_ids.contains(name.as_str())
        {
            if !all_bindings.contains(name) {
                all_bindings.push(name.clone());
            }
            // Also add to binding metadata so template compiler knows about it
            if !ctx.bindings.bindings.contains_key(name.as_str()) {
                ctx.bindings
                    .bindings
                    .insert(name.clone(), BindingType::SetupConst);
            }
        }
    }
    all_bindings.sort();
    all_bindings.dedup();

    all_bindings
}

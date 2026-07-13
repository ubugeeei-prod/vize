//! Selection of bindings exposed from function-mode setup.

use vize_carton::{Bump, FxHashSet, String};
use vize_croquis::macros::runtime_erased_macro_names;

use crate::script::{
    ScriptCompileContext, TemplateUsedIdentifiers, resolve_template_used_identifiers,
};
use crate::types::BindingType;

use super::super::super::import_utils::extract_import_identifiers;
use super::output::prop_types_from_context;

/// Build the list of bindings to include in `__returned__`.
///
/// Filters out compiler macros, destructured props, props bindings, and typed props.
/// Includes imported identifiers used in the template.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_returned_bindings(
    ctx: &mut ScriptCompileContext,
    _has_props_destructure: bool,
    emit_binding_name: &Option<String>,
    imports: &[String],
    external_value_imports: &[String],
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

    let mut imported_identifier_set: FxHashSet<String> = imports
        .iter()
        .flat_map(|import| extract_import_identifiers(import).into_iter())
        .filter(|name| !compiler_macros.contains(name.as_str()))
        .collect();
    imported_identifier_set.extend(
        external_value_imports
            .iter()
            .filter(|name| !compiler_macros.contains(name.as_str()))
            .cloned(),
    );

    // Generate __returned__ object
    let mut returned_bindings: Vec<String> = ctx
        .bindings
        .bindings
        .keys()
        .filter(|name| {
            // Exclude compiler macros, destructured props, and typed props.
            !compiler_macros.contains(name.as_str())
                && !destructured_prop_locals.contains(*name)
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
        let allocator = Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template_src);
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

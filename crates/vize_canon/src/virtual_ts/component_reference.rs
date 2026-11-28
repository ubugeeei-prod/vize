use vize_carton::{CompactString, FxHashSet, String, camelize, capitalize};
use vize_croquis::{BindingType, Croquis};

use super::{
    helpers::{is_reserved_identifier, to_safe_identifier},
    script_facts::ScriptBindings,
    types::VirtualTsOptions,
};

pub(crate) fn component_reference_alias(
    summary: &Croquis,
    options: &VirtualTsOptions,
    type_only: &FxHashSet<CompactString>,
    template_name: &str,
) -> String {
    super::component_aliases::reference(summary, options, type_only, template_name)
}

pub(crate) fn component_binding_reference(
    summary: &Croquis,
    options: &VirtualTsOptions,
    syntactic_type_only_imported_names: &FxHashSet<CompactString>,
    template_name: &str,
) -> String {
    component_binding_reference_with_resolution(
        summary,
        options,
        syntactic_type_only_imported_names,
        template_name,
    )
    .0
}

pub(crate) fn component_binding_reference_with_resolution(
    summary: &Croquis,
    options: &VirtualTsOptions,
    syntactic_type_only_imported_names: &FxHashSet<CompactString>,
    template_name: &str,
) -> (String, bool) {
    let resolved = super::script_facts::with_bindings(summary, |bindings| {
        resolved_component_binding_reference_with_bindings(
            summary,
            bindings,
            options,
            syntactic_type_only_imported_names,
            template_name,
        )
    });
    let is_resolved = resolved.is_some();
    (
        resolved.map(|(reference, _)| reference).unwrap_or_else(|| {
            component_reference_alias(
                summary,
                options,
                syntactic_type_only_imported_names,
                template_name,
            )
        }),
        is_resolved,
    )
}

/// The selected declaration branch establishes value ownership; alias spelling does not.
pub(crate) fn component_binding_reference_with_provider(
    summary: &Croquis,
    options: &VirtualTsOptions,
    type_only: &FxHashSet<CompactString>,
    template_name: &str,
) -> (String, bool) {
    super::script_facts::with_bindings(summary, |bindings| {
        resolved_component_binding_reference_with_bindings(
            summary,
            bindings,
            options,
            type_only,
            template_name,
        )
    })
    .unwrap_or_else(|| {
        (
            component_reference_alias(summary, options, type_only, template_name),
            false,
        )
    })
}

pub(super) fn component_binding_reference_with_bindings(
    summary: &Croquis,
    bindings: ScriptBindings<'_>,
    options: &VirtualTsOptions,
    syntactic_type_only_imported_names: &FxHashSet<CompactString>,
    template_name: &str,
) -> String {
    resolved_component_binding_reference_with_bindings(
        summary,
        bindings,
        options,
        syntactic_type_only_imported_names,
        template_name,
    )
    .map(|(reference, _)| reference)
    .unwrap_or_else(|| {
        component_reference_alias(
            summary,
            options,
            syntactic_type_only_imported_names,
            template_name,
        )
    })
}

pub(crate) fn resolved_component_binding_reference(
    summary: &Croquis,
    options: &VirtualTsOptions,
    syntactic_type_only_imported_names: &FxHashSet<CompactString>,
    template_name: &str,
) -> Option<String> {
    super::script_facts::with_bindings(summary, |bindings| {
        resolved_component_binding_reference_with_bindings(
            summary,
            bindings,
            options,
            syntactic_type_only_imported_names,
            template_name,
        )
        .map(|(reference, _)| reference)
    })
}

fn resolved_component_binding_reference_with_bindings(
    summary: &Croquis,
    bindings: ScriptBindings<'_>,
    options: &VirtualTsOptions,
    syntactic_type_only_imported_names: &FxHashSet<CompactString>,
    template_name: &str,
) -> Option<(String, bool)> {
    let camel_name = camelize(template_name);
    let pascal_name = capitalize(camel_name.as_str());
    // Vue Language Tools resolves PascalCase, camelCase, then the authored tag.
    // Value bindings win over a type-only PascalCase collision
    // (`chartComponent` vs `import type { ChartComponent }`).
    // Ambient `external_template_bindings` must not beat a type-only import of
    // the same name: `import type { ElBadge }` still uses `$Component_*`.
    for candidate in [pascal_name.as_str(), camel_name.as_str(), template_name] {
        if let Some(binding_type) = bindings.binding_type(candidate)
            && !contains_compact_name(syntactic_type_only_imported_names, candidate)
        {
            return Some((
                component_binding_reference_for_summary_binding(
                    summary,
                    bindings,
                    candidate,
                    binding_type,
                    options,
                ),
                true,
            ));
        }
    }
    if dotted_component_root(template_name).is_some()
        && let Some(reference) = value_path_reference(
            summary,
            bindings,
            options,
            syntactic_type_only_imported_names,
            template_name,
        )
    {
        return Some((reference, true));
    }
    // The SFC's own `components` option outranks global and ambient names.
    if let Some(local) = registrations::registered_local(
        summary,
        [template_name, camel_name.as_str(), pascal_name.as_str()],
    ) && let Some(reference) = value_path_reference(
        summary,
        bindings,
        options,
        syntactic_type_only_imported_names,
        local,
    ) {
        return Some((reference, true));
    }
    if let Some(reference) = super::component_aliases::inline_value(
        summary,
        options,
        syntactic_type_only_imported_names,
        template_name,
    ) {
        return Some((reference, true));
    }
    if has_type_only_component_candidate(syntactic_type_only_imported_names, template_name) {
        return Some((
            component_reference_alias(
                summary,
                options,
                syntactic_type_only_imported_names,
                template_name,
            ),
            false,
        ));
    }
    for candidate in [pascal_name.as_str(), camel_name.as_str(), template_name] {
        if options
            .external_template_bindings
            .iter()
            .any(|name| name.as_str() == candidate)
        {
            return Some((String::from(candidate), true));
        }
    }
    None
}

/// `path` (`Child`, `ui.Menu`) when its root is a value binding.
fn value_path_reference(
    summary: &Croquis,
    bindings: ScriptBindings<'_>,
    options: &VirtualTsOptions,
    syntactic_type_only_imported_names: &FxHashSet<CompactString>,
    path: &str,
) -> Option<String> {
    let root = path.split('.').next()?;
    if contains_compact_name(syntactic_type_only_imported_names, root) {
        return None;
    }
    let binding_type = bindings.binding_type(root)?;
    let mut reference = component_binding_reference_for_summary_binding(
        summary,
        bindings,
        root,
        binding_type,
        options,
    );
    reference.push_str(path.get(root.len()..)?);
    Some(reference)
}

/// The value a `components` option key gives the tag (`[tag, camel, pascal]`).
pub(crate) fn registered_component_reference(
    summary: &Croquis,
    options: &VirtualTsOptions,
    syntactic_type_only_imported_names: &FxHashSet<CompactString>,
    candidates: [&str; 3],
) -> Option<String> {
    let local = registrations::registered_local(summary, candidates)?;
    super::script_facts::with_bindings(summary, |bindings| {
        value_path_reference(
            summary,
            bindings,
            options,
            syntactic_type_only_imported_names,
            local,
        )
    })
}

/// A dotted tag is a member path only after all parts are valid identifiers;
/// its first part still needs an actual value-binding owner.
pub(crate) fn dotted_component_root(name: &str) -> Option<&str> {
    let (root, tail) = name.split_once('.')?;
    (oxc_syntax::identifier::is_identifier_name(root)
        && tail
            .split('.')
            .all(oxc_syntax::identifier::is_identifier_name))
    .then_some(root)
}

pub(crate) fn has_dotted_component_value_binding(
    summary: &Croquis,
    type_only: &FxHashSet<CompactString>,
    name: &str,
) -> bool {
    dotted_component_root(name).is_some_and(|root| {
        !contains_compact_name(type_only, root)
            && super::script_facts::contains_binding(summary, root)
    })
}

/// Resolve an expression identifier exactly, without markup case candidates.
pub(crate) fn resolved_exact_component_binding_reference(
    summary: &Croquis,
    options: &VirtualTsOptions,
    syntactic_type_only_imported_names: &FxHashSet<CompactString>,
    name: &str,
) -> Option<String> {
    if contains_compact_name(syntactic_type_only_imported_names, name) {
        return None;
    }
    super::script_facts::with_bindings(summary, |bindings| {
        if let Some(binding_type) = bindings.binding_type(name) {
            return Some(component_binding_reference_for_summary_binding(
                summary,
                bindings,
                name,
                binding_type,
                options,
            ));
        }
        options
            .external_template_bindings
            .iter()
            .any(|binding| binding.as_str() == name)
            .then(|| String::from(name))
    })
}

fn component_binding_reference_for_summary_binding(
    summary: &Croquis,
    bindings: ScriptBindings<'_>,
    candidate: &str,
    binding_type: BindingType,
    options: &VirtualTsOptions,
) -> String {
    if is_reserved_identifier(candidate) && !bindings.is_script_setup() {
        return options_api_reserved_binding_local_with_bindings(bindings, options, candidate);
    }
    if matches!(binding_type, BindingType::Props | BindingType::PropsAliased)
        && !is_props_destructure_local(summary, candidate)
    {
        return to_safe_identifier(candidate);
    }
    String::from(candidate)
}

fn is_props_destructure_local(summary: &Croquis, candidate: &str) -> bool {
    summary
        .macros
        .props_destructure()
        .is_some_and(|destructure| {
            destructure
                .bindings
                .values()
                .any(|binding| binding.local.as_str() == candidate)
                || destructure
                    .rest_id
                    .as_ref()
                    .is_some_and(|rest_id| rest_id.as_str() == candidate)
        })
}

/// The registry keys Vue's `resolveAsset` tries for a markup tag, in order:
/// PascalCase, camelCase, then the authored spelling.
pub(crate) fn registry_keys(tag: &str) -> [String; 3] {
    let camel = camelize(tag);
    [capitalize(camel.as_str()), camel, String::from(tag)]
}

/// The global component a tag no binding owns, as Vue resolves it from
/// `GlobalComponents`: `component` wraps the first registered key's type
/// (named `__C`), and `fallback` is what the tag is when none is registered
/// (a custom element).
///
/// `<my-child>` and `<MyChild>` name the same `GlobalComponents` entry, so
/// whatever reads the component of a tag (props, events, slots, refs) sees
/// it under either spelling. Each key is matched structurally: indexing
/// `GlobalComponents["myChild"]` behind a `"myChild" extends keyof` test is
/// still an error (TS2538) for an absent key, which the checker turns into
/// `any` for the whole lookup.
pub(crate) fn global_component_type(
    tag: &str,
    component: impl Fn(&str) -> String,
    fallback: &str,
) -> String {
    global_component_type_as(tag, "__C", component, fallback)
}

/// [`global_component_type`] with the registered type inferred as `name`, so
/// that lookups for several tags can nest without shadowing each other.
pub(crate) fn global_component_type_as(
    tag: &str,
    name: &str,
    component: impl Fn(&str) -> String,
    fallback: &str,
) -> String {
    let registered = component(name);
    let mut output = String::default();
    let mut previous: Option<String> = None;
    let mut branches = 0_usize;
    for key in registry_keys(tag) {
        if previous.as_ref() == Some(&key) {
            continue;
        }
        let mut key_literal = super::TextBuf::default();
        super::helpers::push_ts_string_literal(&mut key_literal, key.as_str());
        vize_carton::append!(
            output,
            "(import('vue').GlobalComponents extends {{ {key_literal}: infer {name} }} ? {registered} : "
        );
        branches += 1;
        previous = Some(key);
    }
    output.push_str(fallback);
    for _ in 0..branches {
        output.push(')');
    }
    output
}

pub(crate) fn has_type_only_component_candidate(
    syntactic_type_only_imported_names: &FxHashSet<CompactString>,
    template_name: &str,
) -> bool {
    let camel_name = camelize(template_name);
    let pascal_name = capitalize(camel_name.as_str());
    [template_name, camel_name.as_str(), pascal_name.as_str()]
        .iter()
        .any(|candidate| contains_compact_name(syntactic_type_only_imported_names, candidate))
}

pub(crate) fn contains_compact_name(names: &FxHashSet<CompactString>, name: &str) -> bool {
    names.iter().any(|candidate| candidate.as_str() == name)
}

// Both declarations and component references must choose the same local without
// shadowing an authored binding or a configured template binding.
pub(crate) fn options_api_reserved_binding_local(
    summary: &Croquis,
    options: &VirtualTsOptions,
    name: &str,
) -> String {
    super::script_facts::with_bindings(summary, |bindings| {
        options_api_reserved_binding_local_with_bindings(bindings, options, name)
    })
}

fn options_api_reserved_binding_local_with_bindings(
    bindings: ScriptBindings<'_>,
    options: &VirtualTsOptions,
    name: &str,
) -> String {
    let mut local = to_safe_identifier(name);
    while bindings.contains_binding(local.as_str())
        || options
            .template_globals
            .iter()
            .any(|global| global.name == local)
        || options
            .external_template_bindings
            .iter()
            .any(|binding| binding.as_str() == local.as_str())
    {
        local.push('_');
    }
    local
}

/// Whether Vue's compiler gives the element its component before it reads
/// setup bindings (`resolveComponentType`): `<component>` or `<Component>`
/// with an `is` prop is the dynamic component, and the core and DOM built-ins
/// (`Teleport`, `Suspense`, `KeepAlive`, `BaseTransition`, `Transition`,
/// `TransitionGroup`, each also in kebab case) are Vue's own. A setup binding
/// spelled like one of these tags never owns it.
pub(crate) fn resolves_before_setup_bindings(element: &vize_relief::ElementNode<'_>) -> bool {
    if vize_croquis::builtins::is_runtime_builtin_component(element.tag) {
        return true;
    }
    matches!(element.tag, "component" | "Component")
        && element.props.iter().any(|prop| match prop {
            vize_relief::PropNode::Attribute(attribute) => attribute.name == "is",
            vize_relief::PropNode::Directive(directive) => {
                directive.name == "bind"
                    && matches!(
                        &directive.arg,
                        Some(vize_relief::ExpressionNode::Simple(arg))
                            if arg.is_static && arg.content == "is"
                    )
            }
        })
}

mod registrations;
pub(crate) use registrations::is_registered_component;

#[cfg(test)]
mod dotted_tests;

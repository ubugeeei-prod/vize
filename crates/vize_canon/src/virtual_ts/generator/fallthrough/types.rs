//! Shared fallthrough targets with public type names for editor-facing context.

use super::{
    FallthroughRootTarget, explicit_attrs_targets, possible_single_root_targets,
    push_ts_string_literal,
};
use vize_carton::{CompactString, FxHashSet, String};
use vize_croquis::Croquis;
use vize_relief::RootNode;

use crate::virtual_ts::{
    component_reference::resolved_component_binding_reference, types::VirtualTsOptions,
};

/// The bindings a component root resolves through. A root that is not a
/// setup binding (a globally registered component, an unresolved tag) keeps
/// the open `Record<string, unknown>` surface.
pub(in crate::virtual_ts::generator) struct FallthroughComponentScope<'a> {
    pub(in crate::virtual_ts::generator) summary: &'a Croquis,
    pub(in crate::virtual_ts::generator) options: &'a VirtualTsOptions,
    pub(in crate::virtual_ts::generator) syntactic_type_only_imported_names:
        &'a FxHashSet<CompactString>,
    /// `checkRequiredFallthroughAttributes`: forward the root's props as they
    /// are declared, minus the ones the root binds itself, instead of making
    /// every forwarded prop optional.
    pub(in crate::virtual_ts::generator) check_required: bool,
}

/// Template-relative starts of the component roots whose props the component
/// forwards. Empty unless fallthrough resolves to at least one component.
pub(crate) fn fallthrough_component_root_starts(
    summary: &Croquis,
    template_ast: Option<&RootNode<'_>>,
) -> Vec<u32> {
    let Some(template_ast) = template_ast else {
        return Vec::new();
    };
    fallthrough_targets(summary, template_ast)
        .unwrap_or_default()
        .iter()
        .filter_map(|target| match target {
            FallthroughRootTarget::Component(root) => Some(root.start),
            FallthroughRootTarget::Native(_) => None,
        })
        .collect()
}

fn fallthrough_targets(
    summary: &Croquis,
    template_ast: &RootNode<'_>,
) -> Option<Vec<FallthroughRootTarget>> {
    if summary.template_info.inherit_attrs_disabled {
        return explicit_attrs_targets(template_ast);
    }
    explicit_attrs_targets(template_ast).or_else(|| possible_single_root_targets(template_ast))
}

pub(in crate::virtual_ts::generator) fn fallthrough_props_type_ref(
    scope: &FallthroughComponentScope<'_>,
    template_ast: Option<&RootNode<'_>>,
    legacy_vue2: bool,
) -> Option<String> {
    if legacy_vue2 {
        return None;
    }
    fallthrough_type_ref(scope.summary, template_ast, false, Some(scope))
}

/// Use public Vue types in the hook signature displayed by the editor.
pub(in crate::virtual_ts::generator) fn fallthrough_attrs_type_ref(
    summary: &Croquis,
    template_ast: Option<&RootNode<'_>>,
) -> Option<String> {
    fallthrough_type_ref(summary, template_ast, true, None)
}

fn fallthrough_type_ref(
    summary: &Croquis,
    template_ast: Option<&RootNode<'_>>,
    public_types: bool,
    scope: Option<&FallthroughComponentScope<'_>>,
) -> Option<String> {
    let Some(template_ast) = template_ast else {
        return Some(String::from("Record<string, unknown>"));
    };
    let targets = fallthrough_targets(summary, template_ast)?;
    Some(targets_type_ref(&targets, public_types, scope))
}

fn targets_type_ref(
    targets: &[FallthroughRootTarget],
    public_types: bool,
    scope: Option<&FallthroughComponentScope<'_>>,
) -> String {
    let check_required = scope.is_some_and(|scope| scope.check_required);
    let mut ty = String::default();
    for (index, target) in targets.iter().enumerate() {
        let (root, surface) = match target {
            FallthroughRootTarget::Native(root) => {
                (root, native_target_surface(root.tag.as_str(), public_types))
            }
            FallthroughRootTarget::Component(root) => {
                // A component root forwards what the rendered component
                // accepts: its public props and the listener props of its
                // emits, plus whatever it forwards itself. Only a resolved
                // setup binding has that surface; anything else stays open.
                let Some(reference) = scope.and_then(|scope| {
                    resolved_component_binding_reference(
                        scope.summary,
                        scope.options,
                        scope.syntactic_type_only_imported_names,
                        root.tag.as_str(),
                    )
                }) else {
                    return String::from("Record<string, unknown>");
                };
                let mut surface = String::from("__VizeComponentFallthroughProps<typeof ");
                surface.push_str(reference.as_str());
                surface.push('>');
                (root, surface)
            }
        };
        if index > 0 {
            ty.push_str(" & ");
        }
        // Forwarded props are optional to the parent unless it is asked to
        // supply the root's required ones; the root's own bindings are never
        // the parent's to provide either way.
        if !check_required {
            ty.push_str("Partial<");
            ty.push_str(surface.as_str());
            ty.push('>');
        } else if root.authored_keys.is_empty() {
            ty.push_str(surface.as_str());
        } else {
            ty.push_str("Omit<");
            ty.push_str(surface.as_str());
            ty.push_str(", ");
            for (key_index, key) in root.authored_keys.iter().enumerate() {
                if key_index > 0 {
                    ty.push_str(" | ");
                }
                push_ts_string_literal(&mut ty, key.as_str());
            }
            ty.push('>');
        }
    }
    ty
}

fn native_target_surface(tag: &str, public_types: bool) -> String {
    let mut ty = String::from(if public_types {
        "import('vue').NativeElements["
    } else {
        "__VizeNativeElement<"
    });
    push_ts_string_literal(&mut ty, tag);
    ty.push_str(if public_types { "]" } else { ">" });
    ty
}

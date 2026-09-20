//! Shared fallthrough targets with public type names for editor-facing context.

use super::{
    FallthroughRootTarget, explicit_attrs_targets, possible_single_root_targets,
    push_ts_string_literal,
};
use vize_carton::String;
use vize_croquis::Croquis;
use vize_relief::RootNode;

pub(in crate::virtual_ts::generator) fn fallthrough_props_type_ref(
    summary: &Croquis,
    template_ast: Option<&RootNode<'_>>,
    legacy_vue2: bool,
) -> Option<String> {
    if legacy_vue2 {
        return None;
    }
    fallthrough_type_ref(summary, template_ast, false)
}

/// Use public Vue types in the hook signature displayed by the editor.
pub(in crate::virtual_ts::generator) fn fallthrough_attrs_type_ref(
    summary: &Croquis,
    template_ast: Option<&RootNode<'_>>,
) -> Option<String> {
    fallthrough_type_ref(summary, template_ast, true)
}

fn fallthrough_type_ref(
    summary: &Croquis,
    template_ast: Option<&RootNode<'_>>,
    public_types: bool,
) -> Option<String> {
    let Some(template_ast) = template_ast else {
        return Some(String::from("Record<string, unknown>"));
    };
    if summary.template_info.inherit_attrs_disabled {
        return explicit_attrs_targets(template_ast)
            .map(|targets| targets_type_ref(&targets, public_types));
    }

    let targets = explicit_attrs_targets(template_ast)
        .or_else(|| possible_single_root_targets(template_ast))?;
    Some(targets_type_ref(&targets, public_types))
}

fn targets_type_ref(targets: &[FallthroughRootTarget], public_types: bool) -> String {
    if targets
        .iter()
        .any(|target| matches!(target, FallthroughRootTarget::Component))
    {
        return String::from("Record<string, unknown>");
    }

    let mut ty = String::default();
    for (index, target) in targets.iter().enumerate() {
        let FallthroughRootTarget::Native(tag) = target else {
            unreachable!("component roots returned open fallthrough props above");
        };
        if index > 0 {
            ty.push_str(" & ");
        }
        ty.push_str(if public_types {
            "Partial<import('vue').NativeElements["
        } else {
            "Partial<__VizeNativeElement<"
        });
        push_ts_string_literal(&mut ty, tag.as_str());
        ty.push_str(if public_types { "]>" } else { ">>" });
    }
    ty
}

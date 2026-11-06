//! The user key of a `v-if` branch.

use super::super::structural_keys::keys_equal;
use crate::errors::ErrorCode;
use crate::{ElementNode, IfNode, PropNode, TemplateChildNode};

use super::super::TransformContext;
use super::{MATCH_SCOPE_RAW_NAME, extract_key_prop};

/// Take the branch element's `key`, processed for identifier prefixing
/// (`keyA` -> `_ctx.keyA`).
///
/// A key on an element that also has `v-for` belongs to the loop. The branch
/// of a patterned-template arm is rendered inside the arm's binding scope, so
/// its key reads those bindings (`:key="id"` for `v-when="{ const id }"`)
/// rather than component state.
pub(super) fn take_user_key<'a>(
    node: TemplateChildNode<'a>,
) -> (TemplateChildNode<'a>, Option<PropNode<'a>>) {
    let TemplateChildNode::Element(mut el) = node else {
        return (node, None);
    };
    let has_v_for = el
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "for"));
    let user_key = if has_v_for {
        None
    } else {
        extract_key_prop(&mut el)
    };

    (TemplateChildNode::Element(el), user_key)
}

pub(super) fn process_user_key<'a>(
    ctx: &mut TransformContext<'a>,
    node: &TemplateChildNode<'a>,
    user_key: &mut Option<PropNode<'a>>,
) {
    if let TemplateChildNode::Element(el) = node
        && let Some(PropNode::Directive(dir)) = user_key
        && (ctx.options.prefix_identifiers || ctx.options.is_ts)
        && let Some(exp) = &dir.exp
    {
        let arm_scope = arm_binding_scope(el);
        if let Some((bindings, source)) = arm_scope {
            ctx.enter_v_for_scope(Some(bindings), None, None, source);
        }
        let processed = crate::steps::expression::process_branch_key(ctx, exp);
        if arm_scope.is_some() {
            ctx.exit_scope();
        }
        dir.exp = Some(processed);
    }
}

/// Vue compares each raw new key against earlier already-processed keys.
pub(super) fn collision_count(
    ctx: &TransformContext<'_>,
    if_node: &IfNode<'_>,
    new_key: &PropNode<'_>,
) -> usize {
    if_node
        .branches
        .iter()
        .filter(|branch| {
            branch.user_key.as_ref().is_some_and(|existing| {
                keys_equal(
                    existing,
                    new_key,
                    ctx.template_syntax_quirks(),
                    ctx.source,
                    branch.is_template_if,
                )
            })
        })
        .count()
}

pub(super) fn report_collisions(
    ctx: &mut TransformContext<'_>,
    new_key: &PropNode<'_>,
    count: usize,
) {
    let loc = match new_key {
        PropNode::Attribute(attr) => &attr.loc,
        PropNode::Directive(dir) => &dir.loc,
    };
    for _ in 0..count {
        ctx.on_error(ErrorCode::VIfSameKey, Some(loc.clone()));
    }
}

/// `(bindings, source)` of the patterned-template binding scope this branch
/// wraps, e.g. `("[, id]", "__vize_match_0")`.
fn arm_binding_scope<'e>(branch: &'e ElementNode<'_>) -> Option<(&'e str, &'e str)> {
    let [TemplateChildNode::Element(scope)] = branch.children.as_slice() else {
        return None;
    };
    scope.props.iter().find_map(|prop| match prop {
        PropNode::Directive(dir) if dir.raw_name == Some(MATCH_SCOPE_RAW_NAME) => {
            let crate::ExpressionNode::Simple(exp) = dir.exp.as_ref()? else {
                return None;
            };
            exp.content.split_once(" in ")
        }
        _ => None,
    })
}

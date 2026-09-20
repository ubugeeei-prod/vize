//! The user key of a `v-if` branch.

use crate::{ElementNode, PropNode, TemplateChildNode};

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
    ctx: &mut TransformContext<'a>,
    node: TemplateChildNode<'a>,
) -> (TemplateChildNode<'a>, Option<PropNode<'a>>) {
    let TemplateChildNode::Element(mut el) = node else {
        return (node, None);
    };
    let has_v_for = el
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "for"));
    let mut user_key = if has_v_for {
        None
    } else {
        extract_key_prop(&mut el)
    };

    if let Some(PropNode::Directive(dir)) = &mut user_key
        && (ctx.options.prefix_identifiers || ctx.options.is_ts)
        && let Some(exp) = &dir.exp
    {
        let arm_scope = arm_binding_scope(&el);
        if let Some((bindings, source)) = arm_scope {
            ctx.enter_v_for_scope(Some(bindings), None, None, source);
        }
        let processed = crate::steps::expression::process_expression(ctx, exp, false);
        if arm_scope.is_some() {
            ctx.exit_scope();
        }
        dir.exp = Some(processed);
    }
    (TemplateChildNode::Element(el), user_key)
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

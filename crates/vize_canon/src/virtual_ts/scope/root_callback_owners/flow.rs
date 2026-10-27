//! Conservative boundaries of the existing root expression emission stream.

use vize_carton::{FxHashMap, profile};
use vize_croquis::ScopeKind;
use vize_relief::{ElementType, TemplateChildNode};

use super::{
    super::{context::ComponentPropsContext, root_owner_packets::callback_input_facts},
    UsageInput,
};

pub(super) fn first_deferred_frontier(
    ctx: &ComponentPropsContext<'_, '_>,
    usages: &FxHashMap<u32, UsageInput<'_>>,
) -> Option<u32> {
    let scope = profile!("canon.virtual_ts.root_callback.scope_frontier", {
        ctx.summary
            .scopes
            .iter()
            .filter(|scope| {
                matches!(
                    scope.kind,
                    ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::VMatch | ScopeKind::VWhen
                )
            })
            .map(|scope| scope.span.start)
            .min()
    });
    let input = profile!("canon.virtual_ts.root_callback.input_flow", {
        ctx.template_ast
            .and_then(|root| first_deferred_input(&root.children, usages))
    });
    scope.into_iter().chain(input).min()
}

fn first_deferred_input(
    children: &[TemplateChildNode<'_>],
    usages: &FxHashMap<u32, UsageInput<'_>>,
) -> Option<u32> {
    for child in children {
        let found = match child {
            TemplateChildNode::Element(element) => {
                // Outlet payloads and nontrivial component inputs are checked
                // in another phase. Never move a later callback ahead of them.
                if element.tag_type == ElementType::Slot
                    || usages.get(&element.loc.span.start).is_some_and(|usage| {
                        usage
                            .syntax
                            .get_or_init(|| callback_input_facts(element))
                            .is_none()
                    })
                {
                    Some(element.loc.span.start)
                } else {
                    first_deferred_input(&element.children, usages)
                }
            }
            TemplateChildNode::If(node) => node
                .branches
                .iter()
                .find_map(|branch| first_deferred_input(&branch.children, usages)),
            TemplateChildNode::For(node) => Some(node.loc.span.start),
            _ => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}

//! Keep canonical callback checks at their authored root render position.
//!
//! A callback's type query observes the surrounding control flow at creation.
//! Emitting it after a later interpolation changes that type even if its body
//! never runs. These buffers belong to actual direct root usages; they do not
//! authorize moving a branch, loop, slot or an EXPECT inference packet.

use vize_carton::{FxHashMap, FxHashSet, String, profile};
use vize_croquis::{BindingType, ScopeKind, TemplateExpression, croquis::ComponentUsage};
use vize_relief::TemplateChildNode;

use super::{
    component_prop_checker::contains_inline_function_prop_value,
    component_props::generate_root_usage_checks,
    context::{ComponentPropsContext, ScopeMappings},
    root_owner_packets::{RootOwnerBuffer, RootOwnerPackets},
};
use crate::virtual_ts::{
    VizeSemanticLink,
    diagnostic_ownership::DiagnosticOwnershipCollector,
    expressions::{ExpressionListEmitContext, generate_expressions},
    template_binding_access::TemplateBindingAccess,
};

mod flow;

struct UsageInput<'a> {
    index: usize,
    usage: &'a ComponentUsage,
    syntax: std::cell::OnceCell<Option<bool>>,
}

#[derive(Default)]
pub(super) struct RootCallbackOwners {
    indices: FxHashSet<usize>,
    scopes: FxHashMap<u32, Vec<(u32, RootOwnerBuffer)>>,
}

impl RootCallbackOwners {
    pub(super) fn collect(
        ctx: &ComponentPropsContext<'_, '_>,
        usages: &[(usize, &ComponentUsage)],
        expected: &RootOwnerPackets,
        expressions: &FxHashMap<u32, Vec<&TemplateExpression>>,
    ) -> Self {
        let mut result = Self::default();
        let Some(root) = ctx.template_ast else {
            return result;
        };
        if ctx.legacy_vue2
            || ctx.experimental_strict_slot_children
            || !usages.iter().any(|(idx, usage)| {
                !expected.contains(*idx)
                    && usage.props.iter().any(|prop| {
                        prop.is_dynamic
                            && !prop.name_is_dynamic
                            && !matches!(prop.name.as_str(), "key" | "ref")
                            && prop.value.as_ref().is_some_and(|value| {
                                contains_inline_function_prop_value(value.as_str())
                            })
                    })
            })
        {
            return result;
        }
        // The text marker is only an allocation-free fast gate. Retained
        // syntax, not that marker or a second parse, identifies the callback.
        let first_callback = profile!("canon.virtual_ts.root_callback.retained_gate", {
            root.children.iter().find_map(|child| {
                let TemplateChildNode::Element(element) = child else {
                    return None;
                };
                if element.tag_type != vize_relief::ElementType::Component {
                    return None;
                }
                let facts = super::root_owner_packets::callback_input_facts(element);
                (facts == Some(true)).then_some((element.loc.span.start, facts))
            })
        });
        let Some((first, facts)) = first_callback else {
            return result;
        };
        crate::virtual_ts::script_facts::with_bindings(ctx.summary, |bindings| {
            if !bindings.is_script_setup() {
                return;
            }
            let by_start: FxHashMap<_, _> = usages
                .iter()
                .map(|&(index, usage)| {
                    (
                        usage.start,
                        UsageInput {
                            index,
                            usage,
                            syntax: std::cell::OnceCell::new(),
                        },
                    )
                })
                .collect();
            if let Some(input) = by_start.get(&first) {
                let _ = input.syntax.set(facts);
            }
            let ordered_scopes: FxHashSet<_> = expressions
                .iter()
                .filter(|(_, expressions)| {
                    expressions.windows(2).all(|pair| {
                        pair.first()
                            .zip(pair.get(1))
                            .is_some_and(|(a, b)| a.start <= b.start)
                    })
                })
                .map(|(scope, _)| *scope)
                .collect();
            let frontier = flow::first_deferred_frontier(ctx, &by_start);
            for child in &root.children {
                let TemplateChildNode::Element(element) = child else {
                    continue;
                };
                let Some(input) = by_start.get(&element.loc.span.start) else {
                    continue;
                };
                let (idx, usage) = (input.index, input.usage);
                let scope = usage.scope_id.as_u32();
                if expected.contains(idx)
                    || frontier.is_some_and(|frontier| usage.start >= frontier)
                    || usage.end != element.loc.span.end
                    || usage.vif_guard.is_some()
                    || !usage.slots.is_empty()
                    || !usage.spread_props.is_empty()
                    || ctx.explicit_generics.alias(usage.start).is_some()
                    || bindings.binding_type(usage.name.as_str()) != Some(BindingType::SetupConst)
                    || input.syntax.get().copied().flatten() != Some(true)
                    || !ctx
                        .summary
                        .scopes
                        .get_scope(usage.scope_id)
                        .is_some_and(|scope| {
                            matches!(
                                scope.kind,
                                ScopeKind::ScriptSetup
                                    | ScopeKind::JsGlobalUniversal
                                    | ScopeKind::JsGlobalBrowser
                                    | ScopeKind::JsGlobalNode
                                    | ScopeKind::VueGlobal
                            )
                        })
                {
                    continue;
                }
                // The root stream must have a real authored binding boundary.
                // Unknown or non-monotonic source recovery retains its old owner.
                let Some(expressions) = expressions.get(&scope) else {
                    continue;
                };
                let boundary =
                    expressions.partition_point(|expression| expression.start < usage.start);
                if !ordered_scopes.contains(&scope)
                    || !expressions
                        .get(boundary)
                        .is_some_and(|expression| expression.end <= usage.end)
                {
                    continue;
                }
                let mut buffer = RootOwnerBuffer::default();
                let owner_ctx = ComponentPropsContext {
                    inference_prelude: None,
                    diagnostic_ownership: ctx
                        .diagnostic_ownership
                        .map(|_| &buffer.diagnostic_ownership),
                    ..*ctx
                };
                profile!("canon.virtual_ts.root_callback.owner", {
                    generate_root_usage_checks(
                        &mut buffer.text,
                        &mut buffer.mappings,
                        &owner_ctx,
                        idx,
                        usage,
                        true,
                    )
                });
                result.indices.insert(idx);
                result
                    .scopes
                    .entry(scope)
                    .or_default()
                    .push((usage.start, buffer));
            }
        });
        result
    }

    pub(super) fn contains(&self, idx: usize) -> bool {
        self.indices.contains(&idx)
    }

    pub(super) fn has_scope(&self, scope: u32) -> bool {
        self.scopes.contains_key(&scope)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the existing scope stream and its complete projection metadata travel together"
    )]
    pub(super) fn generate_expressions(
        &mut self,
        ts: &mut String,
        mappings: &mut ScopeMappings<'_>,
        links: &mut Vec<VizeSemanticLink>,
        ownership: Option<&DiagnosticOwnershipCollector>,
        scope: u32,
        expressions: &[&TemplateExpression],
        access: &TemplateBindingAccess,
        context: &ExpressionListEmitContext<'_>,
    ) {
        let Some(owners) = self.scopes.remove(&scope) else {
            generate_expressions(ts, mappings, expressions, access, context);
            return;
        };
        let mut first = 0;
        for (start, buffer) in owners {
            let mut end = first;
            while expressions
                .get(end)
                .is_some_and(|expression| expression.start < start)
            {
                end += 1;
            }
            if let Some(before) = expressions.get(first..end) {
                generate_expressions(ts, mappings, before, access, context);
            }
            buffer.append_to(
                ts,
                mappings.navigation,
                mappings.diagnostics,
                links,
                ownership,
            );
            first = end;
        }
        if let Some(after) = expressions.get(first..) {
            generate_expressions(ts, mappings, after, access, context);
        }
    }
}

#[cfg(test)]
mod tests;

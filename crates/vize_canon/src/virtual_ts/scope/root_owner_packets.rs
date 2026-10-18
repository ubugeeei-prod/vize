//! Deferred checks for root owners with no mutable template dependencies.
//!
//! Reuse actual Croquis usage indices and event scope spans. These are emission
//! buffers, not another template/scope graph. Dynamic expressions deliberately
//! keep the old order: moving a closure past an assignment changes TS narrowing.

use vize_carton::{FxHashMap, FxHashSet, String};
use vize_croquis::{BindingType, Scope, ScopeData, analysis::ComponentUsage};
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};

use super::{component_prop_navigation, context::ComponentPropsContext};
use crate::template_diagnostic_directives::{DirectivePolicy, TemplateDiagnosticDirectives};
use crate::virtual_ts::{VizeSemanticLink, types::VizeMapping};

#[derive(Default)]
pub(super) struct RootOwnerPacket {
    pub(super) text: String,
    pub(super) mappings: Vec<VizeMapping>,
    pub(super) diagnostic_mappings: Vec<VizeMapping>,
    pub(super) semantic_links: Vec<VizeSemanticLink>,
}

impl RootOwnerPacket {
    pub(super) fn append_to(
        self,
        text: &mut String,
        mappings: &mut Vec<VizeMapping>,
        diagnostic_mappings: &mut Vec<VizeMapping>,
        links: &mut Vec<VizeSemanticLink>,
    ) {
        let offset = text.len();
        text.push_str(self.text.as_str());
        for (target, rows) in [
            (mappings, self.mappings),
            (diagnostic_mappings, self.diagnostic_mappings),
        ] {
            target.extend(rows.into_iter().map(|mut mapping| {
                mapping.gen_range.start += offset;
                mapping.gen_range.end += offset;
                for span in &mut mapping.sub_spans {
                    span.gen_range.start += offset;
                    span.gen_range.end += offset;
                }
                mapping
            }));
        }
        links.extend(self.semantic_links.into_iter().map(|mut link| {
            link.source_range.start += offset;
            link.source_range.end += offset;
            link.target_range.start += offset;
            link.target_range.end += offset;
            link
        }));
    }
}

#[derive(Default)]
pub(super) struct RootOwnerPackets {
    packets: FxHashMap<usize, RootOwnerPacket>,
    /// The exact directive spans used by Croquis when creating handler scopes.
    events: FxHashMap<(u32, u32), usize>,
}

impl RootOwnerPackets {
    pub(super) fn collect(
        ctx: &ComponentPropsContext<'_, '_>,
        usages: &[(usize, &ComponentUsage)],
    ) -> Self {
        let mut result = Self::default();
        let Some(root) = ctx.template_ast else {
            return result;
        };
        if !root.source.contains("@vue-expect-error")
            || ctx.legacy_vue2
            || ctx.options.strict_instance_globals
            || ctx.experimental_strict_slot_children
            || !crate::virtual_ts::script_facts::is_script_setup(ctx.summary)
        {
            return result;
        }
        // The source marker is only a cheap activation gate. Exact ownership,
        // precedence and orphan handling come from the shared canonical walk.
        let directives = TemplateDiagnosticDirectives::for_root(root, 0);
        let expected_owners: FxHashSet<_> = directives
            .directives()
            .iter()
            .filter(|directive| directive.policy == DirectivePolicy::Expect)
            .flat_map(|directive| {
                directive
                    .targets
                    .iter()
                    .map(|range| (range.start, range.end))
            })
            .collect();
        if expected_owners.is_empty() {
            return result;
        }
        let by_start: FxHashMap<_, _> = usages
            .iter()
            .map(|&(idx, usage)| (usage.start, (idx, usage)))
            .collect();
        crate::virtual_ts::script_facts::with_bindings(ctx.summary, |bindings| {
            for child in &root.children {
                let TemplateChildNode::Element(element) = child else {
                    continue;
                };
                let Some(&(idx, usage)) = by_start.get(&element.loc.span.start) else {
                    continue;
                };
                if !expected_owners.contains(&(usage.start as usize, usage.end as usize))
                    || usage.end != element.loc.span.end || usage.vif_guard.is_some()
                    || ctx.explicit_generics.alias(usage.start).is_some()
                    || bindings.binding_type(usage.name.as_str()) != Some(BindingType::SetupConst)
                    || usage.props.is_empty()
                    || usage.props.iter().all(|prop| matches!(prop.name.as_str(), "key" | "ref"))
                    || usage.props.iter().any(|prop| prop.is_dynamic || prop.name_is_dynamic)
                    || !usage.spread_props.is_empty()
                    || usage.events.is_empty()
                    || usage.events.iter().any(|event| event.name_is_dynamic
                        || event.handler.as_ref().is_none_or(|handler|
                            bindings.binding_type(handler.as_str().trim()) != Some(BindingType::SetupConst)))
                    || element.props.iter().any(|prop| match prop {
                        PropNode::Attribute(_) => false,
                        PropNode::Directive(directive) => directive.name != "on"
                            || !matches!(directive.arg.as_ref(), Some(ExpressionNode::Simple(arg)) if arg.is_static),
                    })
                    || usage.slots.iter().any(|slot|
                        component_prop_navigation::slot_navigation_source_range(ctx.template_source, slot)
                            .is_some_and(|range| range.start < usage.start as usize || range.end > usage.end as usize))
                {
                    continue;
                }
                result.packets.insert(idx, RootOwnerPacket::default());
                for event in &usage.events {
                    result.events.insert((event.start, event.end), idx);
                }
            }
        });
        result
    }

    pub(super) fn contains(&self, idx: usize) -> bool {
        self.packets.contains_key(&idx)
    }
    pub(super) fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }
    pub(super) fn get_mut(&mut self, idx: usize) -> Option<&mut RootOwnerPacket> {
        self.packets.get_mut(&idx)
    }
    pub(super) fn for_scope(&mut self, scope: &Scope) -> Option<&mut RootOwnerPacket> {
        if !matches!(scope.data(), ScopeData::EventHandler(_)) {
            return None;
        }
        let idx = self.events.get(&(scope.span.start, scope.span.end))?;
        self.packets.get_mut(idx)
    }
    pub(super) fn remove(&mut self, idx: usize) -> Option<RootOwnerPacket> {
        self.packets.remove(&idx)
    }
}

#[cfg(test)]
mod tests;

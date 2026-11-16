//! Emit an `@vue-expect-error` owner's checks as one generated interval.
//!
//! TypeScript holds one interval per expectation. The template walk emits a
//! scope's bindings before its listener blocks, and a closure-scoped
//! component's prop checks in a later replay of that closure, so a sibling's
//! checks can fall between an owner's own. For an owner whose relocation is
//! proven, its listeners and its replayed checks are emitted right after its
//! own bindings instead: its authored render position, in the same block.
//! Root components keep `root_owner_packets`; everything else keeps its order.

use vize_carton::{FxHashMap, FxHashSet, profile};
use vize_croquis::analysis::ComponentUsage;
use vize_croquis::{BindingType, ScopeData, ScopeId, ScopeKind, TemplateExpression};
use vize_relief::{ElementNode, ElementType, ExpressionNode, PropNode, TemplateChildNode};

use super::component_prop_checker::contains_inline_function_prop_value;
use super::component_slots::{ComponentSlotCheckMeta, generate_component_slot_checks};
use super::context::{ComponentPropsContext, ScopeGenContext, ScopeMappings};
use super::empty_component_props::is_empty_props_usage;
use super::root_callback_owners::RootCallbackOwners;
use super::root_owner_packets::{RelocationSafety, stable_render_flow_with_loops};
use crate::template_diagnostic_directives::{DirectivePolicy, TemplateDiagnosticDirectives};
use crate::virtual_ts::component_reference::component_binding_reference;
use crate::virtual_ts::expressions::{
    ComponentPropCheckContext, ExpressionListEmitContext, generate_component_prop_checks,
    generate_expressions_in_enclosing_guard,
};
use crate::virtual_ts::types::{VirtualTsCheckOptions, VizeMapping};

/// What an owner emits after the last binding of its opening tag.
struct SplicePoint<'a> {
    end: u32,
    handlers: Vec<ScopeId>,
    usage: Option<(usize, &'a ComponentUsage)>,
}

#[derive(Default)]
pub(super) struct ExpectOwnerSplices<'a, 'template> {
    props: Option<ComponentPropsContext<'a, 'template>>,
    points: FxHashMap<u32, Vec<SplicePoint<'a>>>,
    handlers: FxHashSet<u32>,
    usages: FxHashSet<usize>,
}

impl<'a, 'template> ExpectOwnerSplices<'a, 'template> {
    pub(super) fn collect(
        ctx: &ComponentPropsContext<'a, 'template>,
        usages: Option<&[(usize, &'a ComponentUsage)]>,
        expressions: &FxHashMap<u32, Vec<&TemplateExpression>>,
        callback_owners: &RootCallbackOwners,
        check_options: VirtualTsCheckOptions,
        script: Option<&str>,
    ) -> Self {
        let mut result = Self::default();
        let Some(root) = ctx.template_ast else {
            return result;
        };
        if !root.source.contains("@vue-expect-error")
            || !check_options.check_template_bindings
            || ctx.legacy_vue2
            || ctx.options.strict_instance_globals
            || ctx.experimental_strict_slot_children
            || !crate::virtual_ts::script_facts::is_script_setup(ctx.summary)
        {
            return result;
        }
        // The marker is a cheap gate; ownership comes from the shared walk.
        let directives = TemplateDiagnosticDirectives::for_root(root, 0);
        let targets: FxHashSet<_> = directives
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
        let mut owners = Vec::new();
        collect_owners(&root.children, &targets, &mut owners);
        if owners.is_empty() || !stable_render_flow_with_loops(root, ctx.summary) {
            return result;
        }
        let usages: FxHashMap<_, _> = usages
            .unwrap_or_default()
            .iter()
            .map(|&(idx, usage)| (usage.start, (idx, usage)))
            .collect();
        profile!("canon.virtual_ts.expect_splice.collect", {
            crate::virtual_ts::script_facts::with_bindings(ctx.summary, |bindings| {
                let safety = RelocationSafety::new(script, bindings);
                for element in owners {
                    let usage = usages.get(&element.loc.span.start).copied();
                    let Some((scope, point)) = plan_owner(
                        ctx,
                        element,
                        usage,
                        expressions,
                        check_options,
                        &safety,
                        |name| bindings.binding_type(name),
                    ) else {
                        continue;
                    };
                    if callback_owners.has_scope(scope) {
                        continue;
                    }
                    result
                        .handlers
                        .extend(point.handlers.iter().map(|id| id.as_u32()));
                    result.usages.extend(point.usage.map(|(idx, _)| idx));
                    result.points.entry(scope).or_default().push(point);
                }
            });
        });
        for points in result.points.values_mut() {
            points.sort_unstable_by_key(|point| point.end);
        }
        if !result.points.is_empty() {
            result.props = Some(*ctx);
        }
        result
    }

    pub(super) fn has_scope(&self, scope: u32) -> bool {
        self.points.contains_key(&scope)
    }

    pub(super) fn moves_handler(&self, scope: u32) -> bool {
        self.handlers.contains(&scope)
    }

    /// Closure-scoped usages whose replayed checks moved to their owner.
    pub(super) fn moved_usages(&self) -> &FxHashSet<usize> {
        &self.usages
    }

    /// Emit `scope`'s bindings with each owner's moved checks after its own.
    /// Only called for a [`Self::has_scope`] scope; its stream is ordered.
    pub(super) fn generate_expressions(
        &self,
        ts: &mut crate::virtual_ts::TextBuf,
        mappings: &mut ScopeMappings<'_>,
        ctx: &ScopeGenContext<'_, '_>,
        scope: u32,
        indent: &str,
        enclosing_guard: Option<&str>,
    ) {
        let exprs = ctx
            .expressions_by_scope
            .get(&scope)
            .map_or(&[][..], Vec::as_slice);
        let context = ExpressionListEmitContext::new(
            ctx.skipped_expression_ranges,
            ctx.template_offset,
            indent,
            ctx.checks,
        );
        let access = ctx.template_binding_access;
        let mut first = 0;
        for point in self.points.get(&scope).into_iter().flatten() {
            // A shared guard is stripped from the whole list at once; owners
            // are unguarded, so a guarded list keeps them after it.
            let end = if enclosing_guard.is_some() {
                exprs.len()
            } else {
                exprs.partition_point(|expr| expr.start < point.end)
            }
            .max(first);
            if let Some(before) = exprs.get(first..end) {
                generate_expressions_in_enclosing_guard(
                    ts,
                    mappings,
                    before,
                    access,
                    &context,
                    enclosing_guard,
                );
            }
            first = end;
            self.emit_point(ts, mappings, ctx, point, indent);
        }
        if let Some(after) = exprs.get(first..) {
            generate_expressions_in_enclosing_guard(
                ts,
                mappings,
                after,
                access,
                &context,
                enclosing_guard,
            );
        }
    }

    fn emit_point(
        &self,
        ts: &mut crate::virtual_ts::TextBuf,
        mappings: &mut ScopeMappings<'_>,
        ctx: &ScopeGenContext<'_, '_>,
        point: &SplicePoint<'_>,
        indent: &str,
    ) {
        for &handler in &point.handlers {
            if let Some(scope) = ctx.summary.scopes.get_scope(handler) {
                super::node::generate_scope_node(ts, mappings, ctx, scope, indent);
            }
        }
        if let (Some((idx, usage)), Some(props)) = (point.usage, self.props.as_ref()) {
            emit_usage_checks(ts, mappings.navigation, props, idx, usage, indent);
        }
    }
}

/// The closure replay's usages by scope. A moved usage's replayed checks
/// already follow its own bindings in the template pass.
pub(super) fn replayed_by_scope<'u>(
    usages: &[(usize, &'u ComponentUsage)],
    moved: &FxHashSet<usize>,
) -> FxHashMap<u32, Vec<(usize, &'u ComponentUsage)>> {
    let mut by_scope: FxHashMap<u32, Vec<_>> = FxHashMap::default();
    for &(idx, usage) in usages.iter().filter(|(idx, _)| !moved.contains(idx)) {
        let scope = usage.scope_id.as_u32();
        by_scope.entry(scope).or_default().push((idx, usage));
    }
    by_scope
}

/// The per-usage body of the closure replay (`generate_scope_checks`).
fn emit_usage_checks(
    ts: &mut crate::virtual_ts::TextBuf,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ComponentPropsContext<'_, '_>,
    idx: usize,
    usage: &ComponentUsage,
    indent: &str,
) {
    let component_ref = ctx.explicit_generics.usage_reference(
        usage.start,
        component_binding_reference(
            ctx.summary,
            ctx.options,
            ctx.syntactic_type_only_imported_names,
            usage.name.as_str(),
        ),
    );
    let mut check_context = ComponentPropCheckContext::new(
        ts,
        mappings,
        ctx.template_binding_access,
        ctx.source_context(),
        indent,
    )
    .strict_v_model(ctx.strict_v_model)
    .typed_fallthrough_values(ctx.typed_fallthrough_values)
    .hoist_shared_preamble(ctx.hoist_shared_preamble);
    generate_component_prop_checks(&mut check_context, usage, idx, component_ref.as_str());
    generate_component_slot_checks(
        &mut check_context,
        usage,
        idx,
        component_ref.as_str(),
        ComponentSlotCheckMeta {
            summary: ctx.summary,
            options: ctx.options,
            syntactic_type_only_imported_names: ctx.syntactic_type_only_imported_names,
            template_ast: ctx.template_ast,
            experimental_strict_slot_children: ctx.experimental_strict_slot_children,
        },
    );
}

fn collect_owners<'n, 'a>(
    children: &'n [TemplateChildNode<'a>],
    targets: &FxHashSet<(usize, usize)>,
    owners: &mut Vec<&'n ElementNode<'a>>,
) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                let span = (
                    element.loc.span.start as usize,
                    element.loc.span.end as usize,
                );
                if targets.contains(&span) {
                    owners.push(element);
                }
                collect_owners(&element.children, targets, owners);
            }
            TemplateChildNode::If(node) => {
                for branch in &node.branches {
                    collect_owners(&branch.children, targets, owners);
                }
            }
            TemplateChildNode::IfBranch(branch) => {
                collect_owners(&branch.children, targets, owners)
            }
            TemplateChildNode::For(node) => collect_owners(&node.children, targets, owners),
            _ => {}
        }
    }
}

/// The owner's scope and what moves to its splice point, if relocation is
/// proven: its bindings are one unguarded run of that scope's ordered stream,
/// its listeners are direct child blocks, and a component's replay holds only
/// value and slot checks of static, non-callback props.
fn plan_owner<'a>(
    ctx: &ComponentPropsContext<'_, '_>,
    element: &ElementNode<'_>,
    usage: Option<(usize, &'a ComponentUsage)>,
    expressions: &FxHashMap<u32, Vec<&TemplateExpression>>,
    check_options: VirtualTsCheckOptions,
    safety: &RelocationSafety<'_>,
    binding_type: impl Fn(&str) -> Option<BindingType>,
) -> Option<(u32, SplicePoint<'a>)> {
    let (start, end) = (element.loc.span.start, element.loc.span.end);
    let component = match element.tag_type {
        ElementType::Element => false,
        ElementType::Component => true,
        ElementType::Slot | ElementType::Template => return None,
    };
    let props_supported = element.props.iter().all(|prop| match prop {
        PropNode::Attribute(_) => true,
        PropNode::Directive(directive) => match directive.name {
            "for" => true,
            // A component's listeners also have replayed contract checks.
            "on" if component => false,
            "bind" | "on" => matches!(
                directive.arg.as_ref(),
                Some(ExpressionNode::Simple(arg)) if arg.is_static
            ),
            _ => false,
        },
    });
    if !props_supported || (component != usage.is_some()) {
        return None;
    }
    // Every binding inside the opening tag belongs to one scope.
    let mut owned = None;
    for (&scope, exprs) in expressions {
        let first = exprs
            .iter()
            .position(|expr| start <= expr.start && expr.start < end);
        let Some(first) = first else {
            continue;
        };
        let kind = ctx.summary.scopes.get_scope(ScopeId::new(scope))?.kind;
        if matches!(kind, ScopeKind::EventHandler) {
            continue;
        }
        let run = exprs.get(first..)?;
        let len = run.iter().take_while(|expr| expr.start < end).count();
        let ordered = exprs.windows(2).all(|pair| match pair {
            [a, b] => a.start <= b.start,
            _ => true,
        });
        if owned.is_some()
            || !ordered
            || run
                .iter()
                .take(len)
                .any(|expr| expr.end > end || expr.vif_guard.is_some())
        {
            return None;
        }
        owned = Some((scope, kind));
    }
    let (scope, kind) = owned?;
    if !matches!(
        kind,
        ScopeKind::VFor
            | ScopeKind::VSlot
            | ScopeKind::ScriptSetup
            | ScopeKind::JsGlobalUniversal
            | ScopeKind::JsGlobalBrowser
            | ScopeKind::JsGlobalNode
            | ScopeKind::VueGlobal
    ) {
        return None;
    }
    let mut handlers = Vec::new();
    for candidate in ctx.summary.scopes.iter() {
        let ScopeData::EventHandler(data) = candidate.data() else {
            continue;
        };
        if candidate.span.start < start || candidate.span.end > end {
            continue;
        }
        let handler_exprs = expressions
            .get(&candidate.id.as_u32())
            .map_or(&[][..], Vec::as_slice);
        if candidate.parent().map(|parent| parent.as_u32()) != Some(scope)
            || data.event_name_is_dynamic
            || handler_exprs.iter().any(|expr| expr.vif_guard.is_some())
            || !data
                .handler_expression
                .as_ref()
                .is_some_and(|handler| safety.deferred_handler(handler.as_str()))
        {
            return None;
        }
        handlers.push(candidate.id);
    }
    let usage = match usage {
        Some((idx, usage)) => {
            let movable = matches!(kind, ScopeKind::VFor | ScopeKind::VSlot)
                && check_options.check_props
                && usage.end == end
                && usage.scope_id.as_u32() == scope
                && usage.vif_guard.is_none()
                && usage.events.is_empty()
                && usage.slots.is_empty()
                && usage.spread_props.is_empty()
                && !is_empty_props_usage(usage)
                && ctx.explicit_generics.alias(usage.start).is_none()
                && !ctx.dynamic_aliases.is_scoped(usage.start)
                && binding_type(usage.name.as_str()) == Some(BindingType::SetupConst)
                && usage.props.iter().all(|prop| {
                    !prop.name_is_dynamic
                        && prop.value.as_ref().is_none_or(|value| {
                            !prop.is_dynamic || !contains_inline_function_prop_value(value)
                        })
                });
            if !movable || !handlers.is_empty() {
                return None;
            }
            Some((idx, usage))
        }
        None if handlers.is_empty() || !check_options.check_event_handlers() => return None,
        None => None,
    };
    Some((
        scope,
        SplicePoint {
            end,
            handlers,
            usage,
        },
    ))
}

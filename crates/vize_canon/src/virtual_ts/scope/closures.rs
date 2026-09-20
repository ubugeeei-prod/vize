use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use vize_carton::{FxHashMap, FxHashSet, String, profile};
use vize_croquis::{Croquis, ScopeId, ScopeKind};

use crate::virtual_ts::expressions::{
    ExpressionListEmitContext, TemplateValueCheckTables, generate_expressions,
};
use crate::virtual_ts::{VizeSemanticLink, types::VizeMapping};

use super::component_event_navigation::emit_event_references;
use super::component_prop_expressions::collect_component_prop_expression_ranges;
use super::component_props::{collect_checkable_usages, generate_component_props};
use super::context::{ComponentPropsContext, ScopeGenContext, ScopeGenerationOptions};
use super::globals::{generate_instance_global_refs, generate_undefined_refs};
pub(super) use super::node::generate_scope_node;
use super::slot_outlet_props::{SlotOutletChecks, generate_scope_slot_outlet_checks};
use super::vif_guard::common_vif_guard_prefix_outside_v_for_scope;

pub(crate) fn generate_scope_closures(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    semantic_links: &mut Vec<VizeSemanticLink>,
    summary: &Croquis,
    template_binding_access: &TemplateBindingAccess,
    template_offset: u32,
    options: ScopeGenerationOptions<'_, '_>,
) {
    let check_options = options.check_options;
    let virtual_ts_options = options.virtual_ts_options;
    let check_tables = TemplateValueCheckTables::collect(summary, &options);
    let checks = check_tables.as_checks(options.template_ast.map(|root| root.source));

    if check_options.check_props
        && check_options.check_unknown_props
        && !options.legacy_vue2
        && let Some(root) = options.template_ast
    {
        super::native_prop_names::emit(ts, mappings, root, template_offset);
    }
    super::dynamic_component::emit_dynamic_component_aliases(ts, summary, options.template_ast);

    let expressions_by_scope: FxHashMap<u32, Vec<_>> =
        profile!("canon.virtual_ts.group_template_expressions", {
            let mut expressions_by_scope: FxHashMap<u32, Vec<_>> = FxHashMap::default();
            let pattern_ranges = super::patterns::expression_ranges(summary);
            for expr in &summary.template_expressions {
                if pattern_ranges.contains(&(expr.start, expr.end)) {
                    continue;
                }
                expressions_by_scope
                    .entry(expr.scope_id.as_u32())
                    .or_default()
                    .push(expr);
            }
            expressions_by_scope
        });
    let slot_outlets = if check_options.check_props {
        profile!("canon.virtual_ts.collect_slot_outlets", {
            SlotOutletChecks::collect(summary, options.template_ast)
        })
    } else {
        SlotOutletChecks::default()
    };
    slot_outlets.emit_helpers(ts);
    let skipped_expression_ranges =
        profile!("canon.virtual_ts.component_prop_expression_ranges", {
            collect_component_prop_expression_ranges(summary, &options, &slot_outlets)
        });

    let children_map: FxHashMap<u32, Vec<ScopeId>> =
        profile!("canon.virtual_ts.build_scope_tree", {
            let mut children_map: FxHashMap<u32, Vec<ScopeId>> = FxHashMap::default();
            for scope in summary.scopes.iter() {
                if let Some(parent_id) = scope.parent() {
                    children_map
                        .entry(parent_id.as_u32())
                        .or_default()
                        .push(scope.id);
                }
            }
            children_map
        });

    let vfor_enclosing_guards: FxHashMap<u32, String> =
        profile!("canon.virtual_ts.vfor_enclosing_guards", {
            if !check_options.check_template_bindings {
                FxHashMap::default()
            } else {
                summary
                    .scopes
                    .iter()
                    .filter(|scope| matches!(scope.kind, ScopeKind::VFor))
                    .filter_map(|scope| {
                        let scope_id = scope.id.as_u32();
                        expressions_by_scope
                            .get(&scope_id)
                            .and_then(|exprs| {
                                common_vif_guard_prefix_outside_v_for_scope(exprs, scope)
                            })
                            .map(|guard| (scope_id, guard))
                    })
                    .collect()
            }
        });

    let nested_scope_ids: FxHashSet<ScopeId> =
        profile!("canon.virtual_ts.collect_nested_scope_ids", {
            summary
                .scopes
                .iter()
                .filter(|scope| {
                    scope.parent().is_some_and(|pid| {
                        summary.scopes.get_scope(pid).is_some_and(|parent| {
                            matches!(
                                parent.kind,
                                ScopeKind::VFor
                                    | ScopeKind::VSlot
                                    | ScopeKind::VMatch
                                    | ScopeKind::VWhen
                            )
                        })
                    })
                })
                .map(|scope| scope.id)
                .collect()
        });

    if check_options.check_template_bindings {
        profile!(
            "canon.virtual_ts.instance_global_refs",
            generate_instance_global_refs(ts, mappings, summary, template_offset, &options)
        );
    }
    // Under `checkRequiredFallthroughAttributes` the fallthrough root's
    // required props are forwarded to this component's parent, so the root
    // usage must not also demand them here.
    let relaxed_required_usage_starts: FxHashSet<u32> = if check_options.fallthrough_attributes
        && check_options.check_required_fallthrough_attributes
        && !options.legacy_vue2
    {
        crate::virtual_ts::generator::fallthrough_component_root_starts(
            summary,
            options.template_ast,
        )
        .into_iter()
        .collect()
    } else {
        FxHashSet::default()
    };
    let props_ctx = ComponentPropsContext {
        summary,
        template_ast: options.template_ast,
        template_source: options.template_ast.map(|root| root.source),
        children_map: &children_map,
        vfor_enclosing_guards: &vfor_enclosing_guards,
        template_binding_access,
        syntactic_type_only_imported_names: options.syntactic_type_only_imported_names,
        template_offset,
        options: virtual_ts_options,
        preserve_event_navigation: options.preserve_event_navigation,
        check_unknown_events: check_options.check_unknown_events && check_options.check_emits,
        component_binding_check: options.component_binding_check,
        legacy_vue2: options.legacy_vue2,
        check_unknown_props: check_options.check_unknown_props,
        experimental_strict_slot_children: options.experimental_strict_slot_children,
        relaxed_required_usage_starts: &relaxed_required_usage_starts,
    };
    let usages = check_options
        .check_props
        .then(|| collect_checkable_usages(&props_ctx));
    if let Some(usages) = &usages {
        emit_event_references(ts, mappings, &props_ctx, usages);
    }
    for scope in summary.scopes.iter() {
        let scope_id = scope.id.as_u32();
        let ctx = ScopeGenContext {
            summary,
            virtual_ts_options,
            expressions_by_scope: &expressions_by_scope,
            skipped_expression_ranges: &skipped_expression_ranges,
            children_map: &children_map,
            slot_outlets: &slot_outlets,
            template_binding_access,
            syntactic_type_only_imported_names: options.syntactic_type_only_imported_names,
            checks,
            template_ast: options.template_ast,
            template_source: options.template_ast.map(|root| root.source),
            template_offset,
            check_options,
            legacy_vue2: options.legacy_vue2,
        };

        if nested_scope_ids.contains(&scope.id) {
            continue;
        }

        if matches!(
            scope.kind,
            ScopeKind::JsGlobalUniversal
                | ScopeKind::JsGlobalBrowser
                | ScopeKind::JsGlobalNode
                | ScopeKind::VueGlobal
        ) {
            if let Some(exprs) = expressions_by_scope.get(&scope_id)
                && check_options.check_template_bindings
            {
                generate_expressions(
                    ts,
                    mappings,
                    exprs,
                    template_binding_access,
                    &ExpressionListEmitContext::new(
                        &skipped_expression_ranges,
                        template_offset,
                        "  ",
                        checks,
                    ),
                );
            }
            generate_scope_slot_outlet_checks(ts, mappings, scope_id, &ctx, "  ");
            continue;
        }
        profile!(
            "canon.virtual_ts.scope_node",
            generate_scope_node(ts, mappings, &ctx, scope, "  ")
        );
    }

    if check_options.check_template_bindings {
        profile!("canon.virtual_ts.undefined_refs", {
            generate_undefined_refs(
                ts,
                mappings,
                summary,
                template_binding_access,
                template_offset,
                &options,
            )
        });
    }
    if let Some(usages) = &usages {
        profile!(
            "canon.virtual_ts.component_props",
            generate_component_props(ts, mappings, semantic_links, &props_ctx, usages)
        );
    }
    slot_outlets.emit_result(ts, summary, None, "  ");
}

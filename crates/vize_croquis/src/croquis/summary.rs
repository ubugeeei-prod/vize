//! Reusable semantic summary derived from a [`Croquis`].
//!
//! This module gives downstream crates one stable place to consume the facts
//! already collected by croquis without re-counting public fields differently.

use super::Croquis;
use super::template::TemplateExpressionKind;
use crate::scope::ScopeKind;
pub use crate::semantic::CroquisSemanticSummary;

impl CroquisSemanticSummary {
    /// Build a summary from the current croquis facts.
    pub fn from_croquis(croquis: &Croquis) -> Self {
        let mut summary = Self {
            scope_count: croquis.scopes.len(),
            symbol_count: croquis.symbols.len(),
            script_binding_count: croquis.bindings.bindings.len(),
            prop_alias_count: croquis.bindings.props_aliases.len(),
            macro_call_count: croquis.macros.all_calls().len(),
            prop_definition_count: croquis.macros.props().len(),
            emit_definition_count: croquis.macros.emits().len(),
            emit_call_count: croquis.macros.emit_calls().len(),
            model_definition_count: croquis.macros.models().len(),
            exposed_binding_count: croquis.macros.exposes().len(),
            slot_definition_count: croquis.macros.slots().len(),
            top_level_await_count: croquis.macros.top_level_awaits().len(),
            hoist_count: croquis.hoists.count(),
            reactive_source_count: croquis.reactivity.count(),
            reactivity_loss_count: croquis.reactivity.losses().len(),
            race_condition_count: croquis.race_conditions.risks().len(),
            provide_count: croquis.provide_inject.provides().len(),
            inject_count: croquis.provide_inject.injects().len(),
            destructured_inject_count: croquis.provide_inject.destructured_injects().count(),
            composable_count: croquis.provide_inject.composables().len(),
            setup_context_violation_count: croquis.setup_context.count(),
            used_component_count: croquis.used_components.len(),
            component_registration_count: croquis.component_registrations.len(),
            component_usage_count: croquis.component_usages.len(),
            used_directive_count: croquis.used_directives.len(),
            template_expression_count: croquis.template_expressions.len(),
            element_id_count: croquis.element_ids.len(),
            undefined_ref_count: croquis.undefined_refs.len(),
            unused_binding_count: croquis.unused_bindings.len(),
            type_export_count: croquis.type_exports.len(),
            invalid_export_count: croquis.invalid_exports.len(),
            import_statement_count: croquis.import_statements.len(),
            re_export_count: croquis.re_exports.len(),
            binding_span_count: croquis.binding_spans.len(),
            has_multiple_roots: croquis.template_info.has_multiple_roots(),
            uses_attrs: croquis.template_info.uses_attrs,
            binds_attrs_explicitly: croquis.template_info.binds_attrs_explicitly,
            inherit_attrs_disabled: croquis.template_info.inherit_attrs_disabled,
            ..Default::default()
        };

        for scope in croquis.scopes.iter() {
            summary.scope_binding_count += scope.binding_count();
            match scope.kind {
                ScopeKind::VFor => {
                    summary.template_scope_count += 1;
                    summary.v_for_scope_count += 1;
                }
                ScopeKind::VSlot => {
                    summary.template_scope_count += 1;
                    summary.v_slot_scope_count += 1;
                }
                ScopeKind::EventHandler => {
                    summary.template_scope_count += 1;
                    summary.event_handler_scope_count += 1;
                }
                ScopeKind::Callback => summary.callback_scope_count += 1,
                _ => {}
            }
        }

        for symbol in croquis.symbols.iter() {
            summary.symbol_reference_count += symbol.references.len();
            if !symbol.is_used() {
                summary.unused_symbol_count += 1;
            }
        }

        for usage in &croquis.component_usages {
            summary.passed_prop_count += usage.props.len();
            summary.event_listener_count += usage.events.len();
            summary.slot_usage_count += usage.slots.len();
            if usage.has_spread_attrs {
                summary.spread_attr_component_count += 1;
            }
        }

        for expression in &croquis.template_expressions {
            match expression.kind {
                TemplateExpressionKind::VIf => summary.v_if_expression_count += 1,
                TemplateExpressionKind::VModel => summary.v_model_expression_count += 1,
                _ => {}
            }
        }

        for id in &croquis.element_ids {
            if id.is_static {
                summary.static_element_id_count += 1;
            } else {
                summary.dynamic_element_id_count += 1;
            }
            if id.kind.is_definition() {
                summary.id_definition_count += 1;
            } else {
                summary.id_reference_count += 1;
            }
        }

        summary
    }
}

impl Croquis {
    /// Return a reusable aggregate summary of the semantic facts in this croquis.
    #[inline]
    pub fn semantic_summary(&self) -> CroquisSemanticSummary {
        CroquisSemanticSummary::from_croquis(self)
    }
}

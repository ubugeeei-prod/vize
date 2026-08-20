//! Element visiting orchestrator.
//!
//! Two-pass directive processing: first pass collects v-for/v-slot scope
//! info (which must be entered before other directives), second pass
//! processes v-bind, v-if, v-show, v-model, v-on in the correct scope.

mod bounds;
mod first_pass;
mod scopes;
mod second_pass;
mod v_for_scope;

use crate::croquis::ComponentUsage;
use crate::drawer::Drawer;
use crate::drawer::helpers::{ConditionalKind, is_component_tag};
use vize_carton::{CompactString, SmallVec, profile};
use vize_relief::ElementNode;

impl Drawer {
    /// Visit element node.
    ///
    /// Orchestrates directive processing, scope management, and child traversal.
    pub(in crate::drawer) fn visit_element(
        &mut self,
        el: &ElementNode<'_>,
        scope_vars: &mut Vec<CompactString>,
    ) {
        let tag = el.tag.as_str();
        let is_component = is_component_tag(tag);
        let mut subtree_end = None;

        let component_usage_name = if is_component {
            Some(CompactString::new(tag))
        } else {
            self.dynamic_component_target(el, tag)
        };

        if self.options.track_usage {
            if let Some(component_name) = component_usage_name.as_ref() {
                self.croquis.used_components.insert(component_name.clone());
            } else if is_component {
                self.croquis.used_components.insert(CompactString::new(tag));
            }
        }

        let mut component_usage = self.start_component_usage(el, component_usage_name.as_ref());
        let directive_state = self.collect_element_directive_state(el, &mut subtree_end);
        let vif_condition = self.apply_element_conditional(directive_state.conditional);
        // Vue evaluates v-if before same-element v-for aliases exist. Keep the
        // conditional expression in the incoming scope, while retaining its
        // guard metadata for virtual TypeScript control-flow narrowing.
        let vif_guard_pushed = self.push_element_vif_guard(vif_condition.as_ref());
        self.process_element_conditional_directive(el, scope_vars);
        if let Some(ref mut usage) = component_usage {
            usage.vif_guard = self.current_vif_guard();
        }

        let for_vars_count = self.enter_element_for_scope(
            directive_state.for_scope,
            directive_state.key_expression,
            scope_vars,
        );
        // A dynamic v-slot name can reference a same-element v-for alias, but
        // not the slot props that the v-slot value declares for its children.
        self.process_dynamic_slot_argument(el, scope_vars);
        let v_scope_vars_count = self.enter_element_v_scope(directive_state.v_scope, scope_vars);

        if let Some(ref mut usage) = component_usage {
            usage.scope_id = self.croquis.scopes.current_id();
        }

        // Element attrs execute outside the v-slot props they define. They can
        // still reference same-element v-for aliases and petite-vue v-scope
        // bindings.
        profile!("croquis.template.element_ids", self.collect_element_ids(el));

        self.process_element_directives(el, scope_vars, is_component, tag);
        self.check_element_directive_refs(el, scope_vars);

        let slot_vars_count = self.enter_element_slot_scope(
            directive_state.slot_scope,
            is_component,
            tag,
            el,
            &mut subtree_end,
            scope_vars,
        );

        if is_component {
            self.parent_component_stack.push(CompactString::new(tag));
        }
        self.visit_element_children(el, scope_vars);
        if is_component {
            self.parent_component_stack.pop();
        }

        if vif_guard_pushed {
            self.pop_element_vif_guard();
        }

        self.exit_element_slot_scope(slot_vars_count, scope_vars);
        self.exit_element_v_scope(v_scope_vars_count, scope_vars);
        self.exit_element_for_scope(for_vars_count, scope_vars);

        if let Some(ref mut usage) = component_usage {
            profile!(
                "croquis.template.component.props_events",
                self.collect_component_props_events(el, tag, usage)
            );
            profile!(
                "croquis.template.component.slots",
                self.collect_component_slots(el, usage)
            );
        }

        if let Some(usage) = component_usage {
            self.croquis.component_usages.push(usage);
        }
    }

    fn start_component_usage(
        &self,
        el: &ElementNode<'_>,
        name: Option<&CompactString>,
    ) -> Option<ComponentUsage> {
        let name = name?;
        (self.options.track_usage).then(|| ComponentUsage {
            name: name.clone(),
            start: el.loc.start.offset,
            end: el.loc.end.offset,
            props: SmallVec::new(),
            events: SmallVec::new(),
            slots: SmallVec::new(),
            has_spread_attrs: false,
            spread_props: SmallVec::new(),
            scope_id: crate::scope::ScopeId::ROOT,
            vif_guard: None,
        })
    }

    fn apply_element_conditional(
        &mut self,
        conditional: Option<(ConditionalKind, Option<CompactString>)>,
    ) -> Option<CompactString> {
        match conditional {
            Some((ConditionalKind::If, cond)) => {
                self.vif_branch_conditions.clear();
                let guard = crate::drawer::helpers::build_branch_guard(
                    &self.vif_branch_conditions,
                    cond.as_deref(),
                );
                if let Some(cond) = cond {
                    self.vif_branch_conditions.push(cond);
                }
                guard
            }
            Some((ConditionalKind::ElseIf, cond)) => {
                let guard = crate::drawer::helpers::build_branch_guard(
                    &self.vif_branch_conditions,
                    cond.as_deref(),
                );
                if let Some(cond) = cond {
                    self.vif_branch_conditions.push(cond);
                }
                guard
            }
            Some((ConditionalKind::Else, _)) => {
                let guard =
                    crate::drawer::helpers::build_branch_guard(&self.vif_branch_conditions, None);
                self.vif_branch_conditions.clear();
                guard
            }
            None => {
                // A non-conditional element breaks any open v-if chain.
                self.vif_branch_conditions.clear();
                None
            }
        }
    }

    fn push_element_vif_guard(&mut self, condition: Option<&CompactString>) -> bool {
        let Some(condition) = condition else {
            return false;
        };

        self.vif_guard_stack.push(condition.clone());
        // Stack changed: recompute the memoized joined guard.
        self.refresh_vif_guard_cache();
        true
    }

    fn pop_element_vif_guard(&mut self) {
        self.vif_guard_stack.pop();
        // Stack changed: recompute the memoized joined guard.
        self.refresh_vif_guard_cache();
    }

    fn visit_element_children(
        &mut self,
        el: &ElementNode<'_>,
        scope_vars: &mut Vec<CompactString>,
    ) {
        // Children form a fresh sibling group, so the running `v-if` branch
        // chain is saved and reset here and restored afterwards.
        let saved_branch_conditions = std::mem::take(&mut self.vif_branch_conditions);
        for child in el.children.iter() {
            self.visit_template_child(child, scope_vars);
        }
        self.vif_branch_conditions = saved_branch_conditions;
    }
}

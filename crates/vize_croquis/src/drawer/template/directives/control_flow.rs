//! Control flow directive handling.
//!
//! Processes v-if/v-else-if/v-else and v-for nodes at the template AST level
//! (as opposed to directive-level processing in `visit_element`).

use crate::ScopeBinding;
use crate::drawer::Drawer;
use crate::drawer::helpers::build_branch_guard;
use crate::scope::VForScopeData;
use vize_carton::{CompactString, SmallVec, profile, smallvec};
use vize_relief::BindingType;
use vize_relief::{ExpressionNode, ForNode, IfNode, PropNode};

impl Drawer {
    /// Visit if node.
    pub(in crate::drawer) fn visit_if(
        &mut self,
        if_node: &IfNode<'_>,
        scope_vars: &mut Vec<CompactString>,
    ) {
        let mut previous_conditions = SmallVec::<[CompactString; 4]>::new();

        for branch in if_node.branches.iter() {
            if self.options.detect_undefined
                && self.script_drawn
                && let Some(ref cond) = branch.condition
            {
                profile!(
                    "croquis.template.v_if.condition_refs",
                    self.check_expression_refs(cond, scope_vars)
                );
            }

            if self.options.detect_undefined
                && self.script_drawn
                && let Some(PropNode::Directive(dir)) = &branch.user_key
                && let Some(ref exp) = dir.exp
            {
                profile!(
                    "croquis.template.v_if.key_refs",
                    self.check_expression_refs(exp, scope_vars)
                );
            }

            let current_condition = branch.condition.as_ref().map(|cond| match cond {
                ExpressionNode::Simple(s) => s.content.as_str(),
                ExpressionNode::Compound(c) => c.loc.source.as_str(),
            });

            let branch_guard =
                build_branch_guard(previous_conditions.as_slice(), current_condition);
            let guard_pushed = if let Some(ref guard) = branch_guard {
                self.vif_guard_stack.push(guard.clone());
                // Stack changed: recompute the memoized joined guard.
                self.refresh_vif_guard_cache();
                true
            } else {
                false
            };

            // Branch children are a fresh sibling group for flat v-if chains.
            let saved_branch_conditions = std::mem::take(&mut self.vif_branch_conditions);
            for child in branch.children.iter() {
                self.visit_template_child(child, scope_vars);
            }
            self.vif_branch_conditions = saved_branch_conditions;

            // Pop v-if guard
            if guard_pushed {
                self.vif_guard_stack.pop();
                // Stack changed: recompute the memoized joined guard.
                self.refresh_vif_guard_cache();
            }

            if let Some(condition) = current_condition {
                previous_conditions.push(CompactString::new(condition));
            }
        }
    }

    /// Visit for node.
    pub(in crate::drawer) fn visit_for(
        &mut self,
        for_node: &ForNode<'_>,
        scope_vars: &mut Vec<CompactString>,
    ) {
        let vars_added = profile!(
            "croquis.template.v_for.extract_vars",
            self.extract_for_vars(for_node)
        );
        let vars_count = vars_added.len();

        if self.options.analyze_template_scopes && !vars_added.is_empty() {
            let source_content = match &for_node.source {
                ExpressionNode::Simple(s) => CompactString::new(s.content.as_str()),
                ExpressionNode::Compound(c) => CompactString::new(c.loc.source.as_str()),
            };

            let value_alias = vars_added
                .first()
                .cloned()
                .unwrap_or_else(|| CompactString::const_new("_"));

            self.croquis.scopes.enter_v_for_scope(
                VForScopeData {
                    value_alias: value_alias.clone(),
                    value_bindings: smallvec![value_alias],
                    key_alias: vars_added.get(1).cloned(),
                    index_alias: vars_added.get(2).cloned(),
                    source: source_content,
                    key_expression: None,
                },
                for_node.loc.start.offset,
                for_node.loc.end.offset,
            );
            // Entering a v-for scope: O(1) flag read by `is_in_vfor_scope`.
            self.vfor_depth += 1;
            for var in &vars_added {
                self.croquis
                    .scopes
                    .add_binding(var.clone(), ScopeBinding::new(BindingType::SetupConst, 0));
            }
        }

        for var in vars_added {
            scope_vars.push(var);
        }

        if self.options.detect_undefined && self.script_drawn {
            profile!(
                "croquis.template.v_for.source_refs",
                self.check_expression_refs(&for_node.source, scope_vars)
            );
        }

        // v-for children are a fresh sibling group for flat v-if chains.
        let saved_branch_conditions = std::mem::take(&mut self.vif_branch_conditions);
        for child in for_node.children.iter() {
            self.visit_template_child(child, scope_vars);
        }
        self.vif_branch_conditions = saved_branch_conditions;

        for _ in 0..vars_count {
            scope_vars.pop();
        }
        if self.options.analyze_template_scopes && vars_count > 0 {
            self.croquis.scopes.exit_scope();
            // Pairs with the increment at v-for scope enter above.
            self.vfor_depth -= 1;
        }
    }

    /// Extract variables from v-for expression.
    fn extract_for_vars(&self, for_node: &ForNode<'_>) -> Vec<CompactString> {
        let mut vars = Vec::new();

        if let Some(ExpressionNode::Simple(exp)) = &for_node.value_alias {
            vars.push(exp.content.clone());
        }

        if let Some(ExpressionNode::Simple(exp)) = &for_node.key_alias {
            vars.push(exp.content.clone());
        }

        if let Some(ExpressionNode::Simple(exp)) = &for_node.object_index_alias {
            vars.push(exp.content.clone());
        }

        vars
    }
}

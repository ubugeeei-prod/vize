//! Control flow directive handling.
//!
//! Processes v-if/v-else-if/v-else and v-for nodes at the template AST level
//! (as opposed to directive-level processing in `visit_element`).

use crate::ScopeBinding;
use crate::drawer::Drawer;
use crate::drawer::helpers::{VForScopeAliases, build_branch_guard, parse_v_for_scope_expression};
use crate::scope::VForScopeData;
use vize_carton::{CompactString, SmallVec, appends, profile};
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
                && let Some(ref cond) = branch.condition
            {
                profile!(
                    "croquis.template.v_if.condition_refs",
                    self.check_expression_refs(cond, scope_vars)
                );
            }

            if self.options.detect_undefined
                && let Some(PropNode::Directive(dir)) = &branch.user_key
                && let Some(ref exp) = dir.exp
            {
                profile!(
                    "croquis.template.v_if.key_refs",
                    self.check_expression_refs(exp, scope_vars)
                );
            }

            let compound_condition;
            let current_condition = match branch.condition.as_ref() {
                Some(ExpressionNode::Simple(s)) => Some(s.content),
                Some(ExpressionNode::Compound(c)) => {
                    compound_condition =
                        CompactString::new(c.loc.span.slice(&self.template_source));
                    Some(compound_condition.as_str())
                }
                None => None,
            };

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
        let aliases = self.options.analyze_template_scopes.then(|| {
            profile!(
                "croquis.template.v_for.parse_scope_aliases",
                self.extract_for_scope_aliases(for_node)
            )
        });
        let vars_added = aliases
            .as_ref()
            .and_then(Option::as_ref)
            .map(v_for_scope_bindings)
            .unwrap_or_else(|| {
                profile!(
                    "croquis.template.v_for.extract_vars",
                    self.extract_for_vars(for_node)
                )
            });
        let vars_count = vars_added.len();

        if let Some(Some(aliases)) = aliases {
            let scope_id = self.croquis.scopes.enter_v_for_scope(
                VForScopeData {
                    value_alias: aliases.value_pattern,
                    value_bindings: aliases.value_bindings,
                    key_alias: aliases.key_alias,
                    index_alias: aliases.index_alias,
                    source: aliases.source,
                    key_expression: None,
                },
                for_node.loc.span.start,
                for_node.loc.span.end,
            );
            self.croquis
                .scopes
                .set_v_for_source_offset(scope_id, for_node.source.loc().span.start);
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

        if self.options.detect_undefined {
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
            vars.push(exp.content.into());
        }

        if let Some(ExpressionNode::Simple(exp)) = &for_node.key_alias {
            vars.push(exp.content.into());
        }

        if let Some(ExpressionNode::Simple(exp)) = &for_node.object_index_alias {
            vars.push(exp.content.into());
        }

        vars
    }

    fn extract_for_scope_aliases(&self, for_node: &ForNode<'_>) -> Option<VForScopeAliases> {
        let value = for_node
            .value_alias
            .as_ref()
            .map(|alias| expression_content(alias, &self.template_source))?;
        let source = expression_content(&for_node.source, &self.template_source);
        let alias = match (&for_node.key_alias, &for_node.object_index_alias) {
            (None, None) => CompactString::new(value),
            (key, index) => {
                let key = key
                    .as_ref()
                    .map(|alias| expression_content(alias, &self.template_source))
                    .unwrap_or("");
                let index = index
                    .as_ref()
                    .map(|alias| expression_content(alias, &self.template_source))
                    .unwrap_or("");
                let mut alias = CompactString::new("");
                appends!(alias, "(", value, ", ", key, ", ", index, ")");
                alias
            }
        };
        let mut expr = CompactString::new("");
        appends!(expr, &alias, " in ", source);
        parse_v_for_scope_expression(&expr)
    }
}

fn v_for_scope_bindings(aliases: &VForScopeAliases) -> Vec<CompactString> {
    let mut bindings = Vec::with_capacity(
        aliases.value_bindings.len()
            + usize::from(aliases.key_alias.is_some())
            + usize::from(aliases.index_alias.is_some()),
    );
    bindings.extend(aliases.value_bindings.iter().cloned());
    if let Some(key) = &aliases.key_alias {
        bindings.push(key.clone());
    }
    if let Some(index) = &aliases.index_alias {
        bindings.push(index.clone());
    }
    bindings
}

fn expression_content<'a>(exp: &'a ExpressionNode<'_>, source: &'a str) -> &'a str {
    match exp {
        ExpressionNode::Simple(s) => s.content,
        ExpressionNode::Compound(c) => c.loc.span.slice(source),
    }
}

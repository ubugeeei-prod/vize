//! Original branch callbacks and child traversal.

use super::{LintVisitor, parse_slot_scope_variables, parse_v_for_variables};
use vize_l0::{CompactString, profile};
use vize_relief::{ElementNode, PropNode};

impl<'a, 'ctx, 'rules> LintVisitor<'a, 'ctx, 'rules> {
    #[inline]
    pub(super) fn visit_if(&mut self, if_node: &vize_relief::IfNode<'a>) {
        // Run if checks
        let keep_mask = self.keep_mask;
        profile!("patina.rules.check_if", {
            for (index, (rule, rule_name)) in self
                .rules
                .iter()
                .zip(self.rule_names.iter().copied())
                .enumerate()
            {
                if !Self::rule_active(keep_mask, index) {
                    continue;
                }
                self.ctx.current_rule = rule_name;
                rule.check_if(self.ctx, if_node);
            }
        });

        // Visit branches
        for branch in if_node.branches.iter() {
            for child in branch.children.iter() {
                self.visit_child(child);
            }
        }
    }

    #[inline]
    pub(super) fn visit_for(&mut self, for_node: &vize_relief::ForNode<'a>) {
        // Run for checks
        let keep_mask = self.keep_mask;
        profile!("patina.rules.check_for", {
            for (index, (rule, rule_name)) in self
                .rules
                .iter()
                .zip(self.rule_names.iter().copied())
                .enumerate()
            {
                if !Self::rule_active(keep_mask, index) {
                    continue;
                }
                self.ctx.current_rule = rule_name;
                rule.check_for(self.ctx, for_node);
            }
        });

        // Visit children
        for child in for_node.children.iter() {
            self.visit_child(child);
        }
    }
    /// Extract variable names from v-for directive on an element
    #[inline]
    pub(super) fn extract_v_for_vars(&self, el: &ElementNode<'a>) -> Vec<CompactString> {
        for prop in el.props.iter() {
            if let PropNode::Directive(dir) = prop
                && dir.name == "for"
                && let Some(exp) = &dir.exp
            {
                return parse_v_for_variables(exp);
            }
        }
        Vec::new()
    }

    #[inline]
    pub(super) fn extract_slot_scope_vars(&self, el: &ElementNode<'a>) -> Vec<CompactString> {
        for prop in el.props.iter() {
            if let PropNode::Directive(dir) = prop
                && dir.name == "slot"
                && let Some(exp) = &dir.exp
            {
                return parse_slot_scope_variables(exp);
            }
        }
        Vec::new()
    }
}

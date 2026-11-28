//! Root callbacks in registry order.

use super::LintVisitor;
use crate::context::CallbackSite;
use vize_l0::profile;
use vize_relief::RootNode;

impl<'a, 'ctx, 'rules> LintVisitor<'a, 'ctx, 'rules> {
    /// Visit the root node and traverse the AST.
    ///
    /// Suppression directives are the caller's: the SFC facade prescans the
    /// L1 comment directives before this walk, so they already suppress
    /// diagnostics of `run_on_template` rules, which fire before any comment
    /// is visited.
    #[inline]
    pub fn visit_root(&mut self, root: &RootNode<'a>) {
        // Run template-level checks under one profiling span. Rule dispatch
        // happens for every file, so profiling once around the callback batch is
        // cheaper than creating a span for every individual rule callback.
        let selection = self.selection;
        profile!("patina.rules.run_on_template", {
            Self::callback_rules(
                self.ctx,
                self.rules,
                self.rule_names,
                selection,
                &self.indexed_plans.template,
                CallbackSite::Template,
                |_, rule, ctx| rule.run_on_template(ctx, root),
            );
        });

        // Visit children
        for child in root.children.iter() {
            self.visit_child(child);
        }
    }
}

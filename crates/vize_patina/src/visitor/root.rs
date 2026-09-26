//! Root callbacks and suppression registration in registry order.

use super::{LintVisitor, element_has_directive};
use crate::context::LintContext;
use vize_l0::directive::{DirectiveKind, parse_level_severity, parse_vize_directive};
use vize_l0::profile;
use vize_relief::{ElementNode, RootNode, SourceLocation, TemplateChildNode};

impl<'a, 'ctx, 'rules> LintVisitor<'a, 'ctx, 'rules> {
    /// Visit the root node and traverse the AST
    #[inline]
    pub fn visit_root(&mut self, root: &RootNode<'a>) {
        self.visit_root_with_template_hook(root, &mut |_, _| false);
    }

    /// Replace a template-level callback at its original registry position.
    /// A migrated document hook shares the suppression pre-scan and preserves
    /// ordering relative to template checks that still use the Relief visitor.
    pub(crate) fn visit_root_with_template_hook(
        &mut self,
        root: &RootNode<'a>,
        hook: &mut impl FnMut(usize, &mut LintContext<'a>) -> bool,
    ) {
        // Pre-scan suppression directives so they can suppress diagnostics
        // produced by `run_on_template` rules (which fire before per-element
        // traversal would register them). Without this pass, directives are
        // registered too late and template-phase rules —
        // `vue/no-dupe-v-else-if`, `vue/no-mutating-props`, etc. — can't be
        // suppressed. (#968, #1196)
        self.prescan_suppression_directives(root);

        // Run template-level checks under one profiling span. Rule dispatch
        // happens for every file, so profiling once around the callback batch is
        // cheaper than creating a span for every individual rule callback.
        let keep_mask = self.keep_mask;
        profile!("patina.rules.run_on_template", {
            for (index, (rule, rule_name)) in self
                .rules
                .iter()
                .zip(self.rule_names.iter().copied())
                .enumerate()
            {
                self.ctx.current_rule = rule_name;
                if hook(index, self.ctx) || !Self::rule_active(keep_mask, index) {
                    continue;
                }
                rule.run_on_template(self.ctx, root);
            }
        });

        // Visit children
        for child in root.children.iter() {
            self.visit_child(child);
        }
    }

    /// Walk the AST and register every suppression-only directive into the
    /// context up-front. Directive kinds that emit diagnostics (Todo, Fixme,
    /// Deprecated) are intentionally NOT processed here; they rely on the
    /// existing ordering that runs during the main traversal.
    fn prescan_suppression_directives(&mut self, root: &RootNode<'a>) {
        let mut forget_next_child = false;
        self.prescan_suppression_in_children(&root.children, &mut forget_next_child);
    }

    fn prescan_suppression_in_children(
        &mut self,
        children: &[TemplateChildNode<'a>],
        forget_next_child: &mut bool,
    ) {
        for (index, node) in children.iter().enumerate() {
            self.prescan_suppression_in_child(children, index, node, forget_next_child);
        }
    }

    fn prescan_suppression_in_child(
        &mut self,
        siblings: &[TemplateChildNode<'a>],
        index: usize,
        node: &TemplateChildNode<'a>,
        forget_next_child: &mut bool,
    ) {
        match node {
            TemplateChildNode::Comment(comment) => {
                self.ctx
                    .register_lint_comment(comment.content, comment.loc.span.start);
                if let Some(kind) = comment.directive {
                    let line = self.ctx.offset_to_line(comment.loc.span.start);
                    match kind {
                        DirectiveKind::Expected => {
                            self.ctx.expect_error_next_line(line);
                        }
                        DirectiveKind::Level => {
                            if let Some(d) =
                                parse_vize_directive(comment.content, line, comment.loc.span.start)
                                && let Some(severity) = parse_level_severity(&d.payload)
                            {
                                self.ctx.set_severity_override_next_line(line, severity);
                            }
                        }
                        DirectiveKind::IgnoreStart => {
                            self.ctx.push_ignore_region(line);
                        }
                        DirectiveKind::IgnoreEnd => {
                            self.ctx.pop_ignore_region(line);
                        }
                        DirectiveKind::Forget => {
                            *forget_next_child = true;
                        }
                        _ => {}
                    }
                }
            }
            TemplateChildNode::Element(el) => {
                if *forget_next_child {
                    *forget_next_child = false;
                    self.disable_forgotten_element(siblings, index, el);
                }
                self.prescan_suppression_in_children(&el.children, forget_next_child);
            }
            TemplateChildNode::If(if_node) => {
                if *forget_next_child {
                    *forget_next_child = false;
                    self.disable_loc_range(&if_node.loc);
                }
                for branch in if_node.branches.iter() {
                    self.prescan_suppression_in_children(&branch.children, forget_next_child);
                }
            }
            TemplateChildNode::For(for_node) => {
                if *forget_next_child {
                    *forget_next_child = false;
                    self.disable_loc_range(&for_node.loc);
                }
                self.prescan_suppression_in_children(&for_node.children, forget_next_child);
            }
            _ => {}
        }
    }

    fn disable_forgotten_element(
        &mut self,
        siblings: &[TemplateChildNode<'a>],
        index: usize,
        el: &ElementNode<'a>,
    ) {
        self.disable_loc_range(&el.loc);

        if !element_has_directive(el, "if") {
            return;
        }

        for sibling in siblings.iter().skip(index + 1) {
            let TemplateChildNode::Element(branch) = sibling else {
                continue;
            };
            if element_has_directive(branch, "else-if") {
                self.disable_loc_range(&branch.loc);
                continue;
            }
            if element_has_directive(branch, "else") {
                self.disable_loc_range(&branch.loc);
            }
            break;
        }
    }

    fn disable_loc_range(&mut self, loc: &SourceLocation) {
        let start_line = self.ctx.offset_to_line(loc.span.start);
        let end_line = self.ctx.offset_to_line(loc.span.end);
        self.ctx.disable_all(start_line, Some(end_line));
    }
}

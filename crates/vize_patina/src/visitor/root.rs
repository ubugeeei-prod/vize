//! Root callbacks and suppression registration in registry order.

use super::{LintVisitor, element_has_directive};
use crate::context::CallbackSite;
use vize_l0::directive::{DirectiveKind, parse_level_severity, parse_vize_directive};
use vize_l0::{CompactString, profile};
use vize_relief::{ElementNode, RootNode, TemplateChildNode};

struct PendingForget {
    rules: Vec<CompactString>,
    marker: (u32, u32),
}

impl<'a, 'ctx, 'rules> LintVisitor<'a, 'ctx, 'rules> {
    /// Visit the root node and traverse the AST
    #[inline]
    pub fn visit_root(&mut self, root: &RootNode<'a>) {
        // Pre-scan suppression directives so they can suppress diagnostics
        // produced by `run_on_template` rules (which fire before per-element
        // traversal would register them). Without this pass, directives are
        // registered too late and template-phase rules —
        // `vue/no-dupe-v-else-if`, `vue/no-mutating-props`, etc. — can't be
        // suppressed. (#968, #1196)
        self.ctx.record_callback_site(CallbackSite::Prescan);
        if let Some(directives) = self.template_directives {
            directives.prescan(self.ctx);
        } else {
            self.prescan_suppression_directives(root);
        }

        // Run template-level checks under one profiling span. Rule dispatch
        // happens for every file, so profiling once around the callback batch is
        // cheaper than creating a span for every individual rule callback.
        let keep_mask = self.keep_mask;
        profile!("patina.rules.run_on_template", {
            Self::callback_rules(
                self.ctx,
                self.rules,
                self.rule_names,
                keep_mask,
                CallbackSite::Template,
                |_, rule, ctx| rule.run_on_template(ctx, root),
            );
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
    pub(crate) fn prescan_suppression_directives(&mut self, root: &RootNode<'a>) {
        self.prescan_suppression_in_children(&root.children);
    }

    fn prescan_suppression_in_children(&mut self, children: &[TemplateChildNode<'a>]) {
        vize_l0::recursion::ensure_sufficient_stack(|| {
            let mut forget_next_child = Vec::new();
            for (index, node) in children.iter().enumerate() {
                self.prescan_suppression_in_child(children, index, node, &mut forget_next_child);
            }
            for pending in forget_next_child {
                self.ctx
                    .disable_exact_targets(&pending.rules, Vec::new(), pending.marker);
            }
        });
    }

    fn prescan_suppression_in_child(
        &mut self,
        siblings: &[TemplateChildNode<'a>],
        index: usize,
        node: &TemplateChildNode<'a>,
        forget_next_child: &mut Vec<PendingForget>,
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
                            if let Some(directive) =
                                parse_vize_directive(comment.content, line, comment.loc.span.start)
                            {
                                forget_next_child.push(PendingForget {
                                    rules: forget_rules(&directive.payload),
                                    marker: (comment.loc.span.start, comment.loc.span.end),
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            TemplateChildNode::Element(el) => {
                if !forget_next_child.is_empty() {
                    let targets = self.forgotten_element_targets(siblings, index, el);
                    self.apply_forgotten_targets(forget_next_child, targets);
                }
                self.prescan_suppression_in_children(&el.children);
            }
            TemplateChildNode::If(if_node) => {
                if !forget_next_child.is_empty() {
                    self.apply_forgotten_targets(
                        forget_next_child,
                        vec![(if_node.loc.span.start, if_node.loc.span.end)],
                    );
                }
                for branch in if_node.branches.iter() {
                    self.prescan_suppression_in_children(&branch.children);
                }
            }
            TemplateChildNode::For(for_node) => {
                if !forget_next_child.is_empty() {
                    self.apply_forgotten_targets(
                        forget_next_child,
                        vec![(for_node.loc.span.start, for_node.loc.span.end)],
                    );
                }
                self.prescan_suppression_in_children(&for_node.children);
            }
            TemplateChildNode::Text(text)
                if !text.content.trim().is_empty() && !forget_next_child.is_empty() =>
            {
                self.apply_forgotten_targets(
                    forget_next_child,
                    vec![(text.loc.span.start, text.loc.span.end)],
                );
            }
            TemplateChildNode::Interpolation(interpolation) if !forget_next_child.is_empty() => {
                self.apply_forgotten_targets(
                    forget_next_child,
                    vec![(interpolation.loc.span.start, interpolation.loc.span.end)],
                );
            }
            _ => {}
        }
    }

    fn forgotten_element_targets(
        &self,
        siblings: &[TemplateChildNode<'a>],
        index: usize,
        el: &ElementNode<'a>,
    ) -> Vec<(u32, u32)> {
        let mut targets = vec![(el.loc.span.start, el.loc.span.end)];

        if !element_has_directive(el, "if") {
            return targets;
        }

        for sibling in siblings.iter().skip(index + 1) {
            let TemplateChildNode::Element(branch) = sibling else {
                continue;
            };
            if element_has_directive(branch, "else-if") {
                targets.push((branch.loc.span.start, branch.loc.span.end));
                continue;
            }
            if element_has_directive(branch, "else") {
                targets.push((branch.loc.span.start, branch.loc.span.end));
            }
            break;
        }
        targets
    }

    fn apply_forgotten_targets(
        &mut self,
        pending: &mut Vec<PendingForget>,
        targets: Vec<(u32, u32)>,
    ) {
        for directive in pending.drain(..) {
            self.ctx
                .disable_exact_targets(&directive.rules, targets.clone(), directive.marker);
        }
    }
}

fn forget_rules(payload: &str) -> Vec<CompactString> {
    payload
        .split(|character: char| character == ',' || character.is_ascii_whitespace())
        .map(|candidate| candidate.trim_matches(['"', '\'', '(', ')', '[', ']', '{', '}', ';']))
        .filter(|candidate| looks_like_rule_id(candidate))
        .map(CompactString::from)
        .collect()
}

fn looks_like_rule_id(candidate: &str) -> bool {
    let Some((namespace, name)) = candidate.split_once('/') else {
        return false;
    };
    !namespace.is_empty()
        && !name.is_empty()
        && candidate.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_' | b'@' | b'.' | b'$')
        })
}

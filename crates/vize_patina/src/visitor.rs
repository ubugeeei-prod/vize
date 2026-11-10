//! AST visitor for lint rule execution.
//!
//! High-performance visitor with minimal allocations.

use crate::context::{
    CallbackSite, ElementContext, LintContext, TemplateCommentDirectives, process_vize_directive,
};
use crate::rule::Rule;
use vize_l0::{CompactString, profile};
use vize_relief::{ElementNode, PropNode, TemplateChildNode};

pub use crate::visitor_scope::{
    has_free_reference, parse_slot_scope_variables, parse_v_for_variables,
};

mod branches;
mod dispatch;
mod root;

/// Visit the AST and run all rules
pub struct LintVisitor<'a, 'ctx, 'rules> {
    ctx: &'ctx mut LintContext<'a>,
    rules: &'rules [Box<dyn Rule>],
    rule_names: &'rules [&'static str],
    run_exit_element_rules: bool,
    /// Optional per-rule keep mask, parallel to `rules`. When `Some`, a rule is
    /// dispatched only where its entry is `true`. Used by the JSX/TSX fallback
    /// lowering pass to skip rules already handled by the zero-cost markup IR
    /// pass, so a migrated rule never reports twice. `None` (the common
    /// template path) runs every rule with no extra work.
    keep_mask: Option<&'rules [bool]>,
    track_aria_hidden_inert: bool,
    template_directives: Option<&'ctx TemplateCommentDirectives<'ctx, 'a>>,
}

impl<'a, 'ctx, 'rules> LintVisitor<'a, 'ctx, 'rules> {
    /// Create a new visitor
    #[inline]
    pub fn new(
        ctx: &'ctx mut LintContext<'a>,
        rules: &'rules [Box<dyn Rule>],
        rule_names: &'rules [&'static str],
        run_exit_element_rules: bool,
    ) -> Self {
        Self {
            ctx,
            rules,
            rule_names,
            run_exit_element_rules,
            keep_mask: None,
            track_aria_hidden_inert: rule_names.contains(&"a11y/no-aria-hidden-on-focusable"),
            template_directives: None,
        }
    }

    /// Create a visitor that dispatches only the rules whose `keep_mask` entry
    /// is `true`. The mask must be the same length as `rules`.
    ///
    /// The JSX/TSX lint path uses this for its fallback lowering pass: rules
    /// that already ran over the zero-cost markup IR are masked out here so a
    /// migrated rule produces exactly one diagnostic, not one per backend.
    #[inline]
    pub fn with_rule_filter(
        ctx: &'ctx mut LintContext<'a>,
        rules: &'rules [Box<dyn Rule>],
        rule_names: &'rules [&'static str],
        run_exit_element_rules: bool,
        keep_mask: &'rules [bool],
    ) -> Self {
        Self {
            ctx,
            rules,
            rule_names,
            run_exit_element_rules,
            keep_mask: Some(keep_mask),
            track_aria_hidden_inert: rule_names.iter().enumerate().any(|(index, name)| {
                *name == "a11y/no-aria-hidden-on-focusable"
                    && Self::rule_active(Some(keep_mask), index)
            }),
            template_directives: None,
        }
    }

    pub(crate) fn with_template_directives(
        mut self,
        directives: &'ctx TemplateCommentDirectives<'ctx, 'a>,
    ) -> Self {
        self.template_directives = Some(directives);
        self
    }

    /// Whether the rule at `index` is active under the current [`Self::keep_mask`].
    ///
    /// Hot path: with no mask (the common template path) this is a constant
    /// `true`; the JSX fallback pass uses it to skip rules the markup IR pass
    /// already handled. Reads only `keep_mask`, so it does not conflict with the
    /// `&mut self.ctx` the dispatch loops hold.
    #[inline]
    fn rule_active(keep_mask: Option<&[bool]>, index: usize) -> bool {
        match keep_mask {
            Some(mask) => mask.get(index).copied().unwrap_or(false),
            None => true,
        }
    }

    #[inline]
    fn visit_child(&mut self, node: &TemplateChildNode<'a>) {
        vize_l0::recursion::ensure_sufficient_stack(|| self.visit_child_inner(node));
    }

    fn visit_child_inner(&mut self, node: &TemplateChildNode<'a>) {
        match node {
            TemplateChildNode::Element(el) => {
                self.visit_element(el);
            }
            TemplateChildNode::Interpolation(interp) => {
                // Coalesce all interpolation rule callbacks into one span for
                // the same reason as template-level checks: callback dispatch is
                // hot and individual rule spans add measurable overhead.
                let keep_mask = self.keep_mask;
                profile!("patina.rules.check_interpolation", {
                    Self::callback_rules(
                        self.ctx,
                        self.rules,
                        self.rule_names,
                        keep_mask,
                        CallbackSite::Interpolation(interp.loc.span.start),
                        |_, rule, ctx| rule.check_interpolation(ctx, interp),
                    );
                });
            }
            TemplateChildNode::If(if_node) => {
                self.visit_if(if_node);
            }
            TemplateChildNode::For(for_node) => {
                self.visit_for(for_node);
            }
            TemplateChildNode::Comment(comment) => {
                if let Some(directives) = self.template_directives {
                    if directives.contains(comment.loc.span.start) {
                        self.ctx
                            .record_callback_site(CallbackSite::Comment(comment.loc.span.start));
                        directives.process_at(self.ctx, comment.loc.span.start);
                    }
                } else if let Some(kind) = comment.directive {
                    self.ctx
                        .record_callback_site(CallbackSite::Comment(comment.loc.span.start));
                    process_vize_directive(self.ctx, comment.content, comment.loc.span, kind);
                }
            }
            TemplateChildNode::Text(text) => {
                self.ctx
                    .record_callback_site(CallbackSite::Text(text.loc.span.start));
            }
            _ => {}
        }
    }

    fn visit_element(&mut self, el: &ElementNode<'a>) {
        // Check for v-for, v-if, and v-slot directives using iterators (no allocation)
        let has_v_for = el
            .props
            .iter()
            .any(|p| matches!(p, PropNode::Directive(d) if d.name == "for"));
        let has_v_if = el
            .props
            .iter()
            .any(|p| matches!(p, PropNode::Directive(d) if d.name == "if" || d.name == "else-if"));
        let has_v_slot = el
            .props
            .iter()
            .any(|p| matches!(p, PropNode::Directive(d) if d.name == "slot"));

        // Extract scope variables (only allocates if directives exist).
        let mut v_for_vars = if has_v_for {
            self.extract_v_for_vars(el)
        } else {
            Vec::new()
        };
        if has_v_slot {
            v_for_vars.extend(self.extract_slot_scope_vars(el));
        }

        // Build element context with CompactString tag (efficient for small strings)
        let elem_ctx = ElementContext {
            tag: CompactString::from(el.tag),
            has_v_for,
            has_v_if,
            v_for_vars,
        };

        self.ctx.push_element(elem_ctx);
        let previous_inert = self.ctx.aria_hidden_inert;
        if self.track_aria_hidden_inert {
            let element = crate::markup::MarkupElement::new(el);
            let static_inert = crate::rules::a11y::markup_helpers::has_static_inert(&element);
            self.ctx.aria_hidden_inert = static_inert
                || (previous_inert
                    && !crate::rules::a11y::markup_helpers::blocks_inert_inheritance(&element));
        }

        // Enter element - run rules. Element/directive/exit/branch callbacks
        // follow the same coalesced-span pattern as root/interpolation checks:
        // one guard around the rule batch, not one guard per rule.
        let keep_mask = self.keep_mask;
        profile!("patina.rules.enter_element", {
            Self::callback_rules(
                self.ctx,
                self.rules,
                self.rule_names,
                keep_mask,
                CallbackSite::Element(el.loc.span.start),
                |_, rule, ctx| rule.enter_element(ctx, el),
            );
        });

        self.ctx
            .record_callback_site(CallbackSite::Attributes(el.loc.span.start));
        // Check directives
        for prop in el.props.iter() {
            if let PropNode::Directive(dir) = prop {
                profile!("patina.rules.check_directive", {
                    Self::callback_rules(
                        self.ctx,
                        self.rules,
                        self.rule_names,
                        keep_mask,
                        CallbackSite::Directive(el.loc.span.start, dir.loc.span.start),
                        |_, rule, ctx| rule.check_directive(ctx, el, dir),
                    );
                });
            }
        }

        // Visit children
        for child in el.children.iter() {
            self.visit_child(child);
        }

        if self.run_exit_element_rules {
            // Exit element - run rules
            profile!("patina.rules.exit_element", {
                Self::callback_rules(
                    self.ctx,
                    self.rules,
                    self.rule_names,
                    keep_mask,
                    CallbackSite::ExitElement(el.loc.span.start),
                    |_, rule, ctx| rule.exit_element(ctx, el),
                );
            });
        }

        self.ctx.pop_element();
        self.ctx.aria_hidden_inert = previous_inert;
    }
}

fn element_has_directive(el: &ElementNode, name: &str) -> bool {
    el.props
        .iter()
        .any(|p| matches!(p, PropNode::Directive(d) if d.name == name))
}

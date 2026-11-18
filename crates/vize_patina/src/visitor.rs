//! AST visitor for lint rule execution.
//!
//! High-performance visitor with minimal allocations.

use crate::context::{
    CallbackSite, ElementContext, LintContext, TemplateCommentDirectives, process_vize_directive,
};
use crate::rule::{Rule, RuleHooks};
use vize_l0::{CompactString, SmallVec, profile};
use vize_relief::{ElementNode, PropNode, TemplateChildNode};

pub use crate::visitor_scope::{
    has_free_reference, has_free_reference_retained, parse_slot_scope_variables,
    parse_v_for_variables,
};

mod branches;
mod dispatch;
mod root;

#[derive(Clone, Copy)]
pub(super) enum RuleSelection<'rules> {
    All,
    Mask(&'rules [bool]),
    Indices(&'rules [usize]),
}

#[derive(Default)]
pub(super) struct IndexedHookMask {
    words: SmallVec<[u64; 2]>,
}

impl IndexedHookMask {
    fn insert(&mut self, rank: usize) {
        let word = rank / u64::BITS as usize;
        while self.words.len() <= word {
            self.words.push(0);
        }
        self.words[word] |= 1 << (rank % u64::BITS as usize);
    }
}

#[derive(Default)]
struct IndexedRulePlans {
    template: IndexedHookMask,
    element: IndexedHookMask,
    exit: IndexedHookMask,
    directive: IndexedHookMask,
    for_node: IndexedHookMask,
    if_node: IndexedHookMask,
    interpolation: IndexedHookMask,
}

impl IndexedRulePlans {
    fn new(rules: &[Box<dyn Rule>], indices: &[usize]) -> Self {
        let mut plans = Self::default();
        for &rank in indices {
            let Some(rule) = rules.get(rank) else {
                continue;
            };
            let hooks = rule.template_hooks();
            for (hook, mask) in [
                (RuleHooks::TEMPLATE, &mut plans.template),
                (RuleHooks::ELEMENT, &mut plans.element),
                (RuleHooks::EXIT, &mut plans.exit),
                (RuleHooks::DIRECTIVE, &mut plans.directive),
                (RuleHooks::FOR, &mut plans.for_node),
                (RuleHooks::IF, &mut plans.if_node),
                (RuleHooks::INTERPOLATION, &mut plans.interpolation),
            ] {
                if hooks.contains(hook) {
                    mask.insert(rank);
                }
            }
        }
        plans
    }
}

/// Visit the AST and run all rules
pub struct LintVisitor<'a, 'ctx, 'rules> {
    ctx: &'ctx mut LintContext<'a>,
    rules: &'rules [Box<dyn Rule>],
    rule_names: &'rules [&'static str],
    run_exit_element_rules: bool,
    /// Rule ranks to dispatch. The SFC split stores its smaller legacy side as
    /// compact ranks so every callback skips facade rules without scanning the
    /// full registry. JSX retains its existing boolean partition.
    selection: RuleSelection<'rules>,
    /// Callback masks keyed by registry rank for the legacy subset.
    indexed_plans: IndexedRulePlans,
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
            selection: RuleSelection::All,
            indexed_plans: IndexedRulePlans::default(),
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
            selection: RuleSelection::Mask(keep_mask),
            indexed_plans: IndexedRulePlans::default(),
            track_aria_hidden_inert: rule_names.iter().enumerate().any(|(index, name)| {
                *name == "a11y/no-aria-hidden-on-focusable"
                    && Self::rule_active(RuleSelection::Mask(keep_mask), index)
            }),
            template_directives: None,
        }
    }

    /// Create a visitor over an ordered subset of registry ranks.
    #[inline]
    pub(crate) fn with_rule_indices(
        ctx: &'ctx mut LintContext<'a>,
        rules: &'rules [Box<dyn Rule>],
        rule_names: &'rules [&'static str],
        run_exit_element_rules: bool,
        indices: &'rules [usize],
    ) -> Self {
        let indexed_plans = IndexedRulePlans::new(rules, indices);
        Self {
            ctx,
            rules,
            rule_names,
            run_exit_element_rules,
            selection: RuleSelection::Indices(indices),
            indexed_plans,
            track_aria_hidden_inert: indices
                .iter()
                .any(|&index| rule_names.get(index) == Some(&"a11y/no-aria-hidden-on-focusable")),
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

    /// Whether the rule at `index` is active under `selection`.
    #[inline]
    fn rule_active(selection: RuleSelection<'_>, index: usize) -> bool {
        match selection {
            RuleSelection::All => true,
            RuleSelection::Mask(mask) => mask.get(index).copied().unwrap_or(false),
            RuleSelection::Indices(indices) => indices.binary_search(&index).is_ok(),
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
                let selection = self.selection;
                profile!("patina.rules.check_interpolation", {
                    Self::callback_rules(
                        self.ctx,
                        self.rules,
                        self.rule_names,
                        selection,
                        &self.indexed_plans.interpolation,
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
        // Derive the element's cheap facts in one property pass. They used to
        // rescan the opening tag independently before every callback batch.
        let (mut has_v_for, mut has_v_if, mut has_v_slot, mut vue_is_component) =
            (false, false, false, false);
        for prop in el.props.iter() {
            match prop {
                PropNode::Directive(directive) => match directive.name {
                    "for" => has_v_for = true,
                    "if" | "else-if" => has_v_if = true,
                    "slot" => has_v_slot = true,
                    _ => {}
                },
                PropNode::Attribute(attribute) => {
                    vue_is_component |= attribute.name.eq_ignore_ascii_case("is")
                        && attribute
                            .value
                            .as_ref()
                            .is_some_and(|value| value.content.starts_with("vue:"));
                }
            }
        }

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
            vue_is_component,
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
        let selection = self.selection;
        profile!("patina.rules.enter_element", {
            Self::callback_rules(
                self.ctx,
                self.rules,
                self.rule_names,
                selection,
                &self.indexed_plans.element,
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
                        selection,
                        &self.indexed_plans.directive,
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
                    selection,
                    &self.indexed_plans.exit,
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

//! [`MarkupRules`]: several markup rules driven by one document walk.

use super::{
    MarkupBinding, MarkupConditional, MarkupContext, MarkupDirective, MarkupDocument,
    MarkupElement, MarkupList, MarkupRule, MarkupText, MarkupTraversal,
};
use crate::context::CallbackSite;
use crate::ir::ByteRange;
use vize_l0::SmallVec;

/// A markup rule at its registry rank.
#[derive(Clone, Copy)]
pub struct RankedMarkupRule<'r> {
    rank: usize,
    name: &'static str,
    traversal: MarkupTraversal,
    rule: &'r dyn MarkupRule,
    requires_original_element_syntax: bool,
}

impl<'r> RankedMarkupRule<'r> {
    /// `rank` is the rule's registry index; it orders rules at one callback site.
    pub fn new(rank: usize, rule: &'r dyn MarkupRule) -> Self {
        Self {
            rank,
            name: rule.name(),
            traversal: rule.traversal(),
            rule,
            requires_original_element_syntax: rule.requires_original_element_syntax(),
        }
    }

    /// The rule's registry rank.
    #[inline]
    pub fn rank(&self) -> usize {
        self.rank
    }

    /// The rule's name.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub(crate) fn requires_original_element_syntax(&self) -> bool {
        self.requires_original_element_syntax
    }
}

/// Markup rules driven by one walk, every hook forwarded in rank order.
///
/// Every input (SFC template, JSX/TSX, Pug) dispatches its markup rules
/// through this one walk, so adding a rule adds hook calls, never a walk.
/// Each rule receives exactly the callbacks its own
/// [`MarkupRule::traversal`] keeps.
pub struct MarkupRules<'r> {
    rules: &'r [RankedMarkupRule<'r>],
    elements_and_bindings: SmallVec<[&'r RankedMarkupRule<'r>; 64]>,
    attributes_and_lists: SmallVec<[&'r RankedMarkupRule<'r>; 8]>,
    text_and_exits: SmallVec<[&'r RankedMarkupRule<'r>; 8]>,
    interpolations: SmallVec<[&'r RankedMarkupRule<'r>; 8]>,
    directives: SmallVec<[&'r RankedMarkupRule<'r>; 8]>,
    conditionals: SmallVec<[&'r RankedMarkupRule<'r>; 8]>,
    has_bindings: bool,
    requires_original_element_syntax: bool,
}

impl<'r> MarkupRules<'r> {
    /// Rules in rank order; hooks forward in slice order.
    pub fn new(rules: &'r [RankedMarkupRule<'r>]) -> Self {
        let mut batch = Self {
            rules,
            elements_and_bindings: SmallVec::new(),
            attributes_and_lists: SmallVec::new(),
            text_and_exits: SmallVec::new(),
            interpolations: SmallVec::new(),
            directives: SmallVec::new(),
            conditionals: SmallVec::new(),
            has_bindings: false,
            requires_original_element_syntax: false,
        };
        for entry in rules {
            let traversal = entry.traversal;
            if traversal.enters_elements() || traversal.enters_bindings() {
                batch.elements_and_bindings.push(entry);
            }
            if traversal.keeps_attributes_and_lists() {
                batch.attributes_and_lists.push(entry);
            }
            if traversal.keeps_text_and_exits() {
                batch.text_and_exits.push(entry);
            }
            if traversal.enters_interpolations() {
                batch.interpolations.push(entry);
            }
            if traversal.enters_directives() {
                batch.directives.push(entry);
            }
            if traversal.enters_conditionals() {
                batch.conditionals.push(entry);
            }
            batch.has_bindings |= traversal.enters_bindings();
            batch.requires_original_element_syntax |= entry.requires_original_element_syntax;
        }
        batch
    }

    /// Whether no rule is selected.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Point the context at `entry` for every rule `keeps` admits.
    #[inline]
    fn each<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        entries: &[&'r RankedMarkupRule<'r>],
        mut hook: impl FnMut(&RankedMarkupRule<'r>, &mut MarkupContext<'_, 'a>),
    ) {
        for &entry in entries {
            ctx.lint().set_callback_rule(entry.rank);
            ctx.lint().current_rule = entry.name;
            hook(entry, ctx);
        }
    }
}

impl MarkupRule for MarkupRules<'_> {
    fn name(&self) -> &'static str {
        ""
    }

    fn requires_original_element_syntax(&self) -> bool {
        self.requires_original_element_syntax
    }

    fn needs_original_element_syntax(&self, ctx: &crate::context::LintContext<'_>) -> bool {
        self.requires_original_element_syntax
            && self
                .rules
                .iter()
                .any(|rule| rule.requires_original_element_syntax && ctx.is_rule_enabled(rule.name))
    }

    fn traversal(&self) -> MarkupTraversal {
        if self.elements_and_bindings.is_empty()
            && self.attributes_and_lists.is_empty()
            && self.text_and_exits.is_empty()
            && self.interpolations.is_empty()
            && self.directives.is_empty()
            && self.conditionals.is_empty()
        {
            MarkupTraversal::Document
        } else if self.conditionals.is_empty() {
            MarkupTraversal::Batch
        } else {
            MarkupTraversal::BatchWithConditionals
        }
    }

    fn shares_element_reads(&self) -> bool {
        true
    }

    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        ctx.lint().replay_callback_site(CallbackSite::Template);
        for entry in self.rules {
            ctx.lint().set_callback_rule(entry.rank);
            ctx.lint().current_rule = entry.name;
            entry.rule.enter_document(ctx, document);
        }
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        ctx.lint()
            .replay_callback_site(CallbackSite::Element(element.range().start));
        // One binding walk per element, not per rule. Each rule still reports
        // its attributes beside its element, before the next rule runs.
        let mut bindings: SmallVec<[MarkupBinding<'a>; 8]> = SmallVec::new();
        if self.has_bindings {
            element.walk_bindings(&mut |binding| bindings.push(binding));
        }
        for &entry in &self.elements_and_bindings {
            match entry.traversal {
                MarkupTraversal::Full => {
                    ctx.lint().set_callback_rule(entry.rank);
                    ctx.lint().current_rule = entry.name;
                    entry.rule.enter_element(ctx, element);
                    for binding in &bindings {
                        ctx.lint().set_callback_rule(entry.rank);
                        ctx.lint().current_rule = entry.name;
                        entry.rule.enter_binding(ctx, element, binding);
                    }
                }
                MarkupTraversal::Elements
                | MarkupTraversal::ElementsAndText
                | MarkupTraversal::ElementsAndInterpolations
                | MarkupTraversal::ElementsAndDirectives => {
                    ctx.lint().set_callback_rule(entry.rank);
                    ctx.lint().current_rule = entry.name;
                    entry.rule.enter_element(ctx, element);
                }
                MarkupTraversal::Bindings => {
                    for binding in &bindings {
                        ctx.lint().set_callback_rule(entry.rank);
                        ctx.lint().current_rule = entry.name;
                        entry.rule.enter_binding(ctx, element, binding);
                    }
                }
                MarkupTraversal::Directives
                | MarkupTraversal::AttributesAndLists
                | MarkupTraversal::Batch
                | MarkupTraversal::BatchWithConditionals
                | MarkupTraversal::Document => debug_assert!(
                    false,
                    "element dispatch plan contains a traversal without element or binding hooks"
                ),
            }
        }
    }

    fn enter_attributes<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        ctx.lint()
            .replay_callback_site(CallbackSite::Attributes(element.range().start));
        self.each(ctx, &self.attributes_and_lists, |entry, ctx| {
            entry.rule.enter_attributes(ctx, element);
        });
    }

    fn exit_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        ctx.lint()
            .replay_callback_site(CallbackSite::ExitElement(element.range().start));
        self.each(ctx, &self.text_and_exits, |entry, ctx| {
            entry.rule.exit_element(ctx, element);
        });
    }

    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
        ctx.lint().replay_callback_site(CallbackSite::Directive(
            element.range().start,
            directive.range().start,
        ));
        self.each(ctx, &self.directives, |entry, ctx| {
            entry.rule.enter_directive(ctx, element, directive);
        });
    }

    fn enter_conditional<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        conditional: &MarkupConditional<'a>,
    ) {
        ctx.lint()
            .replay_callback_site(CallbackSite::Conditional(conditional.range().start));
        self.each(ctx, &self.conditionals, |entry, ctx| {
            entry.rule.enter_conditional(ctx, conditional);
        });
    }

    fn enter_list<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, list: &MarkupList<'a>) {
        ctx.lint()
            .replay_callback_site(CallbackSite::List(list.range().start));
        self.each(ctx, &self.attributes_and_lists, |entry, ctx| {
            entry.rule.enter_list(ctx, list);
        });
    }

    fn enter_text<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, text: &MarkupText<'a>) {
        ctx.lint()
            .replay_callback_site(CallbackSite::Text(text.range().start));
        self.each(ctx, &self.text_and_exits, |entry, ctx| {
            entry.rule.enter_text(ctx, text);
        });
    }

    fn enter_interpolation(&self, ctx: &mut MarkupContext<'_, '_>, range: ByteRange) {
        ctx.lint()
            .replay_callback_site(CallbackSite::Interpolation(range.start));
        self.each(ctx, &self.interpolations, |entry, ctx| {
            entry.rule.enter_interpolation(ctx, range);
        });
    }
}

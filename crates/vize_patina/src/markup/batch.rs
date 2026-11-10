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
}

impl<'r> RankedMarkupRule<'r> {
    /// `rank` is the rule's registry index; it orders rules at one callback site.
    pub fn new(rank: usize, rule: &'r dyn MarkupRule) -> Self {
        Self {
            rank,
            name: rule.name(),
            traversal: rule.traversal(),
            rule,
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
}

/// Markup rules driven by one walk, every hook forwarded in rank order.
///
/// Every input (SFC template, JSX/TSX, Pug) dispatches its markup rules
/// through this one walk, so adding a rule adds hook calls, never a walk.
/// Each rule receives exactly the callbacks its own
/// [`MarkupRule::traversal`] keeps.
#[derive(Clone, Copy)]
pub struct MarkupRules<'r> {
    rules: &'r [RankedMarkupRule<'r>],
    bindings: bool,
}

impl<'r> MarkupRules<'r> {
    /// Rules in rank order; hooks forward in slice order.
    pub fn new(rules: &'r [RankedMarkupRule<'r>]) -> Self {
        Self {
            rules,
            bindings: rules.iter().any(|rule| rule.traversal.enters_bindings()),
        }
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
        keeps: fn(MarkupTraversal) -> bool,
        mut hook: impl FnMut(&RankedMarkupRule<'r>, &mut MarkupContext<'_, 'a>),
    ) {
        for entry in self.rules {
            if keeps(entry.traversal) {
                ctx.lint().set_callback_rule(entry.rank);
                ctx.lint().current_rule = entry.name;
                hook(entry, ctx);
            }
        }
    }
}

impl MarkupRule for MarkupRules<'_> {
    fn name(&self) -> &'static str {
        ""
    }

    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        ctx.lint().replay_callback_site(CallbackSite::Template);
        self.each(
            ctx,
            |_| true,
            |entry, ctx| entry.rule.enter_document(ctx, document),
        );
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        ctx.lint()
            .replay_callback_site(CallbackSite::Element(element.range().start));
        // One binding walk per element, not per rule. Each rule still reports
        // its attributes beside its element, before the next rule runs.
        let mut bindings: SmallVec<[MarkupBinding<'a>; 8]> = SmallVec::new();
        if self.bindings {
            element.walk_bindings(&mut |binding| bindings.push(binding));
        }
        for entry in self.rules {
            if entry.traversal.enters_elements() {
                ctx.lint().set_callback_rule(entry.rank);
                ctx.lint().current_rule = entry.name;
                entry.rule.enter_element(ctx, element);
            }
            if entry.traversal.enters_bindings() {
                for binding in &bindings {
                    ctx.lint().set_callback_rule(entry.rank);
                    ctx.lint().current_rule = entry.name;
                    entry.rule.enter_binding(ctx, element, binding);
                }
            }
        }
    }

    fn enter_attributes<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        ctx.lint()
            .replay_callback_site(CallbackSite::Attributes(element.range().start));
        self.each(
            ctx,
            MarkupTraversal::keeps_attributes_and_lists,
            |entry, ctx| {
                entry.rule.enter_attributes(ctx, element);
            },
        );
    }

    fn exit_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        ctx.lint()
            .replay_callback_site(CallbackSite::ExitElement(element.range().start));
        self.each(ctx, MarkupTraversal::keeps_text_and_exits, |entry, ctx| {
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
        self.each(ctx, MarkupTraversal::is_full, |entry, ctx| {
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
        self.each(ctx, MarkupTraversal::is_full, |entry, ctx| {
            entry.rule.enter_conditional(ctx, conditional);
        });
    }

    fn enter_list<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, list: &MarkupList<'a>) {
        ctx.lint()
            .replay_callback_site(CallbackSite::List(list.range().start));
        self.each(
            ctx,
            MarkupTraversal::keeps_attributes_and_lists,
            |entry, ctx| {
                entry.rule.enter_list(ctx, list);
            },
        );
    }

    fn enter_text<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, text: &MarkupText<'a>) {
        ctx.lint()
            .replay_callback_site(CallbackSite::Text(text.range().start));
        self.each(ctx, MarkupTraversal::keeps_text_and_exits, |entry, ctx| {
            entry.rule.enter_text(ctx, text);
        });
    }

    fn enter_interpolation(&self, ctx: &mut MarkupContext<'_, '_>, range: ByteRange) {
        ctx.lint()
            .replay_callback_site(CallbackSite::Interpolation(range.start));
        self.each(ctx, MarkupTraversal::enters_interpolations, |entry, ctx| {
            entry.rule.enter_interpolation(ctx, range);
        });
    }
}

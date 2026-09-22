//! [`MarkupRule`]: a lint rule expressed against the zero-copy markup IR.

use super::binding::MarkupBinding;
use super::directive::MarkupDirective;
use super::element::MarkupElement;
use super::hooks::MarkupHooks;
use super::node::MarkupText;
use super::scope::{MarkupConditional, MarkupList};
use super::{MarkupContext, MarkupDocument};
use crate::ir::ByteRange;

/// A lint rule expressed against the zero-copy markup IR.
///
/// This is the parallel of [`crate::rule::Rule`] for the unified IR: the same
/// rule object can run over Vue templates *and* JSX/TSX. All hooks default to
/// empty, so a rule overrides only what it needs. Drive a rule with
/// [`MarkupDocument::visit_with`] / [`MarkupDocumentVisitor`].
///
/// Hooks fire in source order during a single depth-first traversal. Reporting
/// uses [`MarkupContext::lint`] + the `*_at` [`LintContext`] helpers so all
/// diagnostics and fixes map back to original syntax via [`ByteRange`].
pub trait MarkupRule {
    /// The rule name, used to set [`LintContext::current_rule`] before each
    /// callback so diagnostics are attributed and rule-level suppression works.
    fn name(&self) -> &'static str;

    /// The hooks this rule implements. A fused lint pass calls the rule only
    /// at these, and skips a binding / directive walk no rule subscribes to.
    /// The default subscribes to every hook — always correct; a rule narrows
    /// it to exactly the hooks it overrides (the switch oracle and the rule's
    /// own tests catch a hook left out).
    fn hooks(&self) -> MarkupHooks {
        MarkupHooks::ALL
    }

    /// Called once before traversal begins.
    #[allow(unused_variables)]
    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {}

    /// Called on entering each element / component / fragment / template / slot.
    #[allow(unused_variables)]
    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {}

    /// Called on exiting each element.
    #[allow(unused_variables)]
    fn exit_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {}

    /// Called for every binding on an element (attribute / bind / event / model
    /// / custom directive), in source order.
    #[allow(unused_variables)]
    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
    }

    /// Called for every directive-like binding on an element (Vue `v-*` or a
    /// directive-like JSX attribute). A strict subset of [`Self::enter_binding`]
    /// for rules that only care about directives.
    #[allow(unused_variables)]
    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
    }

    /// Called on entering a conditional scope (a `v-if` chain, a lowered JSX
    /// conditional, an S2 `ui.if`).
    #[allow(unused_variables)]
    fn enter_conditional<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        conditional: &MarkupConditional<'a>,
    ) {
    }

    /// Called on entering a list scope (a `v-for`, a lowered JSX `.map()`, an
    /// S2 `ui.for`).
    #[allow(unused_variables)]
    fn enter_list<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, list: &MarkupList<'a>) {}

    /// Called for each text node.
    #[allow(unused_variables)]
    fn enter_text<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, text: &MarkupText<'a>) {}

    /// Called for each interpolation / expression node (`{{ … }}` or a JSX
    /// `{expr}` child). The range addresses the original source.
    #[allow(unused_variables)]
    fn enter_interpolation(&self, ctx: &mut MarkupContext<'_, '_>, range: ByteRange) {}
}

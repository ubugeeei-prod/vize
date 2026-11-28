//! [`MarkupRule`]: a lint rule expressed against the zero-copy markup IR.

use super::binding::MarkupBinding;
use super::directive::MarkupDirective;
use super::element::MarkupElement;
use super::node::MarkupText;
use super::scope::{MarkupConditional, MarkupList};
use super::{MarkupContext, MarkupDocument};
use crate::ir::ByteRange;

/// The Davinci level a [`MarkupRule`] reads: the authored L1 surface, or the
/// L1→L2 lowering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkupLevel {
    /// The authored L1 surface tree.
    L1,
    /// The L1→L2 lowering.
    L2,
}

/// Callbacks a rule needs after [`MarkupRule::enter_document`].
///
/// The default keeps every hook. Narrower traversals may omit only hooks the
/// rule leaves empty; element-entry rules retain the same authored ancestry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MarkupTraversal {
    /// Keep every traversal callback.
    #[default]
    Full,
    /// Call element entries while omitting other empty traversal callbacks.
    Elements,
    /// Call element entries, text nodes and element exits, preserving ancestry.
    ElementsAndText,
    /// Call element entries and interpolation / expression nodes.
    ElementsAndInterpolations,
    /// Call element entries and directive-like bindings.
    ElementsAndDirectives,
    /// Visit bindings without empty element, directive or text callbacks.
    Bindings,
    /// Visit directive-like bindings without unrelated callbacks.
    Directives,
    /// Visit authored attributes and list scopes without unrelated callbacks.
    AttributesAndLists,
    /// Multi-rule batch: every hook except bindings and conditionals.
    ///
    /// The batch folds binding dispatch into its element callback to preserve
    /// registry order without walking the same bindings twice.
    Batch,
    /// A multi-rule batch that also needs conditional scopes.
    BatchWithConditionals,
    /// All work happens in `enter_document`; no tree walk is needed.
    Document,
}

// One definition of each traversal's callbacks, shared by the single-rule
// visitor and the multi-rule batch.
impl MarkupTraversal {
    pub(crate) const fn enters_elements(self) -> bool {
        matches!(
            self,
            Self::Full
                | Self::Elements
                | Self::ElementsAndText
                | Self::ElementsAndInterpolations
                | Self::ElementsAndDirectives
                | Self::Batch
                | Self::BatchWithConditionals
        )
    }

    /// `exit_element`, and `enter_text` for text nodes.
    pub(crate) const fn keeps_text_and_exits(self) -> bool {
        matches!(
            self,
            Self::Full | Self::ElementsAndText | Self::Batch | Self::BatchWithConditionals
        )
    }

    /// `enter_attributes` and `enter_list`.
    pub(crate) const fn keeps_attributes_and_lists(self) -> bool {
        matches!(
            self,
            Self::Full | Self::AttributesAndLists | Self::Batch | Self::BatchWithConditionals
        )
    }

    pub(crate) const fn enters_bindings(self) -> bool {
        matches!(self, Self::Full | Self::Bindings)
    }

    pub(crate) const fn enters_interpolations(self) -> bool {
        matches!(
            self,
            Self::Full
                | Self::ElementsAndInterpolations
                | Self::Batch
                | Self::BatchWithConditionals
        )
    }

    pub(crate) const fn enters_directives(self) -> bool {
        matches!(
            self,
            Self::Full
                | Self::Directives
                | Self::ElementsAndDirectives
                | Self::Batch
                | Self::BatchWithConditionals
        )
    }

    pub(crate) const fn enters_conditionals(self) -> bool {
        matches!(self, Self::Full | Self::BatchWithConditionals)
    }
}

/// A lint rule expressed against the zero-copy markup IR.
///
/// This is the parallel of [`crate::rule::Rule`] for the unified IR: the same
/// rule object can run over Vue templates *and* JSX/TSX. All hooks default to
/// empty, so a rule overrides only what it needs. Drive a rule with
/// [`MarkupDocument::visit_with`] / [`super::MarkupDocumentVisitor`].
///
/// Hooks fire in source order during a single depth-first traversal. Reporting
/// uses [`MarkupContext::lint`] + the `*_at` [`crate::context::LintContext`]
/// helpers so all
/// diagnostics and fixes map back to original syntax via [`ByteRange`].
pub trait MarkupRule {
    /// The rule name, used to set the lint context's current rule before each
    /// callback so diagnostics are attributed and rule-level suppression works.
    fn name(&self) -> &'static str;

    /// The level this rule reads.
    fn level(&self) -> MarkupLevel {
        MarkupLevel::L1
    }

    /// Whether an active instance needs original element ranges or syntax
    /// classifications that semantic JSX lowering may change. Source lint
    /// entry points retain the factory packet; raw L2 views refuse this input.
    fn requires_original_element_syntax(&self) -> bool {
        false
    }

    /// Apply explicit rule selection before refusing an unsupported input.
    fn needs_original_element_syntax(&self, ctx: &crate::context::LintContext<'_>) -> bool {
        self.requires_original_element_syntax() && ctx.is_rule_enabled(self.name())
    }

    /// The hooks this rule needs, read once per document before `enter_document`.
    /// Existing rules keep every hook.
    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Full
    }

    /// Whether many rules read each visited element, so matching an L2
    /// element's opening tag once (in the template arena) and replaying it
    /// beats each rule walking it. A single rule keeps the allocation-free
    /// walk.
    fn shares_element_reads(&self) -> bool {
        false
    }

    /// Called once before traversal begins.
    fn enter_document(&self, _ctx: &mut MarkupContext<'_, '_>, _document: &MarkupDocument) {}

    /// Called on entering each element / component / fragment / template / slot.
    fn enter_element<'a>(&self, _ctx: &mut MarkupContext<'_, 'a>, _element: &MarkupElement<'a>) {}

    /// Called after all element-entry hooks, before per-binding/directive hooks.
    /// Authored structural directives consumed into scopes are still available
    /// through the element's retained source attributes at this point.
    fn enter_attributes<'a>(&self, _ctx: &mut MarkupContext<'_, 'a>, _element: &MarkupElement<'a>) {
    }

    /// Called on exiting each element.
    fn exit_element<'a>(&self, _ctx: &mut MarkupContext<'_, 'a>, _element: &MarkupElement<'a>) {}

    /// Called for every binding on an element (attribute / bind / event / model
    /// / custom directive), in source order.
    fn enter_binding<'a>(
        &self,
        _ctx: &mut MarkupContext<'_, 'a>,
        _element: &MarkupElement<'a>,
        _binding: &MarkupBinding<'a>,
    ) {
    }

    /// Called for every directive-like binding on an element (Vue `v-*` or a
    /// directive-like JSX attribute). A strict subset of [`Self::enter_binding`]
    /// for rules that only care about directives.
    fn enter_directive<'a>(
        &self,
        _ctx: &mut MarkupContext<'_, 'a>,
        _element: &MarkupElement<'a>,
        _directive: &MarkupDirective<'a>,
    ) {
    }

    /// Called on entering a conditional scope (a `v-if` chain, a lowered JSX
    /// conditional, an L2 `ui.if`).
    fn enter_conditional<'a>(
        &self,
        _ctx: &mut MarkupContext<'_, 'a>,
        _conditional: &MarkupConditional<'a>,
    ) {
    }

    /// Called on entering a list scope (a `v-for`, a lowered JSX `.map()`, an
    /// L2 `ui.for`).
    fn enter_list<'a>(&self, _ctx: &mut MarkupContext<'_, 'a>, _list: &MarkupList<'a>) {}

    /// Called for each text node.
    fn enter_text<'a>(&self, _ctx: &mut MarkupContext<'_, 'a>, _text: &MarkupText<'a>) {}

    /// Called for each interpolation / expression node (`{{ … }}` or a JSX
    /// `{expr}` child). The range addresses the original source.
    fn enter_interpolation(&self, _ctx: &mut MarkupContext<'_, '_>, _range: ByteRange) {}
}

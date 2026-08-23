//! The `vue.*` dialect family: genuinely Vue-specific ops that ride along
//! the neutral core instead of shaping it.
//!
//! A dialect op lands with the transform that needs it, never
//! speculatively. The fairness litmus test (P2-16) is what keeps this
//! family honest - a lint rule written against `ui.*` must run unchanged
//! on SFC and JSX, so anything only Vue understands belongs here. Style
//! `v-bind()` (P2-10) has no JSX twin: CSS `v-bind(color)` is SFC-only.

use vize_carton::{Span, Vec};

use super::DynamicName;
use crate::expr::ExprRef;

/// `vue.directive` - a Vue custom directive (`v-pin:top.lazy="value"`),
/// carried through S2 for a consumer that understands it.
///
/// Built-in directives never appear here: they normalize into `ui.*` ops
/// at lowering (`v-if` into [`super::IfOp`], `v-model` into
/// [`super::ModelOp`], ...); only user-defined directives survive as
/// dialect ops, exactly as the shipped pipeline emits runtime directive
/// references for them today.
#[derive(Debug)]
pub struct VueDirectiveOp<'a> {
    /// Directive name without the `v-` prefix, a slice of the source.
    pub name: &'a str,
    /// The authored argument (`v-pin:top`, `v-pin:[dir]`), when present.
    pub argument: Option<DynamicName<'a>>,
    /// Modifier names in authored order, without their leading dots.
    pub modifiers: Vec<'a, &'a str>,
    /// The directive's value expression, when authored.
    pub value: Option<ExprRef<'a>>,
    /// The whole directive's source range.
    pub span: Span,
}

/// `vue.css-bind` - one CSS `v-bind()` in an SFC style block (P2-10).
///
/// Not `ui.bind`: that is the template one-way binding. The CSS form
/// points into a style block, so [`VueCssBindOp::span`] is
/// **block-relative** (`Span::to_block_relative` against the style
/// block's content start). The bound text rides as [`ExprRef`] under
/// P2-5b's contract — CSS `v-bind()` contents are exactly the kind of
/// text that may have no retained AST.
#[derive(Debug)]
pub struct VueCssBindOp<'a> {
    /// The `v-bind()` argument, as authored inside the parentheses.
    pub value: ExprRef<'a>,
    /// The `v-bind(...)` call's range, relative to the style block.
    pub span: Span,
}

/// See [`crate::op`] for the guard rationale.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<VueDirectiveOp<'_>>() == 88);
    assert!(core::mem::size_of::<VueCssBindOp<'_>>() == 24);
};

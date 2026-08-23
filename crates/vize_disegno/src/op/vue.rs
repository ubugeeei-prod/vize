//! The `vue.*` dialect family: genuinely Vue-specific ops that ride along
//! the neutral core instead of shaping it.
//!
//! A dialect op lands with the transform that needs it (P2-9), never
//! speculatively. The fairness litmus test (P2-16) is what keeps this
//! family honest - a lint rule written against `ui.*` must run unchanged
//! on SFC and JSX, so anything only Vue understands belongs here. The
//! Vue 2 template-sugar surfaces (P2-9 installment 7) are exactly that:
//! `.sync`, `slot-scope`/`scope`, and pipe filters have no JSX twin.

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

/// `vue.sync` - Vue 2's `:foo.sync="bar"` two-way bind sugar.
///
/// The bounded subset the shipped pre-transform expands: a static
/// argument, a value expression, and the `sync` modifier. Remaining
/// modifiers (`camel`, …) ride here so the desugar pass can put them
/// on the resulting `ui.bind`. Dynamic-argument `:\[foo].sync` stays
/// `ui.bind` — the shipped lane skips it the same way.
#[derive(Debug)]
pub struct VueSyncOp<'a> {
    /// The bound prop name; static by the bounded-subset rule.
    pub name: DynamicName<'a>,
    /// Modifiers other than `sync`, in authored order.
    pub modifiers: Vec<'a, &'a str>,
    /// The value written back into on `update:<name>`.
    pub value: ExprRef<'a>,
    /// The whole directive's source range.
    pub span: Span,
}

/// `vue.slot-scope` - Vue 2's `slot-scope` / `scope` scoped-slot sugar.
///
/// The companion static `slot="name"` attribute is consumed at lowering
/// into [`VueSlotScopeOp::name`]; its absence is the default slot. A
/// carrier that already authors `v-slot` keeps the attributes as
/// attributes — the shipped lane will not emit a conflicting directive.
#[derive(Debug)]
pub struct VueSlotScopeOp<'a> {
    /// The target slot name from the companion `slot` attribute.
    pub name: Option<&'a str>,
    /// The slot-props expression (`slot-scope="props"`).
    pub params: Option<ExprRef<'a>>,
    /// The `slot-scope` / `scope` attribute's source range.
    pub span: Span,
}

/// See [`crate::op`] for the guard rationale.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<VueDirectiveOp<'_>>() == 88);
    assert!(core::mem::size_of::<VueSyncOp<'_>>() == 72);
    assert!(core::mem::size_of::<VueSlotScopeOp<'_>>() == 40);
};

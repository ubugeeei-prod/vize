//! The `vue.*` dialect family: genuinely Vue-specific ops that ride along
//! the neutral core instead of shaping it.
//!
//! A dialect op lands with the transform that needs it, never
//! speculatively. The fairness litmus test (P2-16) is what keeps this
//! family honest - a lint rule written against `ui.*` must run unchanged
//! on SFC and JSX, so anything only Vue understands belongs here. Style
//! `v-bind()` (P2-10) has no JSX twin: CSS `v-bind(color)` is SFC-only.
//! The Vue 2 template-sugar surfaces (P2-9 installment 7) are the same
//! kind of exception: `.sync`, `slot-scope`/`scope`, and pipe filters
//! have no JSX twin. `v-once` / `v-memo` / `v-show` / `v-html` / `v-text`
//! (P2-11) are the same kind: one-shot / dependency-memoized /
//! display-toggle / content-prop rendering is Vue's, not a fair `ui.*` core op.

use vize_s0::{Span, Vec};

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
/// points into a style block, and its spans are still the S0
/// file-absolute authored offsets of the complete SFC. The bound text
/// rides as [`ExprRef`] under P2-5b's contract — CSS `v-bind()` contents
/// are exactly the kind of text that may have no retained AST.
#[derive(Debug)]
pub struct VueCssBindOp<'a> {
    /// The `v-bind()` argument, as authored inside the parentheses.
    pub value: ExprRef<'a>,
    /// The `v-bind(...)` call's authored range in the complete source.
    pub span: Span,
}

/// `vue.sync` - Vue 2's `:foo.sync="bar"` two-way bind sugar.
///
/// The bounded subset the shipped pre-transform expands: a static
/// argument, a value expression, and the `sync` modifier. Remaining
/// modifiers (`camel`, …) ride here so the desugar pass can put them
/// on the resulting `ui.bind`. Dynamic-argument `:\[foo].sync` stays
/// `ui.bind` — the shipped lane skips it the same way. The name is a
/// static identifier because desugar emits `update:<name>` and cannot
/// spell that from a computed argument.
#[derive(Debug)]
pub struct VueSyncOp<'a> {
    /// The bound prop name (static; dynamic `.sync` never reaches here).
    pub name: &'a str,
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

/// `vue.once` - Vue's `v-once` one-shot render flag.
///
/// A presence op: the well-formed spelling is the bare directive
/// (no argument, no modifier, no value). Realization (`has_v_once` in
/// the shipped codegen) reads the flag; this increment only names it.
#[derive(Debug)]
pub struct VueOnceOp {
    /// The whole directive's source range.
    pub span: Span,
}

/// `vue.memo` - Vue's `v-memo` dependency-memoized render.
///
/// The well-formed spelling carries a value expression and no
/// argument or modifier. The expression rides as [`ExprRef`] under
/// P2-5b — opaque is allowed (the shipped extractor's comment-bearing
/// contents have no retained AST). Realization (`get_memo_exp`) is
/// later; this increment only names the op.
#[derive(Debug)]
pub struct VueMemoOp<'a> {
    /// The memo dependency expression, as authored.
    pub value: ExprRef<'a>,
    /// The whole directive's source range.
    pub span: Span,
}

/// `vue.show` - Vue's `v-show` runtime display toggle.
///
/// It stays a Vue dialect binding rather than a custom `vue.directive`:
/// the runtime helper is `vShow`, not a resolved user directive, and the
/// builtin remains visible to consumers that want to reason about
/// display toggles before DOM realization.
#[derive(Debug)]
pub struct VueShowOp<'a> {
    /// The display predicate expression, as authored.
    pub value: ExprRef<'a>,
    /// The whole directive's source range.
    pub span: Span,
}

/// `vue.html` - Vue's `v-html` raw-HTML property realization.
///
/// It stays a Vue dialect binding instead of a plain `ui.bind` because
/// authored `v-html` is not equivalent to a user prop: it writes the
/// special `innerHTML` DOM prop and consumers may want to flag or
/// isolate that raw-HTML surface before DOM realization.
#[derive(Debug)]
pub struct VueHtmlOp<'a> {
    /// The raw HTML expression, when authored.
    pub value: Option<ExprRef<'a>>,
    /// The whole directive's source range.
    pub span: Span,
}

/// `vue.text` - Vue's `v-text` text-content property realization.
///
/// It stays a Vue dialect binding instead of a plain `ui.bind`: authored
/// `v-text` is not a user prop, it forces the DOM `textContent` prop through
/// Vue's display-string coercion.
#[derive(Debug)]
pub struct VueTextOp<'a> {
    /// The text-content expression, when authored.
    pub value: Option<ExprRef<'a>>,
    /// The whole directive's source range.
    pub span: Span,
}

/// See [`crate::op`] for the guard rationale.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<VueDirectiveOp<'_>>() == 88);
    assert!(core::mem::size_of::<VueCssBindOp<'_>>() == 24);
    assert!(core::mem::size_of::<VueSyncOp<'_>>() == 64);
    assert!(core::mem::size_of::<VueSlotScopeOp<'_>>() == 40);
    assert!(core::mem::size_of::<VueOnceOp>() == 8);
    assert!(core::mem::size_of::<VueMemoOp<'_>>() == 24);
    assert!(core::mem::size_of::<VueShowOp<'_>>() == 24);
    assert!(core::mem::size_of::<VueHtmlOp<'_>>() == 24);
    assert!(core::mem::size_of::<VueTextOp<'_>>() == 24);
};

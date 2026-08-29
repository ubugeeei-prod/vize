//! `ui.slot` - the slot outlet, its content-side dual `ui.slot-content`,
//! and the static-or-computed name position they share with
//! `vue.directive` arguments.

use vize_s0::{Span, Vec};

use super::Region;
use crate::expr::ExprRef;

/// A name position that may be authored statically or computed at runtime:
/// slot names (`<slot :name="n">`) and directive arguments
/// (`v-pin:[direction]`).
///
/// No `PartialEq`, following [`ExprRef`]: the dynamic case must not be
/// comparable (pessimal law 4 applies to whatever the expression is).
#[derive(Debug, Clone, Copy)]
pub enum DynamicName<'a> {
    /// Authored as a literal name, a slice of the source.
    Static(&'a str),
    /// Computed by an expression.
    Dynamic(ExprRef<'a>),
}

/// `ui.slot` - a slot outlet owning its fallback region.
///
/// Since the P2-9 element/binding-family installment the outlet carries
/// its own props surface - the slot props the parent's slot function
/// receives: static attributes plus attached one-way bindings
/// ([`super::BindOp`] / [`super::OnOp`]). The `name` position is not part
/// of that surface (it selects the slot, it is not passed to it), which
/// is why it stays a dedicated field.
#[derive(Debug)]
pub struct SlotOp<'a> {
    /// The outlet name (`default` when the dialect leaves it implicit is a
    /// lowering decision, not this type's).
    pub name: DynamicName<'a>,
    /// Static slot props (`name` excluded), in authored order.
    pub attributes: Vec<'a, super::Attribute<'a>>,
    /// Attached bindings, in authored order.
    pub bindings: Vec<'a, super::BindingOp<'a>>,
    /// Content rendered when the parent passes nothing.
    pub fallback: Region<'a>,
    /// The outlet's full source range.
    pub span: Span,
}

/// `ui.slot-content` - one authored `v-slot` spelling on its carrier (a
/// component or a `<template>` child), attached as a binding op.
///
/// The dual of [`SlotOp`]: the outlet renders a slot, this *provides*
/// one. The op is the **syntactic surface** the lowering keeps - name
/// position and params exactly as authored, an unauthored name left
/// `None` (never pre-normalized to `default`: the default-name synthesis
/// is the slot-normalization pass's, where the authored-vs-synthesized
/// origin is a recorded fact). Canonical grouping - which region feeds
/// which slot, the implicit default, the dialect's modifier folding -
/// is that pass's published fact, not this type's.
///
/// Fair by the P2-16 litmus: named/parameterized slot content exists in
/// SFC and JSX alike (the JSX lowering keeps object-literal `v-slots`
/// entries as synthetic template children); only the modifier spelling
/// is Vue's, carried verbatim like [`super::VueDirectiveOp`] does.
#[derive(Debug)]
pub struct SlotContentOp<'a> {
    /// The authored slot name (`#header`, `v-slot:[expr]`); `None` when
    /// the spelling leaves it implicit (bare `v-slot`).
    pub name: Option<DynamicName<'a>>,
    /// Modifier names in authored order, without their leading dots.
    pub modifiers: Vec<'a, &'a str>,
    /// The authored slot-props position (identifier or pattern), when
    /// non-blank.
    pub params: Option<ExprRef<'a>>,
    /// The whole directive's source range.
    pub span: Span,
}

/// See [`crate::op`] for the guard rationale. `SlotOp` moved 56 → 104
/// when the outlet gained its props surface (two 24-byte arena vectors).
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<DynamicName<'_>>() == 24);
    assert!(core::mem::size_of::<SlotOp<'_>>() == 104);
    assert!(core::mem::size_of::<SlotContentOp<'_>>() == 72);
};

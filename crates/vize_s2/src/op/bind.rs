//! `ui.bind` / `ui.on` - the normalized one-way binding ops, landed with
//! the transform that needs them (P2-9, the element/binding family).
//!
//! Fair by the P2-16 litmus: one-way property bindings and event handlers
//! exist in every dialect the pivot serves (SFC directives, JSX props,
//! Svelte attributes/`on:`); only the modifier spellings are Vue's,
//! carried verbatim exactly as [`super::SlotContentOp`] carries its
//! modifiers. The ops are the **syntactic surface** the lowering keeps -
//! name position, modifiers, and value exactly as authored (the one
//! normalization is the lowering's, recorded in provenance: the parser's
//! same-name shorthand `:foo` expands to its camelized value, and the
//! `.foo` dot shorthand synthesizes its leading `prop` modifier, both
//! matching the shipped parser byte-for-byte). What a target *does* with
//! a binding - patch flags, `class`/`style` normalization, `withModifiers`
//! guards, event-name casing - is DOM realization (P2-11), never carried
//! here.

use vize_s0::{Span, Vec};

use super::DynamicName;
use crate::expr::ExprRef;

/// `ui.bind` - one one-way binding (`v-bind:` / `:` / the `.` shorthand),
/// attached to an element, component, or slot outlet.
#[derive(Debug)]
pub struct BindOp<'a> {
    /// The bound name (`:id`, `:[key]`); `None` for the object-spread
    /// form (`v-bind="obj"`).
    pub name: Option<DynamicName<'a>>,
    /// Modifier names in authored order, without their leading dots
    /// (`camel`, `prop`, `attr` - `prop` synthesized for the `.` shorthand
    /// exactly as the shipped parser does).
    pub modifiers: Vec<'a, &'a str>,
    /// The bound value. `None` only when the spelling has no value and no
    /// same-name expansion applies (a dynamic-argument `:[k]` without a
    /// value); a valueless static argument expands at lowering.
    pub value: Option<ExprRef<'a>>,
    /// The whole directive's source range.
    pub span: Span,
}

/// `ui.on` - one event handler binding (`v-on:` / `@`), attached to an
/// element, component, or slot outlet.
#[derive(Debug)]
pub struct OnOp<'a> {
    /// The event name (`@click`, `@[event]`); `None` for the object form
    /// (`v-on="handlers"`).
    pub name: Option<DynamicName<'a>>,
    /// Modifier names in authored order, without their leading dots.
    pub modifiers: Vec<'a, &'a str>,
    /// The handler expression; `None` for a bare listener spelling
    /// (`@click.stop` with no value).
    pub handler: Option<ExprRef<'a>>,
    /// The whole directive's source range.
    pub span: Span,
}

/// See [`crate::op`] for the guard rationale.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<BindOp<'_>>() == 72);
    assert!(core::mem::size_of::<OnOp<'_>>() == 72);
};

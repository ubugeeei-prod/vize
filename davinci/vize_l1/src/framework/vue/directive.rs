//! Vue directive attribute names, decomposed by the [`DirectiveSyntax`] hook.
//!
//! Results borrow source coordinates rather than copying attribute text;
//! decomposition allocates nothing.
//! The concrete syntax policy belongs to each version's table.

pub use crate::markup::grammar::DirectiveSyntax;
use vize_l0::Span;

/// How the directive was spelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectivePrefix {
    /// `v-name`.
    Full,
    /// `:arg` (`v-bind`).
    Bind,
    /// `.arg` (`v-bind` with `.prop`).
    Prop,
    /// `@arg` (`v-on`).
    On,
    /// `#arg` (`v-slot`).
    Slot,
}

/// A directive argument, preserving recovered incomplete input too.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgSyntax {
    /// The static argument bytes.
    Static(Span),
    /// The bytes between brackets, or the recovered bytes after an unclosed `[`.
    Dynamic(Span),
}

/// A decomposed directive attribute name. All spans are absolute UTF-8 offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectiveName {
    pub prefix: DirectivePrefix,
    /// The name without `v-`; shorthand names are empty at the prefix.
    pub name: Span,
    pub arg: Option<ArgSyntax>,
    /// The modifier run including its leading dot; empty at the head's end
    /// when absent. Empty modifiers remain in this run for diagnostics.
    pub modifiers: Span,
}

/// Source admission failure, distinct from a plain attribute or recovered syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveNameError {
    /// The attribute's absolute byte range does not fit the source span model.
    OffsetOverflow,
    /// The syntax provider's allocation-free nesting capacity was exceeded.
    NestingLimit,
}

/// Transitional public path; the implementation is owned by the Vue dialect.
pub use crate::framework::vue::version::v3::VueDirectives;
pub use crate::framework::vue::version::v3::directive::frozen_attribute_name;

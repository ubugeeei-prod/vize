//! Generic contracts for dialect-provided directive attribute-name syntax.
//!
//! Results borrow source coordinates rather than copying attribute text;
//! decomposition allocates nothing.
//! Concrete syntax policy belongs to the dialect modules.

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

/// A dialect hook recognizing and decomposing a complete raw attribute head.
pub trait DirectiveSyntax {
    /// `Ok(None)` means a plain attribute. Recognized malformed syntax retains
    /// its source evidence in `Ok(Some(_))`; lexical diagnostics are separate.
    /// Admission errors never silently turn a directive into a plain attribute.
    fn decompose(
        &self,
        name: &str,
        offset: u32,
    ) -> Result<Option<DirectiveName>, DirectiveNameError>;
}

/// Transitional public path; the implementation is owned by the Vue dialect.
pub use crate::dialect::vue3::VueDirectives;

//! Dialect syntax hooks for directive attribute names.
//!
//! The generic markup lexer reads structure only. How an attribute name
//! decomposes into a directive (`v-on:click.stop`, `@click`, `#default`,
//! `:[dyn]`) is a hook the dialect provides, the role MLIR custom assembly
//! formats play. Results are spans into the source, so decomposition
//! allocates nothing.

#![expect(clippy::todo, reason = "skeleton: #6836")]

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

/// A directive argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgSyntax {
    /// `:title`: the argument bytes.
    Static(Span),
    /// `:[key]`: the bytes between the brackets, later an expression embed.
    Dynamic(Span),
}

/// A decomposed directive attribute name. Spans are absolute source offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectiveName {
    pub prefix: DirectivePrefix,
    /// The directive name without `v-` (`on`, `bind`, `my-dir`); for
    /// shorthands the span is empty at the prefix.
    pub name: Span,
    pub arg: Option<ArgSyntax>,
    /// The `.a.b` modifier run, including its leading dot; empty when there
    /// are none. Split on `.` to iterate without allocating.
    pub modifiers: Span,
}

/// The dialect hook that recognizes and decomposes directive names.
pub trait DirectiveSyntax {
    /// Decompose the attribute name `name`, which starts at byte `offset`.
    /// `None` means the attribute is a plain attribute.
    fn decompose(&self, name: &str, offset: u32) -> Option<DirectiveName>;
}

/// Vue 3 directive-name syntax.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VueDirectives;

impl DirectiveSyntax for VueDirectives {
    fn decompose(&self, _name: &str, _offset: u32) -> Option<DirectiveName> {
        todo!("#6836: decompose Vue directive names in the dialect syntax hook")
    }
}

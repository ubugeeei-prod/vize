//! Expression rewriting by span, from the L2 identifier-resolution table.
//!
//! Expressions are parsed once, in L1. L2 records how each identifier
//! reference resolves (setup binding, prop, render context, template local,
//! …) in a side table keyed by authored span. L4 copies the authored bytes of
//! an expression and splices the accessor for each resolved identifier at its
//! span: no reparse, no string search, and each rewritten identifier gets a
//! named span link for free. DOM, SSR, Vapor and the type-check projection
//! share this one rewriter.
//!
//! The resolution vocabulary here is the consumer view; the table itself is
//! produced by L2 (#6838). Until that producer exists, [`write_expression`] is
//! unfinished.

#![expect(clippy::todo, reason = "skeleton: #6840")]

use vize_l0::Span;

use crate::write::{LinkSink, Writer};

/// How one identifier reference resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// A `<script setup>` binding that is never a ref.
    SetupConst,
    /// A `<script setup>` binding that may be a ref (needs unwrapping).
    SetupMaybeRef,
    /// A `<script setup>` ref binding.
    SetupRef,
    /// A declared prop.
    Props,
    /// A destructured prop read through its alias.
    PropsAliased,
    /// A `data()` field.
    Data,
    /// An options-API member (`computed`, `methods`, …).
    Options,
    /// Anything else on the render context.
    Context,
    /// A template-local binding (`v-for`, slot props); written verbatim.
    Local,
    /// An allow-listed global (`Math`, `Date`, …); written verbatim.
    Global,
}

/// One resolved identifier: its authored span and resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolved {
    pub span: Span,
    pub access: Access,
}

/// The identifiers of one compile unit, sorted by `span.start`.
#[derive(Debug, Clone, Copy)]
pub struct ResolutionTable<'a> {
    pub identifiers: &'a [Resolved],
}

impl<'a> ResolutionTable<'a> {
    /// The resolved identifiers inside `span`, in authored order.
    #[must_use]
    pub fn within(&self, span: Span) -> &'a [Resolved] {
        let identifiers = self.identifiers;
        let start = identifiers.partition_point(|entry| entry.span.start < span.start);
        let rest = identifiers.get(start..).unwrap_or_default();
        let len = rest.partition_point(|entry| entry.span.end <= span.end);
        rest.get(..len).unwrap_or_default()
    }
}

/// Which accessor spelling the enclosing render function uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessStyle {
    /// A separate render function: `_ctx.x`, `$setup.x`, `$props.x`, ….
    Function,
    /// A render function inlined into `setup()`: bindings are in scope, refs
    /// are read through `.value` or `_unref(x)`.
    Inline,
    /// The server renderer's `_ctx` / `$setup` parameters.
    Server,
}

/// Write the expression authored at `span` of `source`, rewriting every
/// identifier `table` resolved inside it for `style`.
///
/// Authored bytes between identifiers are copied verbatim; each rewritten
/// identifier records a named link to its authored span.
pub fn write_expression<L: LinkSink>(
    _writer: &mut Writer<L>,
    _source: &str,
    _span: Span,
    _table: &ResolutionTable<'_>,
    _style: AccessStyle,
) {
    todo!("#6840: splice accessors from the L2 resolution table (needs #6838)")
}

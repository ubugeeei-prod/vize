//! Consuming original callback syntax without losing a refused handoff.

use alloc::{boxed::Box, vec::Vec};
use oxc_ast::ast::Expression;
use oxc_span::SourceType;
use vize_l0::{SourceBlock, Span};

use super::{FilterChain, FilterInvocation, TextBinding, TextBoundary};
use crate::embed::syntax::{
    CommentView, DiagnosticView, EmbedHole, EmbedSyntax, RetainedExpression,
};
use crate::embed::{EmbedSource, Grammar, SourceError};

/// One actual original expression observation, including a refused handoff.
/// Only the existing consuming parser API supplies an arena-lived root.
/// An unexpected shape stays in its original normally owned syntax artifact.
///
/// Its refused owner's borrow cannot be widened to the source/arena lifetime:
/// ```compile_fail
/// use vize_l1::{framework::vue::version::v2::text::TextExpression, embed::syntax::EmbedSyntax};
/// fn widen<'o, 'a>(slot: &'o TextExpression<'a>) -> &'a EmbedSyntax<'a> {
///     slot.original().unwrap()
/// }
/// ```
#[derive(Debug)]
pub struct TextExpression<'a> {
    artifact: RetainedPart<'a>,
}

#[derive(Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "Keep successful owning handoffs inline to avoid a heap allocation per callback/filter slot"
)]
enum RetainedPart<'a> {
    Retained(RetainedExpression<'a>),
    Original(Box<EmbedSyntax<'a>>),
}

impl<'a> TextExpression<'a> {
    fn consume(syntax: EmbedSyntax<'a>) -> Self {
        let artifact = match syntax.into_expression() {
            Ok(retained) => RetainedPart::Retained(retained),
            Err(original) => RetainedPart::Original(original),
        };
        Self { artifact }
    }

    pub fn retained(&self) -> Option<&RetainedExpression<'a>> {
        match &self.artifact {
            RetainedPart::Retained(retained) => Some(retained),
            RetainedPart::Original(_) => None,
        }
    }

    /// A refusal preserves the complete actual owner, never a replacement hole.
    pub fn original(&self) -> Option<&EmbedSyntax<'a>> {
        match &self.artifact {
            RetainedPart::Retained(_) => None,
            RetainedPart::Original(original) => Some(original),
        }
    }

    pub fn grammar(&self) -> Grammar {
        match &self.artifact {
            RetainedPart::Retained(retained) => retained.grammar(),
            RetainedPart::Original(original) => original.grammar(),
        }
    }

    pub fn source_type(&self) -> SourceType {
        match &self.artifact {
            RetainedPart::Retained(retained) => retained.source_type(),
            RetainedPart::Original(original) => original.source_type(),
        }
    }

    pub fn source(&self) -> EmbedSource<'a> {
        match &self.artifact {
            RetainedPart::Retained(retained) => retained.source(),
            RetainedPart::Original(original) => original.source(),
        }
    }

    pub fn hole(&self) -> Option<EmbedHole> {
        match &self.artifact {
            RetainedPart::Retained(retained) => retained.hole(),
            RetainedPart::Original(original) => original.hole(),
        }
    }

    /// This is the relocated original root in the original parser arena.
    /// Its enum address may change during handoff; its payload is not cloned.
    pub fn expression(&self) -> Option<&'a Expression<'a>> {
        self.retained()?.expression()
    }

    pub fn comments(&self) -> impl Iterator<Item = CommentView<'_, 'a>> {
        self.retained()
            .into_iter()
            .flat_map(|owner| owner.comments())
            .chain(
                self.original()
                    .into_iter()
                    .flat_map(|owner| owner.comments()),
            )
    }

    pub fn diagnostics(&self) -> impl Iterator<Item = DiagnosticView<'_, 'a>> {
        self.retained()
            .into_iter()
            .flat_map(|owner| owner.diagnostics())
            .chain(
                self.original()
                    .into_iter()
                    .flat_map(|owner| owner.diagnostics()),
            )
    }

    pub fn decoded_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        match &self.artifact {
            RetainedPart::Retained(retained) => retained.decoded_span(span),
            RetainedPart::Original(original) => original.decoded_span(span),
        }
    }

    pub fn authored_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        match &self.artifact {
            RetainedPart::Retained(retained) => retained.authored_span(span),
            RetainedPart::Original(original) => original.authored_span(span),
        }
    }
}

/// Original filter name and argument occurrences in their original order.
#[derive(Debug)]
pub struct RetainedFilterInvocation<'a> {
    span: Span,
    name: EmbedSource<'a>,
    arguments: Vec<TextExpression<'a>>,
}

impl<'a> RetainedFilterInvocation<'a> {
    pub const fn span(&self) -> Span {
        self.span
    }
    pub const fn name(&self) -> EmbedSource<'a> {
        self.name
    }
    pub fn arguments(&self) -> &[TextExpression<'a>] {
        &self.arguments
    }
}

/// Every original base and argument, including holes and handoff refusals.
#[derive(Debug)]
pub struct RetainedFilterChain<'a> {
    base: TextExpression<'a>,
    filters: Vec<RetainedFilterInvocation<'a>>,
}

impl<'a> RetainedFilterChain<'a> {
    pub fn base(&self) -> &TextExpression<'a> {
        &self.base
    }
    pub fn filters(&self) -> &[RetainedFilterInvocation<'a>] {
        &self.filters
    }
    fn is_admitted(&self) -> bool {
        self.base.expression().is_some()
            && self.filters.iter().all(|filter| {
                filter
                    .arguments
                    .iter()
                    .all(|argument| argument.expression().is_some())
            })
    }
}

/// A moved authentic callback, retaining its complete untrimmed authored window.
/// Inspection confers expression syntax, not CST-child or runtime admission.
#[derive(Debug)]
pub struct RetainedTextBinding<'a> {
    span: Span,
    raw_content: &'a str,
    source: EmbedSource<'a>,
    chain: Option<RetainedFilterChain<'a>>,
    boundaries: Vec<TextBoundary>,
}

impl<'a> RetainedTextBinding<'a> {
    pub const fn span(&self) -> Span {
        self.span
    }
    pub const fn raw_content(&self) -> &'a str {
        self.raw_content
    }
    pub const fn source(&self) -> EmbedSource<'a> {
        self.source
    }
    pub(in crate::framework::vue::version::v2) fn matches(
        &self,
        block: SourceBlock<'a>,
        raw: &str,
    ) -> bool {
        core::ptr::eq(self.raw_content, raw)
            && core::ptr::eq(self.source.authored_root(), block.root_source())
            && block.span_of(raw).map(|content| {
                Span::new(
                    content.start.saturating_sub(2),
                    (content.end + 2).min(block.end()),
                )
            }) == Some(self.span)
    }
    pub fn chain(&self) -> Option<&RetainedFilterChain<'a>> {
        self.chain.as_ref()
    }
    pub fn boundaries(&self) -> &[TextBoundary] {
        &self.boundaries
    }
    pub fn admitted(&self) -> Option<&RetainedFilterChain<'a>> {
        self.chain
            .as_ref()
            .filter(|chain| self.boundaries.is_empty() && chain.is_admitted())
    }
}

pub(super) fn consume(binding: TextBinding<'_>) -> RetainedTextBinding<'_> {
    let TextBinding {
        span,
        raw_content,
        source,
        chain,
        boundaries,
    } = binding;
    RetainedTextBinding {
        span,
        raw_content,
        source,
        chain: chain.map(|FilterChain { base, filters }| RetainedFilterChain {
            base: TextExpression::consume(base),
            filters: filters
                .into_iter()
                .map(
                    |FilterInvocation {
                         span,
                         name,
                         arguments,
                     }| {
                        RetainedFilterInvocation {
                            span,
                            name,
                            arguments: arguments.into_iter().map(TextExpression::consume).collect(),
                        }
                    },
                )
                .collect(),
        }),
        boundaries,
    }
}

#[cfg(test)]
mod tests;

//! Fixed-expression grammar with bounded lookahead for one terminal operand.
use super::{
    EmbeddingGoal, EmbeddingHole, EmbeddingInput, EmbeddingObservation, ParseOptions, Parser,
    shapes,
};
use crate::config::{NoTokensLexerConfig, ParserConfig};

#[derive(Default)]
pub(crate) struct TerminalExpression;

impl ParserConfig for TerminalExpression {
    type LexerConfig = NoTokensLexerConfig;

    #[inline]
    fn lexer_config(&self) -> Self::LexerConfig {
        NoTokensLexerConfig
    }
}

impl<'a> EmbeddingInput<'a> {
    /// Observe this occurrence with the ordinary parser, original arena, lexer
    /// and default options. A private static grammar specialization can select
    /// the parenthesis and primary productions when the original token covers
    /// the complete authored expression between the fixed delimiters.
    #[must_use]
    pub fn observe_terminal_expression(self) -> EmbeddingObservation<'a> {
        if self.goal != EmbeddingGoal::Expr
            || self.content.is_empty()
            || self.content.len() > 64
            || !self.content.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'$'))
        {
            return self.observe();
        }
        let source = self.allocator.alloc_str(self.wrapper.as_str());
        let options = ParseOptions::default();
        let parsed = Parser::new(self.allocator, source, self.source_type)
            .with_options(options)
            .with_config(TerminalExpression)
            .parse();
        include!("result.rs")
    }
}

/// Fixed parser input borrowed from an actual expression observation.
/// This receipt can only prepare a byte-identical occurrence in the same grammar.
#[derive(Clone, Copy)]
pub struct PreparedExpression<'a> {
    allocator: &'a oxc_allocator::Allocator,
    content_span: oxc_span::Span,
    source: &'a str,
    source_type: oxc_span::SourceType,
}

impl<'a> EmbeddingObservation<'a> {
    #[must_use]
    pub fn prepared_expression(&self) -> Option<PreparedExpression<'a>> {
        (self.goal == EmbeddingGoal::Expr).then_some(PreparedExpression {
            allocator: self.allocator,
            content_span: self.content_span,
            source: self.parsed.program.source_text,
            source_type: self.source_type,
        })
    }
}

impl<'a> PreparedExpression<'a> {
    /// The new occurrence uses its own content identity, input and parser arena allocation.
    pub fn prepare_matching(
        self,
        content: &'a str,
        source_type: oxc_span::SourceType,
    ) -> Option<EmbeddingInput<'a>> {
        let range = self.content_span.start as usize..self.content_span.end as usize;
        if self.source_type != source_type || self.source.get(range) != Some(content) {
            return None;
        }
        Some(EmbeddingInput {
            allocator: self.allocator,
            content,
            source_type,
            goal: EmbeddingGoal::Expr,
            wrapper: self.source.into(),
            content_span: self.content_span,
        })
    }
}

#[cfg(test)]
mod tests;

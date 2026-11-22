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

#[cfg(test)]
mod tests;

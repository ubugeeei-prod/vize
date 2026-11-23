//! Reuse byte-identical short wrapper input and its resource decision. Each
//! authored occurrence retains its own parser source and complete OXC result.
use super::{
    Coordinates, Diagnostics, EmbedHole, EmbedSource, EmbedSyntax, Grammar, RetainedExpression,
    Shape,
};
use crate::embed::{
    Lang,
    syntax::{ProgramOptions, admission, wrapped},
};
use alloc::boxed::Box;
use oxc_parser::{EmbeddingGoal, EmbeddingInput, EmbeddingInputError, EmbeddingObservation};
use vize_l0::{Allocator, expression_guard::expression_is_safe_to_parse};

pub struct RetainedExpressionParser<'a> {
    allocator: &'a Allocator,
    recent: [Option<oxc_parser::PreparedExpression<'a>>; 2],
}

impl<'a> RetainedExpressionParser<'a> {
    #[must_use]
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            recent: [None, None],
        }
    }

    /// Parse and retain the original OXC result, diagnostic, comment and
    /// coordinates for this occurrence. The sequence owns no cross-file state.
    pub fn parse(
        &mut self,
        source: EmbedSource<'a>,
        lang: Lang,
    ) -> Result<RetainedExpression<'a>, Box<EmbedSyntax<'a>>> {
        let grammar = Grammar {
            shape: Shape::Expr,
            lang,
        };
        let source_type = ProgramOptions::module(lang).source_type();
        let rejected = |prefix, hole| {
            Ok(RetainedExpression {
                grammar,
                source_type,
                coordinates: Coordinates { source, prefix },
                observation: None,
                diagnostics: Diagnostics::default(),
                hole: Some(hole),
            })
        };
        let text = source.text();
        let short = text.len() <= 64;
        if short
            && let Some(input) = self
                .recent
                .iter()
                .flatten()
                .find_map(|prior| prior.prepare_matching(text, source_type))
        {
            return retain(grammar, source, input.observe_terminal_expression());
        }
        let input = match EmbeddingInput::prepare_in(
            self.allocator.as_oxc(),
            text,
            source_type,
            EmbeddingGoal::Expr,
        ) {
            Ok(input) => input,
            Err(error) => {
                return rejected(
                    0,
                    match error {
                        EmbeddingInputError::SourceTooLarge => EmbedHole::SourceTooLarge,
                        EmbeddingInputError::UnsupportedProfile => EmbedHole::UnsupportedShape,
                    },
                );
            }
        };
        let prefix = input.parser_content_span().start;
        if !admission::allows_small_input(input.parser_source()) {
            return rejected(prefix, EmbedHole::TokenBudget);
        }
        if !expression_is_safe_to_parse(input.parser_source()) {
            return rejected(prefix, EmbedHole::SafetyAdmission);
        }
        let owner = input.observe_terminal_expression();
        if short && let Some(prepared) = owner.prepared_expression() {
            let [older, latest] = &mut self.recent;
            *older = latest.replace(prepared);
        }
        retain(grammar, source, owner)
    }
}

#[inline]
fn retain<'a>(
    grammar: Grammar,
    source: EmbedSource<'a>,
    owner: EmbeddingObservation<'a>,
) -> Result<RetainedExpression<'a>, Box<EmbedSyntax<'a>>> {
    let coordinates = Coordinates {
        source,
        prefix: owner.parser_content_span().start,
    };
    let source_type = owner.source_type();
    let hole = owner.hole().map(wrapped::hole);
    match owner.into_expression() {
        Ok(observed) => Ok(RetainedExpression {
            grammar,
            source_type,
            coordinates,
            observation: Some(observed),
            diagnostics: Diagnostics::default(),
            hole,
        }),
        Err(owner) => Err(Box::new(EmbedSyntax {
            grammar,
            source_type,
            coordinates,
            observation: None,
            embedding: Some(*owner),
            diagnostics: Diagnostics::default(),
            hole,
        })),
    }
}

#[cfg(test)]
mod tests;

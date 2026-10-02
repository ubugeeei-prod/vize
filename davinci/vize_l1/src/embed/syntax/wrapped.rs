//! One original parser-owned embedding, after unchanged local resource gates.

use oxc_diagnostics::Diagnostics;
use oxc_parser::{EmbeddingGoal, EmbeddingHole, EmbeddingInput, EmbeddingInputError};
use vize_l0::{Allocator, expression_guard::expression_is_safe_to_parse};

use super::{
    Embed, EmbedHole, NativeSyntax, ProgramOptions, Shape, admission, coordinates::Coordinates,
};

pub(super) const fn hole(hole: EmbeddingHole) -> EmbedHole {
    match hole {
        EmbeddingHole::Syntax => EmbedHole::Syntax,
        EmbeddingHole::UnsupportedFlow => EmbedHole::UnsupportedFlow,
        EmbeddingHole::InvalidExpressionShape => EmbedHole::InvalidExpressionShape,
        EmbeddingHole::InvalidWrappedShape => EmbedHole::InvalidWrappedShape,
        EmbeddingHole::InvalidModuleContext => EmbedHole::InvalidModuleContext,
        EmbeddingHole::InvalidParameterContext => EmbedHole::InvalidParameterContext,
    }
}

pub(super) fn parse<'a>(allocator: &'a Allocator, embed: Embed<'a>) -> NativeSyntax<'a> {
    let source_type = ProgramOptions::module(embed.grammar.lang).source_type();
    let goal = match embed.grammar.shape {
        Shape::Expr => Some(EmbeddingGoal::Expr),
        Shape::HandlerBody => Some(EmbeddingGoal::HandlerBody),
        Shape::SlotParams => Some(EmbeddingGoal::Parameters),
        _ => None,
    };
    let mut result = NativeSyntax {
        grammar: embed.grammar,
        source_type,
        coordinates: Coordinates {
            source: embed.source,
            prefix: 0,
        },
        observation: None,
        embedding: None,
        diagnostics: Diagnostics::default(),
        hole: None,
    };
    let Some(goal) = goal else {
        result.hole = Some(EmbedHole::UnsupportedShape);
        return result;
    };
    let input = match EmbeddingInput::prepare_in(
        allocator.as_oxc(),
        embed.source.text(),
        source_type,
        goal,
    ) {
        Ok(input) => input,
        Err(error) => {
            result.hole = Some(match error {
                EmbeddingInputError::SourceTooLarge => EmbedHole::SourceTooLarge,
                EmbeddingInputError::UnsupportedProfile => EmbedHole::UnsupportedShape,
            });
            return result;
        }
    };
    result.coordinates.prefix = input.parser_content_span().start;
    if !admission::allows_small_input(input.parser_source()) {
        result.hole = Some(EmbedHole::TokenBudget);
        return result;
    }
    if !expression_is_safe_to_parse(input.parser_source()) {
        result.hole = Some(EmbedHole::SafetyAdmission);
        return result;
    }
    let observation = input.observe();
    result.hole = observation.hole().map(hole);
    result.embedding = Some(observation);
    result
}

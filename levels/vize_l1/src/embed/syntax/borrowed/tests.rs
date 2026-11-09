//! API custody laws over actual original stock observations, not syntax fixtures.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::string_slice,
    reason = "independent original-owner/source/AST laws fail on invalid evidence"
)]

use oxc_parser::{EmbeddingGoal, EmbeddingInput, EmbeddingObservation};
use oxc_span::SourceType;
use vize_l0::{Allocator, Span};

use super::super::{Embed, EmbedSource, Grammar, Lang, NativeSyntax, Shape, parse_once};

mod custody;
mod historical;
mod profile;
mod stock;

fn syntax<'a>(arena: &'a Allocator, text: &'a str, shape: Shape) -> NativeSyntax<'a> {
    parse_once(
        arena,
        Embed {
            source: EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap(),
            grammar: Grammar {
                shape,
                lang: Lang::Js,
            },
        },
    )
}

fn observe<'a>(
    arena: &'a Allocator,
    text: &'a str,
    goal: EmbeddingGoal,
) -> EmbeddingObservation<'a> {
    EmbeddingInput::prepare_in(arena.as_oxc(), text, SourceType::mjs(), goal)
        .unwrap()
        .observe()
}

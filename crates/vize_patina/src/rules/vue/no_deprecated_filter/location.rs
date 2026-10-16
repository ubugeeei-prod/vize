//! Authored filter sequence ranges, matching the input expression AST.

use super::find_filter_pipe;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_l0::expression_guard::expression_is_safe_to_parse;
use vize_l1::markup::entity::{EntityContext, decode_one};
use vize_relief::{SimpleExpressionNode, SourceLocation};

pub(super) fn filter_location(
    expression: &SimpleExpressionNode<'_>,
    authored: &str,
    context: EntityContext,
) -> Option<SourceLocation> {
    let source = expression.content;
    let pipe = find_filter_pipe(source)?;
    let mut start = expression.loc.span.start;
    // A filter sequence starts at its input AST, not optional grouping around
    // that input. Calls, member access and conditionals retain their own spans.
    // Parse only flagged, parenthesized inputs; the usual path stays lexical.
    if source.trim_start().starts_with('(')
        && let Some(input_source) = source.get(..pipe)
        && expression_is_safe_to_parse(input_source)
    {
        let allocator = Allocator::default();
        if let Ok(input) =
            Parser::new(&allocator, input_source, SourceType::ts()).parse_expression()
        {
            let decoded_start = input.get_inner_expression().span().start as usize;
            if let Some(raw) =
                authored.get(expression.loc.span.start as usize..expression.loc.span.end as usize)
                && let Some(offset) = authored_offset(raw, source, decoded_start, context)
            {
                start += offset as u32;
            }
        }
    }
    Some(SourceLocation::new(start, expression.loc.span.end))
}

// Map only the admitted input prefix. Entity-free expressions use their exact
// borrowed span; decoded entities advance by their complete authored spelling.
// A mismatched decoding retains the lexical range instead of inventing a span.
fn authored_offset(
    raw: &str,
    decoded: &str,
    offset: usize,
    context: EntityContext,
) -> Option<usize> {
    if raw == decoded {
        return decoded.is_char_boundary(offset).then_some(offset);
    }
    let mut raw_position = 0;
    let mut decoded_position = 0;
    while decoded_position < offset {
        let rest = raw.get(raw_position..)?;
        let mut characters = decoded.get(decoded_position..)?.chars();
        if rest.starts_with('&')
            && let Some((entity, consumed)) = decode_one(rest.as_bytes(), context)
        {
            let mut matches = true;
            entity.for_each(|character| {
                matches &= characters.next() == Some(character);
                decoded_position += character.len_utf8();
            });
            if !matches {
                return None;
            }
            raw_position += consumed;
        } else {
            let character = rest.chars().next()?;
            if characters.next() != Some(character) {
                return None;
            }
            raw_position += character.len_utf8();
            decoded_position += character.len_utf8();
        }
    }
    (decoded_position == offset).then_some(raw_position)
}

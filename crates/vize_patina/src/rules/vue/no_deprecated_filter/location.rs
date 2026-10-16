//! Authored filter sequence ranges, matching the input expression AST.

use super::find_filter_pipe;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_relief::{SimpleExpressionNode, SourceLocation};

pub(super) fn filter_location(expression: &SimpleExpressionNode<'_>) -> Option<SourceLocation> {
    let source = expression.content;
    let pipe = find_filter_pipe(source)?;
    let mut start = expression.loc.span.start;
    // A filter sequence starts at its input AST, not optional grouping around
    // that input. Calls, member access and conditionals retain their own spans.
    // Parse only flagged, parenthesized inputs; the usual path stays lexical.
    if source.trim_start().starts_with('(')
        && let Some(input_source) = source.get(..pipe)
    {
        let allocator = Allocator::default();
        if let Ok(input) =
            Parser::new(&allocator, input_source, SourceType::ts()).parse_expression()
        {
            start += input.get_inner_expression().span().start;
        }
    }
    Some(SourceLocation::new(start, expression.loc.span.end))
}

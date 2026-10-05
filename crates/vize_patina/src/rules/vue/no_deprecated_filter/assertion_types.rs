//! Cold type-grammar qualification for otherwise suspicious assertion pipes.

use oxc_allocator::Allocator;
use oxc_ast::ast::TSUnionType;
use oxc_ast_visit::{Visit, walk::walk_ts_union_type};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};

pub(super) fn extend_union_spans(source: &str, spans: &mut Vec<(usize, usize)>) {
    let has_assertion = source
        .split(|character: char| {
            !(character.is_alphanumeric() || character == '_' || character == '$')
        })
        .any(|word| matches!(word, "as" | "satisfies"));
    if !has_assertion {
        return;
    }
    let allocator = Allocator::default();
    let Ok(expression) = Parser::new(&allocator, source, SourceType::ts()).parse_expression()
    else {
        // Invalid or unsupported expression syntax keeps the original finding.
        return;
    };
    if !source
        .get(expression.span().end as usize..)
        .is_some_and(|tail| tail.trim().is_empty())
    {
        return;
    }
    UnionSpans { spans }.visit_expression(&expression);
}

struct UnionSpans<'spans> {
    spans: &'spans mut Vec<(usize, usize)>,
}

impl<'a> Visit<'a> for UnionSpans<'_> {
    fn visit_ts_union_type(&mut self, union: &TSUnionType<'a>) {
        self.spans
            .push((union.span.start as usize, union.span.end as usize));
        walk_ts_union_type(self, union);
    }
}

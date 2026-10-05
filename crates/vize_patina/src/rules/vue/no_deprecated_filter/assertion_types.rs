//! Cold type-grammar qualification for otherwise suspicious assertion pipes.

use oxc_allocator::Allocator;
use oxc_ast::ast::TSUnionType;
use oxc_ast_visit::{Visit, walk::walk_ts_union_type};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_l0::String;

pub(super) fn extend_union_spans(source: &str, spans: &mut Vec<(usize, usize)>) {
    let has_assertion = source
        .split(|character: char| {
            !(character.is_alphanumeric() || character == '_' || character == '$')
        })
        .any(|word| matches!(word, "as" | "satisfies"));
    if !has_assertion {
        return;
    }
    // Preserve parentheses and force the entire authored expression to finish.
    // The newline permits an authored trailing line comment before the wrapper.
    let mut wrapped = String::with_capacity(source.len() + 3);
    wrapped.push('(');
    wrapped.push_str(source);
    wrapped.push_str("\n)");
    let allocator = Allocator::default();
    let Ok(expression) = Parser::new(&allocator, &wrapped, SourceType::ts()).parse_expression()
    else {
        return;
    };
    if expression.span().end as usize != wrapped.len() {
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
            .push((union.span.start as usize - 1, union.span.end as usize - 1));
        walk_ts_union_type(self, union);
    }
}

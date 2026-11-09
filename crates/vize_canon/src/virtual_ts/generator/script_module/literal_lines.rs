//! Authored literals that span lines. Their continuation lines are string
//! content, so the setup projection copies them without any rewrite.

use oxc_ast::ast::{Program, StringLiteral, TemplateLiteral};
use oxc_ast_visit::{Visit, walk};

/// Sorted `(start, end)` spans of multi-line template and string literals.
pub(super) fn collect(program: &Program<'_>, script: &str) -> Vec<(u32, u32)> {
    let mut collector = Collector {
        script,
        spans: Vec::new(),
    };
    collector.visit_program(program);
    collector.spans.sort_unstable();
    collector.spans
}

struct Collector<'s> {
    script: &'s str,
    spans: Vec<(u32, u32)>,
}

impl Collector<'_> {
    fn push(&mut self, start: u32, end: u32) {
        if self
            .script
            .get(start as usize..end as usize)
            .is_some_and(|text| text.contains('\n'))
        {
            self.spans.push((start, end));
        }
    }
}

impl<'a> Visit<'a> for Collector<'_> {
    fn visit_template_literal(&mut self, literal: &TemplateLiteral<'a>) {
        self.push(literal.span.start, literal.span.end);
        walk::walk_template_literal(self, literal);
    }

    fn visit_string_literal(&mut self, literal: &StringLiteral<'a>) {
        self.push(literal.span.start, literal.span.end);
    }
}

/// Whether `offset` lies strictly inside one of `spans`.
pub(super) fn inside(spans: &[(u32, u32)], offset: usize) -> bool {
    let after = spans.partition_point(|&(start, _)| (start as usize) < offset);
    after
        .checked_sub(1)
        .and_then(|index| spans.get(index))
        .is_some_and(|&(_, end)| offset < end as usize)
}

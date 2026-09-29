//! Statement ranges for plain TypeScript modules.

use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast::ast::Statement;
use oxc_ast_visit::{Visit, walk::walk_statement};
use oxc_parser::Parser as OxcParser;
use oxc_span::{GetSpan, SourceType};

pub(super) fn statement_spans(source: &str, filename: &str) -> Vec<(u32, u32)> {
    let allocator = OxcAllocator::default();
    let source_type = SourceType::from_path(filename).unwrap_or_default();
    let parsed = OxcParser::new(&allocator, source, source_type).parse();
    if parsed.panicked {
        return Vec::new();
    }
    let mut collector = StatementSpans::default();
    collector.visit_program(&parsed.program);
    collector.spans
}

#[derive(Default)]
struct StatementSpans {
    spans: Vec<(u32, u32)>,
}

impl<'a> Visit<'a> for StatementSpans {
    fn visit_statement(&mut self, statement: &Statement<'a>) {
        let span = statement.span();
        self.spans.push((span.start, span.end));
        walk_statement(self, statement);
    }
}

pub(super) fn enclosing_statement_end(spans: &[(u32, u32)], start: u32, end: u32) -> Option<u32> {
    let mut best_len = u32::MAX;
    let mut best_end = None;
    for &(span_start, span_end) in spans {
        if span_start <= start && end <= span_end {
            let len = span_end.saturating_sub(span_start);
            if len < best_len {
                best_len = len;
                best_end = Some(span_end);
            }
        }
    }
    best_end
}

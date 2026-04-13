use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast::ast::CallExpression;
use oxc_ast_visit::{walk::walk_call_expression, Visit};
use oxc_parser::Parser as OxcParser;
use oxc_span::{GetSpan, SourceType, Span};
use vize_carton::{profile, String};

#[derive(Clone, Copy)]
pub(super) struct RelativeRange {
    pub start: u32,
    pub end: u32,
}

pub(super) fn collect_call_callee_ranges(
    source: &str,
    allow_statement_fallback: bool,
) -> Vec<RelativeRange> {
    let allocator = OxcAllocator::default();
    let source_type = SourceType::from_path("template.ts").unwrap_or_default();
    if let Ok(expression) = profile!(
        "patina.type_aware.template_calls.parse_expression",
        OxcParser::new(&allocator, source, source_type).parse_expression()
    ) {
        let mut collector = CallCalleeCollector::default();
        profile!(
            "patina.type_aware.template_calls.visit_expression",
            collector.visit_expression(&expression)
        );
        return collector.into_relative_ranges(0, source.len() as u32);
    }

    if !allow_statement_fallback {
        return Vec::new();
    }

    const PREFIX: &str = "function __vize_template_handler(){\n";
    let mut wrapped = String::with_capacity(PREFIX.len() + source.len() + 4);
    wrapped.push_str(PREFIX);
    wrapped.push_str(source);
    wrapped.push_str("\n}");

    let parsed = profile!(
        "patina.type_aware.template_calls.parse_statement",
        OxcParser::new(&allocator, wrapped.as_str(), source_type).parse()
    );
    if parsed.panicked || !parsed.errors.is_empty() {
        return Vec::new();
    }

    let mut collector = CallCalleeCollector::default();
    profile!(
        "patina.type_aware.template_calls.visit_program",
        collector.visit_program(&parsed.program)
    );
    collector.into_relative_ranges(PREFIX.len() as u32, source.len() as u32)
}

#[derive(Default)]
struct CallCalleeCollector {
    spans: Vec<Span>,
}

impl CallCalleeCollector {
    fn into_relative_ranges(mut self, base_offset: u32, source_len: u32) -> Vec<RelativeRange> {
        let mut ranges = Vec::with_capacity(self.spans.len());
        let limit = base_offset + source_len;
        self.spans
            .sort_unstable_by_key(|span| (span.start, span.end));
        self.spans
            .dedup_by(|left, right| left.start == right.start && left.end == right.end);
        for span in self.spans {
            if span.end <= span.start || span.start < base_offset || span.end > limit {
                continue;
            }
            ranges.push(RelativeRange {
                start: span.start - base_offset,
                end: span.end - base_offset,
            });
        }
        ranges
    }
}

impl<'a> Visit<'a> for CallCalleeCollector {
    fn visit_call_expression(&mut self, expression: &CallExpression<'a>) {
        self.spans.push(expression.callee.span());
        walk_call_expression(self, expression);
    }
}

use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast::ast::{CallExpression, ChainElement, Expression, Statement};
use oxc_ast_visit::{walk::walk_call_expression, Visit};
use oxc_parser::Parser as OxcParser;
use oxc_span::{GetSpan, SourceType, Span};
use oxc_syntax::operator::UnaryOperator;
use vize_carton::{profile, String};

#[derive(Clone, Copy)]
pub(super) struct RelativeRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Copy)]
pub(super) struct FloatingPromiseRange {
    pub start: u32,
    pub end: u32,
    pub probe_start: u32,
    pub probe_end: u32,
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

pub(super) fn collect_floating_promise_ranges(
    source: &str,
    allow_statement_fallback: bool,
) -> Vec<FloatingPromiseRange> {
    let allocator = OxcAllocator::default();
    let source_type = SourceType::from_path("template.ts").unwrap_or_default();
    if let Ok(expression) = profile!(
        "patina.type_aware.template_floating.parse_expression",
        OxcParser::new(&allocator, source, source_type).parse_expression()
    ) {
        if expression.span().end as usize == source.trim_end().len()
            && !is_explicitly_handled(&expression)
        {
            if let Some(range) = floating_promise_range_for_expression(&expression) {
                return vec![range];
            }
        }
    }

    if !allow_statement_fallback {
        return Vec::new();
    }

    let parsed = profile!(
        "patina.type_aware.template_floating.parse_statement",
        OxcParser::new(&allocator, source, source_type).parse()
    );
    if parsed.panicked || !parsed.errors.is_empty() {
        return Vec::new();
    }

    let mut ranges = Vec::new();
    for statement in &parsed.program.body {
        let Statement::ExpressionStatement(expression_statement) = statement else {
            continue;
        };
        let expression = &expression_statement.expression;
        if is_explicitly_handled(expression) {
            continue;
        }
        let Some(range) = floating_promise_range_for_expression(expression) else {
            continue;
        };
        if range.end <= range.start || range.end as usize > source.len() {
            continue;
        }
        ranges.push(range);
    }
    ranges
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

fn floating_promise_range_for_expression(
    expression: &Expression<'_>,
) -> Option<FloatingPromiseRange> {
    match expression {
        Expression::CallExpression(call) => {
            let span = expression.span();
            let probe = call.callee.span();
            Some(FloatingPromiseRange {
                start: span.start,
                end: span.end,
                probe_start: probe.start,
                probe_end: probe.end,
            })
        }
        Expression::NewExpression(_) => {
            let span = expression.span();
            Some(FloatingPromiseRange {
                start: span.start,
                end: span.end,
                probe_start: span.start,
                probe_end: span.end,
            })
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => {
                let span = expression.span();
                let probe = call.callee.span();
                Some(FloatingPromiseRange {
                    start: span.start,
                    end: span.end,
                    probe_start: probe.start,
                    probe_end: probe.end,
                })
            }
            ChainElement::TSNonNullExpression(non_null) => {
                floating_promise_range_for_expression(&non_null.expression)
            }
            _ => None,
        },
        Expression::ParenthesizedExpression(paren) => {
            floating_promise_range_for_expression(&paren.expression)
        }
        Expression::TSAsExpression(ts_as) => {
            floating_promise_range_for_expression(&ts_as.expression)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            floating_promise_range_for_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            floating_promise_range_for_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn is_explicitly_handled(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::AwaitExpression(_) => true,
        Expression::UnaryExpression(unary) => unary.operator == UnaryOperator::Void,
        Expression::ParenthesizedExpression(paren) => is_explicitly_handled(&paren.expression),
        Expression::TSAsExpression(ts_as) => is_explicitly_handled(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            is_explicitly_handled(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            is_explicitly_handled(&ts_non_null.expression)
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => is_handled_call(call),
            ChainElement::TSNonNullExpression(non_null) => {
                is_explicitly_handled(&non_null.expression)
            }
            _ => false,
        },
        Expression::CallExpression(call) => is_handled_call(call),
        _ => false,
    }
}

fn is_handled_call(call: &CallExpression<'_>) -> bool {
    call.callee
        .as_member_expression()
        .and_then(|member| member.static_property_name())
        .is_some_and(|name| matches!(name, "then" | "catch" | "finally"))
}

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

#[derive(Default)]
pub(super) struct TemplateCallRanges {
    pub callees: Vec<RelativeRange>,
    pub floating_promises: Vec<FloatingPromiseRange>,
}

pub(super) fn collect_template_call_ranges(
    source: &str,
    allow_statement_fallback: bool,
    include_callees: bool,
    include_floating_promises: bool,
) -> TemplateCallRanges {
    let mut ranges = TemplateCallRanges::default();
    if !include_callees && !include_floating_promises {
        return ranges;
    }

    let allocator = OxcAllocator::default();
    let source_type = SourceType::from_path("template.ts").unwrap_or_default();
    if let Ok(expression) = profile!(
        "patina.type_aware.template_queries.parse_expression",
        OxcParser::new(&allocator, source, source_type).parse_expression()
    ) {
        if include_callees {
            let mut collector = CallCalleeCollector::default();
            profile!(
                "patina.type_aware.template_calls.visit_expression",
                collector.visit_expression(&expression)
            );
            ranges.callees = collector.into_relative_ranges(0, source.len() as u32);
        }
        if include_floating_promises {
            if expression.span().end as usize == source.trim_end().len() {
                profile!(
                    "patina.type_aware.template_floating.visit_expression",
                    collect_floating_promise_ranges_for_expression(
                        &expression,
                        &mut ranges.floating_promises
                    )
                );
            } else if allow_statement_fallback {
                ranges.floating_promises =
                    collect_statement_floating_promise_ranges(&allocator, source_type, source);
            }
        }
        return ranges;
    }

    if !allow_statement_fallback {
        return ranges;
    }

    if include_callees {
        ranges.callees = collect_statement_call_callee_ranges(&allocator, source_type, source);
    }
    if include_floating_promises {
        ranges.floating_promises =
            collect_statement_floating_promise_ranges(&allocator, source_type, source);
    }

    ranges
}

fn collect_statement_call_callee_ranges(
    allocator: &OxcAllocator,
    source_type: SourceType,
    source: &str,
) -> Vec<RelativeRange> {
    const PREFIX: &str = "function __vize_template_handler(){\n";
    let mut wrapped = String::with_capacity(PREFIX.len() + source.len() + 4);
    wrapped.push_str(PREFIX);
    wrapped.push_str(source);
    wrapped.push_str("\n}");

    let parsed = profile!(
        "patina.type_aware.template_calls.parse_statement",
        OxcParser::new(allocator, wrapped.as_str(), source_type).parse()
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

fn collect_statement_floating_promise_ranges(
    allocator: &OxcAllocator,
    source_type: SourceType,
    source: &str,
) -> Vec<FloatingPromiseRange> {
    let parsed = profile!(
        "patina.type_aware.template_floating.parse_statement",
        OxcParser::new(allocator, source, source_type).parse()
    );
    if parsed.panicked || !parsed.errors.is_empty() {
        return Vec::new();
    }

    let mut ranges = Vec::new();
    for statement in &parsed.program.body {
        let Statement::ExpressionStatement(expression_statement) = statement else {
            continue;
        };
        let mut statement_ranges = Vec::new();
        collect_floating_promise_ranges_for_expression(
            &expression_statement.expression,
            &mut statement_ranges,
        );
        for range in statement_ranges {
            if range.end <= range.start || range.end as usize > source.len() {
                continue;
            }
            ranges.push(range);
        }
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

fn collect_floating_promise_ranges_for_expression(
    expression: &Expression<'_>,
    ranges: &mut Vec<FloatingPromiseRange>,
) {
    if is_explicitly_handled(expression) {
        return;
    }
    if let Some(range) = floating_promise_range_for_expression(expression) {
        ranges.push(range);
        return;
    }

    match expression {
        Expression::LogicalExpression(logical) => {
            collect_floating_promise_ranges_for_expression(&logical.left, ranges);
            collect_floating_promise_ranges_for_expression(&logical.right, ranges);
        }
        Expression::ConditionalExpression(conditional) => {
            collect_floating_promise_ranges_for_expression(&conditional.test, ranges);
            collect_floating_promise_ranges_for_expression(&conditional.consequent, ranges);
            collect_floating_promise_ranges_for_expression(&conditional.alternate, ranges);
        }
        Expression::SequenceExpression(sequence) => {
            for expression in &sequence.expressions {
                collect_floating_promise_ranges_for_expression(expression, ranges);
            }
        }
        _ => {}
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
    let Some(member) = call.callee.as_member_expression() else {
        return false;
    };

    match member.static_property_name() {
        Some("then" | "catch") => true,
        Some("finally") => is_handled_promise_chain(member.object()),
        _ => false,
    }
}

fn is_handled_promise_chain(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::ParenthesizedExpression(paren) => is_handled_promise_chain(&paren.expression),
        Expression::TSAsExpression(ts_as) => is_handled_promise_chain(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            is_handled_promise_chain(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            is_handled_promise_chain(&ts_non_null.expression)
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => is_handled_call(call),
            ChainElement::TSNonNullExpression(non_null) => {
                is_handled_promise_chain(&non_null.expression)
            }
            _ => false,
        },
        Expression::CallExpression(call) => is_handled_call(call),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{collect_template_call_ranges, FloatingPromiseRange, RelativeRange};

    fn range_slices<'a>(source: &'a str, ranges: &[RelativeRange]) -> Vec<&'a str> {
        ranges
            .iter()
            .map(|range| &source[range.start as usize..range.end as usize])
            .collect()
    }

    fn promise_slices<'a>(source: &'a str, ranges: &[FloatingPromiseRange]) -> Vec<&'a str> {
        ranges
            .iter()
            .map(|range| &source[range.start as usize..range.end as usize])
            .collect()
    }

    #[test]
    fn collects_callees_and_floating_promises_from_one_expression_parse() {
        let source = "enabled && save()";
        let ranges = collect_template_call_ranges(source, false, true, true);

        assert_eq!(range_slices(source, &ranges.callees), vec!["save"]);
        assert_eq!(
            promise_slices(source, &ranges.floating_promises),
            vec!["save()"]
        );
    }

    #[test]
    fn collects_statement_fallback_callees_for_event_handlers() {
        let source = "if (enabled) { save(); track() }";
        let ranges = collect_template_call_ranges(source, true, true, false);

        assert_eq!(range_slices(source, &ranges.callees), vec!["save", "track"]);
        assert!(ranges.floating_promises.is_empty());
    }

    #[test]
    fn collects_statement_fallback_floating_promises() {
        let source = "save(); track()";
        let ranges = collect_template_call_ranges(source, true, false, true);

        assert!(ranges.callees.is_empty());
        assert_eq!(
            promise_slices(source, &ranges.floating_promises),
            vec!["save()", "track()"]
        );
    }
}

use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast::ast::{Argument, CallExpression, ChainElement, Expression, ExpressionStatement};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_expression_statement},
};
use oxc_parser::Parser as OxcParser;
use oxc_span::{GetSpan, SourceType, Span};
use oxc_syntax::operator::UnaryOperator;
use vize_carton::{String, profile};

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
    pub probe_target: FloatingPromiseProbeTarget,
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum FloatingPromiseProbeTarget {
    SourceText,
    ExpressionBinding,
}

#[derive(Default)]
pub(super) struct TemplateCallRanges {
    pub callees: Vec<RelativeRange>,
    pub floating_promises: Vec<FloatingPromiseRange>,
    pub probe_expression_binding: bool,
    pub expression_consumes_source: bool,
}

const TEMPLATE_HANDLER_PREFIX: &str = "function __vize_template_handler(){\n";

pub(super) fn collect_template_call_ranges(
    allocator: &OxcAllocator,
    source_type: SourceType,
    source: &str,
    allow_statement_fallback: bool,
    include_callees: bool,
    include_floating_promises: bool,
) -> TemplateCallRanges {
    // Parse the template expression once and derive every range family from the
    // same AST. Event handlers can contain statement bodies, so only those callers
    // enable the statement fallback; ordinary bindings stay on the cheaper
    // expression parse path. The arena and source type are owned by the caller and
    // reused across every template expression in the file; each call copies all
    // needed data into owned ranges before returning, so the caller may reset the
    // arena afterwards without invalidating the result.
    let mut ranges = TemplateCallRanges::default();
    if !include_callees && !include_floating_promises {
        return ranges;
    }

    if let Ok(expression) = profile!(
        "patina.type_aware.template_queries.parse_expression",
        OxcParser::new(allocator, source, source_type).parse_expression()
    ) {
        ranges.probe_expression_binding = expression_prefers_binding_probe(&expression);
        let expression_consumes_source = expression.span().end as usize == source.trim_end().len();
        ranges.expression_consumes_source = expression_consumes_source;
        if allow_statement_fallback && !expression_consumes_source {
            let statement_ranges = collect_statement_template_call_ranges(
                allocator,
                source_type,
                source,
                include_callees,
                include_floating_promises,
            );
            if include_callees {
                ranges.callees = if statement_ranges.callees.is_empty() {
                    collect_expression_call_callee_ranges(&expression, source.len() as u32)
                } else {
                    statement_ranges.callees
                };
            }
            if include_floating_promises {
                ranges.floating_promises = statement_ranges.floating_promises;
            }

            return ranges;
        }

        if include_callees {
            ranges.callees =
                collect_expression_call_callee_ranges(&expression, source.len() as u32);
        }
        if include_floating_promises {
            if allow_statement_fallback
                && let Some(range) = bare_handler_reference_range_for_expression(&expression)
            {
                ranges.floating_promises.push(range);
            }
            profile!(
                "patina.type_aware.template_floating.visit_expression",
                collect_floating_promise_ranges_for_expression(
                    &expression,
                    &mut ranges.floating_promises
                )
            );
        }
        return ranges;
    }

    if !allow_statement_fallback {
        return ranges;
    }

    collect_statement_template_call_ranges(
        allocator,
        source_type,
        source,
        include_callees,
        include_floating_promises,
    )
}

fn expression_prefers_binding_probe(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::ComputedMemberExpression(_) => true,
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::ComputedMemberExpression(_) => true,
            ChainElement::TSNonNullExpression(non_null) => {
                expression_prefers_binding_probe(&non_null.expression)
            }
            _ => false,
        },
        Expression::ParenthesizedExpression(paren) => {
            expression_prefers_binding_probe(&paren.expression)
        }
        Expression::TSAsExpression(ts_as) => expression_prefers_binding_probe(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            expression_prefers_binding_probe(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            expression_prefers_binding_probe(&ts_non_null.expression)
        }
        _ => false,
    }
}

fn bare_handler_reference_range_for_expression(
    expression: &Expression<'_>,
) -> Option<FloatingPromiseRange> {
    match expression {
        Expression::Identifier(_) => Some(floating_promise_range_from_span(expression.span())),
        Expression::StaticMemberExpression(member) => is_member_handler_reference(&member.object)
            .then(|| floating_promise_range_from_span(expression.span())),
        Expression::ComputedMemberExpression(member) => is_member_handler_reference(&member.object)
            .then(|| {
                floating_promise_range_from_span_with_target(
                    expression.span(),
                    FloatingPromiseProbeTarget::ExpressionBinding,
                )
            }),
        Expression::ChainExpression(chain) => {
            chain_member_handler_reference_range(chain, expression.span())
        }
        Expression::ParenthesizedExpression(paren) => {
            bare_handler_reference_range_for_expression(&paren.expression)
        }
        Expression::TSAsExpression(ts_as) => {
            bare_handler_reference_range_for_expression(&ts_as.expression)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            bare_handler_reference_range_for_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            bare_handler_reference_range_for_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn chain_member_handler_reference_range(
    chain: &oxc_ast::ast::ChainExpression<'_>,
    span: Span,
) -> Option<FloatingPromiseRange> {
    match &chain.expression {
        ChainElement::StaticMemberExpression(member) => is_member_handler_reference(&member.object)
            .then(|| floating_promise_range_from_span(span)),
        ChainElement::ComputedMemberExpression(member) => {
            is_member_handler_reference(&member.object).then(|| {
                floating_promise_range_from_span_with_target(
                    span,
                    FloatingPromiseProbeTarget::ExpressionBinding,
                )
            })
        }
        ChainElement::TSNonNullExpression(non_null) => {
            bare_handler_reference_range_for_expression(&non_null.expression)
        }
        _ => None,
    }
}

fn is_member_handler_reference(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::Identifier(_) | Expression::ThisExpression(_) => true,
        Expression::StaticMemberExpression(member) => is_member_handler_reference(&member.object),
        Expression::ComputedMemberExpression(member) => is_member_handler_reference(&member.object),
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::StaticMemberExpression(member) => {
                is_member_handler_reference(&member.object)
            }
            ChainElement::ComputedMemberExpression(member) => {
                is_member_handler_reference(&member.object)
            }
            ChainElement::TSNonNullExpression(non_null) => {
                is_member_handler_reference(&non_null.expression)
            }
            _ => false,
        },
        _ => false,
    }
}

fn floating_promise_range_from_span(span: Span) -> FloatingPromiseRange {
    floating_promise_range_from_span_with_target(span, FloatingPromiseProbeTarget::SourceText)
}

fn floating_promise_range_from_span_with_target(
    span: Span,
    probe_target: FloatingPromiseProbeTarget,
) -> FloatingPromiseRange {
    FloatingPromiseRange {
        start: span.start,
        end: span.end,
        probe_start: span.start,
        probe_end: span.end,
        probe_target,
    }
}

fn collect_expression_call_callee_ranges(
    expression: &Expression<'_>,
    source_len: u32,
) -> Vec<RelativeRange> {
    let mut collector = CallCalleeCollector::default();
    profile!(
        "patina.type_aware.template_calls.visit_expression",
        collector.visit_expression(expression)
    );
    collector.into_relative_ranges(0, source_len)
}

fn collect_statement_template_call_ranges(
    allocator: &OxcAllocator,
    source_type: SourceType,
    source: &str,
    include_callees: bool,
    include_floating_promises: bool,
) -> TemplateCallRanges {
    let mut wrapped = String::with_capacity(TEMPLATE_HANDLER_PREFIX.len() + source.len() + 4);
    wrapped.push_str(TEMPLATE_HANDLER_PREFIX);
    wrapped.push_str(source);
    wrapped.push_str("\n}");

    let parsed = profile!(
        "patina.type_aware.template_calls.parse_statement",
        OxcParser::new(allocator, wrapped.as_str(), source_type).parse()
    );
    if parsed.panicked || !parsed.errors.is_empty() {
        return TemplateCallRanges::default();
    }

    let mut ranges = TemplateCallRanges::default();
    if include_callees {
        let mut collector = CallCalleeCollector::default();
        profile!(
            "patina.type_aware.template_calls.visit_program",
            collector.visit_program(&parsed.program)
        );
        ranges.callees = collector
            .into_relative_ranges(TEMPLATE_HANDLER_PREFIX.len() as u32, source.len() as u32);
    }
    if include_floating_promises {
        let mut collector = FloatingPromiseCollector::default();
        profile!(
            "patina.type_aware.template_floating.visit_program",
            collector.visit_program(&parsed.program)
        );
        ranges.floating_promises = collector
            .into_relative_ranges(TEMPLATE_HANDLER_PREFIX.len() as u32, source.len() as u32);
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

#[derive(Default)]
struct FloatingPromiseCollector {
    ranges: Vec<FloatingPromiseRange>,
}

impl FloatingPromiseCollector {
    fn into_relative_ranges(
        mut self,
        base_offset: u32,
        source_len: u32,
    ) -> Vec<FloatingPromiseRange> {
        let mut ranges = Vec::with_capacity(self.ranges.len());
        let limit = base_offset + source_len;
        self.ranges.sort_unstable_by_key(|range| {
            (
                range.start,
                range.end,
                range.probe_start,
                range.probe_target,
            )
        });
        self.ranges.dedup_by(|left, right| {
            left.start == right.start
                && left.end == right.end
                && left.probe_start == right.probe_start
                && left.probe_end == right.probe_end
                && left.probe_target == right.probe_target
        });
        for range in self.ranges {
            if range.end <= range.start
                || range.start < base_offset
                || range.end > limit
                || range.probe_end <= range.probe_start
                || range.probe_start < base_offset
                || range.probe_end > limit
            {
                continue;
            }
            ranges.push(FloatingPromiseRange {
                start: range.start - base_offset,
                end: range.end - base_offset,
                probe_start: range.probe_start - base_offset,
                probe_end: range.probe_end - base_offset,
                probe_target: range.probe_target,
            });
        }
        ranges
    }
}

impl<'a> Visit<'a> for FloatingPromiseCollector {
    fn visit_expression_statement(&mut self, statement: &ExpressionStatement<'a>) {
        collect_floating_promise_ranges_for_expression(&statement.expression, &mut self.ranges);
        walk_expression_statement(self, statement);
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
                probe_target: FloatingPromiseProbeTarget::SourceText,
            })
        }
        Expression::NewExpression(_) => {
            let span = expression.span();
            Some(FloatingPromiseRange {
                start: span.start,
                end: span.end,
                probe_start: span.start,
                probe_end: span.end,
                probe_target: FloatingPromiseProbeTarget::SourceText,
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
                    probe_target: FloatingPromiseProbeTarget::SourceText,
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
        Some("then") => has_present_handler_argument(call, 1),
        Some("catch") => has_present_handler_argument(call, 0),
        Some("finally") => is_handled_promise_chain(member.object()),
        _ => false,
    }
}

fn has_present_handler_argument(call: &CallExpression<'_>, index: usize) -> bool {
    call.arguments
        .get(index)
        .is_some_and(|argument| !is_missing_handler_argument(argument))
}

fn is_missing_handler_argument(argument: &Argument<'_>) -> bool {
    match argument {
        Argument::Identifier(identifier) => identifier.name == "undefined",
        Argument::NullLiteral(_) => true,
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
    use super::{
        FloatingPromiseRange, OxcAllocator, RelativeRange, SourceType, TemplateCallRanges,
    };

    fn collect_template_call_ranges(
        source: &str,
        allow_statement_fallback: bool,
        include_callees: bool,
        include_floating_promises: bool,
    ) -> TemplateCallRanges {
        let allocator = OxcAllocator::default();
        let source_type = SourceType::from_path("template.ts").unwrap_or_default();
        super::collect_template_call_ranges(
            &allocator,
            source_type,
            source,
            allow_statement_fallback,
            include_callees,
            include_floating_promises,
        )
    }

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
    fn collects_statement_fallback_callees_after_expression_prefix() {
        let source = "safe(); anyHandler()";
        let ranges = collect_template_call_ranges(source, true, true, false);

        assert_eq!(
            range_slices(source, &ranges.callees),
            vec!["safe", "anyHandler"]
        );
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

    #[test]
    fn collects_statement_fallback_floating_promises_inside_control_flow() {
        let source = "if (enabled) { save(); track() }";
        let ranges = collect_template_call_ranges(source, true, false, true);

        assert!(ranges.callees.is_empty());
        assert_eq!(
            promise_slices(source, &ranges.floating_promises),
            vec!["save()", "track()"]
        );
    }

    #[test]
    fn collects_bare_event_handler_references_as_floating_candidates() {
        let source = "save";
        let ranges = collect_template_call_ranges(source, true, false, true);

        assert_eq!(
            promise_slices(source, &ranges.floating_promises),
            vec!["save"]
        );
    }

    #[test]
    fn collects_member_event_handler_references_as_floating_candidates() {
        let source = "actions.save";
        let ranges = collect_template_call_ranges(source, true, false, true);

        assert_eq!(
            promise_slices(source, &ranges.floating_promises),
            vec!["actions.save"]
        );
    }

    #[test]
    fn collects_optional_member_event_handler_references_as_floating_candidates() {
        let source = "actions?.save";
        let ranges = collect_template_call_ranges(source, true, false, true);

        assert_eq!(
            promise_slices(source, &ranges.floating_promises),
            vec!["actions?.save"]
        );
    }

    #[test]
    fn collects_computed_member_event_handler_references_as_floating_candidates() {
        let source = "actions[method]";
        let ranges = collect_template_call_ranges(source, true, false, true);

        assert_eq!(
            promise_slices(source, &ranges.floating_promises),
            vec!["actions[method]"]
        );
    }

    #[test]
    fn collects_optional_computed_member_event_handler_references_as_floating_candidates() {
        let source = "actions?.[method]";
        let ranges = collect_template_call_ranges(source, true, false, true);

        assert_eq!(
            promise_slices(source, &ranges.floating_promises),
            vec!["actions?.[method]"]
        );
    }

    #[test]
    fn computed_member_expressions_prefer_binding_probes() {
        let computed = collect_template_call_ranges("payload[key]", false, true, false);
        let optional = collect_template_call_ranges("payload?.[key]", false, true, false);
        let call = collect_template_call_ranges("payload[key]()", false, true, false);
        let static_member = collect_template_call_ranges("payload.key", false, true, false);

        assert!(computed.probe_expression_binding);
        assert!(optional.probe_expression_binding);
        assert!(!call.probe_expression_binding);
        assert!(!static_member.probe_expression_binding);
    }

    #[test]
    fn ignores_bare_references_without_event_fallback() {
        let source = "save";
        let ranges = collect_template_call_ranges(source, false, false, true);

        assert!(ranges.floating_promises.is_empty());
    }

    #[test]
    fn reports_then_without_rejection_handler_as_floating() {
        let source = "save().then(() => {})";
        let ranges = collect_template_call_ranges(source, false, false, true);

        assert_eq!(
            promise_slices(source, &ranges.floating_promises),
            vec!["save().then(() => {})"]
        );
    }

    #[test]
    fn ignores_then_with_rejection_handler() {
        let source = "save().then(() => {}, report)";
        let ranges = collect_template_call_ranges(source, false, false, true);

        assert!(ranges.floating_promises.is_empty());
    }

    #[test]
    fn reports_empty_catch_as_floating() {
        let source = "save().catch()";
        let ranges = collect_template_call_ranges(source, false, false, true);

        assert_eq!(
            promise_slices(source, &ranges.floating_promises),
            vec!["save().catch()"]
        );
    }
}

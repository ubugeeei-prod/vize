use super::super::document::TypeAwareDocument;
use super::{
    TemplateContext, TemplatePromiseQuery, TemplateQuery, TemplateQueryKind,
    absolute_expression_range,
    calls::{FloatingPromiseProbeTarget, collect_template_call_ranges},
    generated_offset_for_text, v_for_source_binding_offset,
};
use oxc_allocator::Allocator as OxcAllocator;
use oxc_span::SourceType;
use vize_relief::{
    DirectiveNode, ExpressionNode, ForNode, IfNode, PropNode, RootNode, TemplateChildNode,
    TextCallContent,
};
use vize_s0::profile;

pub(super) fn collect_template_query_sets(
    virtual_ts: &TypeAwareDocument,
    template_ast: &RootNode<'_>,
    template_offset: u32,
    include_template_queries: bool,
    include_template_promise_queries: bool,
) -> (Vec<TemplateQuery>, Vec<TemplatePromiseQuery>) {
    // One traversal populates both query vectors. `TemplateQuerySinks` keeps
    // disabled rules out of the hot path, preserves the source-order discovery
    // the sort/dedup stages rely on, and carries the span-slicing source text.
    let mut template_queries = Vec::new();
    let mut template_promise_queries = Vec::new();
    let mut sinks = TemplateQuerySinks {
        source: template_ast.source,
        template_queries: include_template_queries.then_some(&mut template_queries),
        template_promise_queries: include_template_promise_queries
            .then_some(&mut template_promise_queries),
    };

    // Build the parsing arena and source type once per file; every template
    // expression reuses both (the allocator is reset between expressions).
    let mut allocator = OxcAllocator::default();
    let source_type = SourceType::from_path("template.ts").unwrap_or_default();

    profile!(
        "patina.type_aware.template_query_sets.walk",
        collect_children(
            virtual_ts,
            &template_ast.children,
            template_offset,
            &mut allocator,
            source_type,
            &mut sinks,
        )
    );
    if include_template_queries {
        profile!(
            "patina.type_aware.template_queries.dedupe",
            dedupe_template_queries(&mut template_queries)
        );
    }
    if include_template_promise_queries {
        profile!(
            "patina.type_aware.template_promise_queries.dedupe",
            dedupe_template_promise_queries(&mut template_promise_queries)
        );
    }

    (template_queries, template_promise_queries)
}

struct TemplateQuerySinks<'a> {
    source: &'a str,
    template_queries: Option<&'a mut Vec<TemplateQuery>>,
    template_promise_queries: Option<&'a mut Vec<TemplatePromiseQuery>>,
}

impl TemplateQuerySinks<'_> {
    fn has_queries(&self) -> bool {
        self.template_queries.is_some() || self.template_promise_queries.is_some()
    }
}

fn dedupe_template_queries(queries: &mut Vec<TemplateQuery>) {
    queries.sort_unstable_by_key(|query| {
        (
            query.owner_start,
            query.owner_end,
            query.kind,
            query.context,
            query.source_start,
            query.source_end,
        )
    });
    queries.dedup_by(|left, right| {
        left.kind == right.kind
            && left.context == right.context
            && left.source_start == right.source_start
            && left.source_end == right.source_end
            && left.owner_start == right.owner_start
            && left.owner_end == right.owner_end
    });
}

fn dedupe_template_promise_queries(queries: &mut Vec<TemplatePromiseQuery>) {
    queries.sort_unstable_by_key(|query| {
        (
            query.context,
            query.source_start,
            query.source_end,
            query.generated_offset,
        )
    });
    queries.dedup_by(|left, right| {
        left.context == right.context
            && left.source_start == right.source_start
            && left.source_end == right.source_end
    });
}

fn collect_children(
    virtual_ts: &TypeAwareDocument,
    children: &[TemplateChildNode<'_>],
    template_offset: u32,
    allocator: &mut OxcAllocator,
    source_type: SourceType,
    sinks: &mut TemplateQuerySinks<'_>,
) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                for prop in &element.props {
                    let PropNode::Directive(directive) = prop else {
                        continue;
                    };
                    if sinks.has_queries() {
                        profile!(
                            "patina.type_aware.template_query_sets.directive",
                            collect_directive(
                                virtual_ts,
                                directive,
                                template_offset,
                                allocator,
                                source_type,
                                sinks,
                            )
                        );
                    }
                }
                profile!(
                    "patina.type_aware.template_query_sets.children",
                    collect_children(
                        virtual_ts,
                        &element.children,
                        template_offset,
                        allocator,
                        source_type,
                        sinks,
                    )
                );
            }
            TemplateChildNode::Interpolation(interpolation) => {
                collect_expression(
                    virtual_ts,
                    &interpolation.content,
                    template_offset,
                    TemplateContext::Interpolation,
                    false,
                    allocator,
                    source_type,
                    sinks,
                );
            }
            TemplateChildNode::If(if_node) => {
                profile!(
                    "patina.type_aware.template_query_sets.if",
                    collect_if(
                        virtual_ts,
                        if_node,
                        template_offset,
                        allocator,
                        source_type,
                        sinks
                    )
                )
            }
            TemplateChildNode::IfBranch(branch) => {
                if let Some(condition) = &branch.condition {
                    collect_expression(
                        virtual_ts,
                        condition,
                        template_offset,
                        TemplateContext::Directive,
                        false,
                        allocator,
                        source_type,
                        sinks,
                    );
                }
                profile!(
                    "patina.type_aware.template_query_sets.children",
                    collect_children(
                        virtual_ts,
                        &branch.children,
                        template_offset,
                        allocator,
                        source_type,
                        sinks,
                    )
                );
            }
            TemplateChildNode::For(for_node) => {
                profile!(
                    "patina.type_aware.template_query_sets.for",
                    collect_for(
                        virtual_ts,
                        for_node,
                        template_offset,
                        allocator,
                        source_type,
                        sinks
                    )
                )
            }
            TemplateChildNode::TextCall(text_call) => {
                if let TextCallContent::Interpolation(interpolation) = &text_call.content {
                    collect_expression(
                        virtual_ts,
                        &interpolation.content,
                        template_offset,
                        TemplateContext::Interpolation,
                        false,
                        allocator,
                        source_type,
                        sinks,
                    );
                }
            }
            _ => {}
        }
    }
}

fn collect_if(
    virtual_ts: &TypeAwareDocument,
    if_node: &IfNode<'_>,
    template_offset: u32,
    allocator: &mut OxcAllocator,
    source_type: SourceType,
    sinks: &mut TemplateQuerySinks<'_>,
) {
    for branch in &if_node.branches {
        if let Some(condition) = &branch.condition {
            collect_expression(
                virtual_ts,
                condition,
                template_offset,
                TemplateContext::Directive,
                false,
                allocator,
                source_type,
                sinks,
            );
        }
        profile!(
            "patina.type_aware.template_query_sets.children",
            collect_children(
                virtual_ts,
                &branch.children,
                template_offset,
                allocator,
                source_type,
                sinks,
            )
        );
    }
}

fn collect_for(
    virtual_ts: &TypeAwareDocument,
    for_node: &ForNode<'_>,
    template_offset: u32,
    allocator: &mut OxcAllocator,
    source_type: SourceType,
    sinks: &mut TemplateQuerySinks<'_>,
) {
    let first_query = sinks
        .template_queries
        .as_ref()
        .map_or(0, |queries| queries.len());
    collect_expression(
        virtual_ts,
        &for_node.source,
        template_offset,
        TemplateContext::Directive,
        false,
        allocator,
        source_type,
        sinks,
    );
    retarget_v_for_source_queries(virtual_ts, sinks, first_query);
    profile!(
        "patina.type_aware.template_query_sets.children",
        collect_children(
            virtual_ts,
            &for_node.children,
            template_offset,
            allocator,
            source_type,
            sinks,
        )
    );
}

#[expect(clippy::too_many_arguments, reason = "explicit per-block state")]
fn collect_expression(
    virtual_ts: &TypeAwareDocument,
    expression: &ExpressionNode<'_>,
    template_offset: u32,
    context: TemplateContext,
    allow_statement_fallback: bool,
    allocator: &mut OxcAllocator,
    source_type: SourceType,
    sinks: &mut TemplateQuerySinks<'_>,
) {
    let include_template_queries = sinks.template_queries.is_some();
    let include_template_promise_queries = sinks.template_promise_queries.is_some();
    if include_template_queries || include_template_promise_queries {
        profile!(
            "patina.type_aware.template_query_sets.expression",
            collect_expression_query_sets(
                virtual_ts,
                expression,
                template_offset,
                context,
                allow_statement_fallback,
                allocator,
                source_type,
                sinks,
            )
        );
    }
}

fn collect_directive(
    virtual_ts: &TypeAwareDocument,
    directive: &DirectiveNode<'_>,
    template_offset: u32,
    allocator: &mut OxcAllocator,
    source_type: SourceType,
    sinks: &mut TemplateQuerySinks<'_>,
) {
    let Some(expression) = &directive.exp else {
        return;
    };
    let context = match directive.name {
        "bind" => TemplateContext::Binding,
        "on" => TemplateContext::Event,
        _ => TemplateContext::Directive,
    };
    let first_query = sinks
        .template_queries
        .as_ref()
        .map_or(0, |queries| queries.len());
    collect_expression(
        virtual_ts,
        expression,
        template_offset,
        context,
        matches!(context, TemplateContext::Event),
        allocator,
        source_type,
        sinks,
    );
    if directive.name == "for" {
        retarget_v_for_source_queries(virtual_ts, sinks, first_query);
    } else if directive.name == "bind" && directive.arg.is_none() {
        retarget_slot_spread_call_queries(
            virtual_ts,
            expression.loc().span.slice(sinks.source),
            sinks,
            first_query,
        );
    }
}

/// A slot outlet's unargumented `v-bind` is wrapped in a generated spread
/// helper. The closing parenthesis of its authored call can resolve to that
/// helper's `any`, so query the root callee's return type instead.
fn retarget_slot_spread_call_queries(
    virtual_ts: &TypeAwareDocument,
    expression_source: &str,
    sinks: &mut TemplateQuerySinks<'_>,
    first_query: usize,
) {
    if let Some(queries) = sinks.template_queries.as_deref_mut() {
        let Some(recent) = queries.get_mut(first_query..) else {
            return;
        };
        let Some((expression_index, expression)) = recent
            .iter()
            .enumerate()
            .find(|(_, query)| query.kind == TemplateQueryKind::Expression)
        else {
            return;
        };
        let expression_start = expression.source_start;
        let expression_end = expression.source_end;
        let expression_offset = expression.generated_offset;
        let Some((callee_index, callee)) = recent.iter().enumerate().find(|(_, query)| {
            query.kind == TemplateQueryKind::CallCallee
                && query.source_start == expression_start
                && query.source_end < expression_end
        }) else {
            return;
        };
        let callee_len = (callee.source_end - expression_start) as usize;
        let Some(callee_text) = expression_source.get(..callee_len) else {
            return;
        };
        let Some(callee_offset) =
            slot_spread_callee_offset(&virtual_ts.content, expression_offset, callee_text)
        else {
            return;
        };
        if let Some(callee) = recent.get_mut(callee_index) {
            callee.generated_offset = callee_offset;
        }
        if let Some(expression) = recent.get_mut(expression_index) {
            expression.generated_offset = callee_offset;
            expression.kind = TemplateQueryKind::CallReturn;
        }
    }
}

fn slot_spread_callee_offset(
    generated: &str,
    expression_offset: u32,
    callee_text: &str,
) -> Option<u32> {
    let at = expression_offset as usize;
    let line_start = generated
        .get(..at)?
        .rfind('\n')
        .map_or(0, |index| index + 1);
    let line_end = generated
        .get(at..)?
        .find('\n')
        .map_or(generated.len(), |index| at + index);
    let line = generated.get(line_start..line_end)?;
    if !line.contains("__vizeSlotOutletSpread<") {
        return None;
    }
    let before_expression = line.get(..at - line_start)?;
    let callee_start = before_expression.rfind(callee_text)?;
    let after_callee = line.get(callee_start + callee_text.len()..)?;
    if !after_callee.starts_with('(') && !after_callee.starts_with("?.(") {
        return None;
    }
    u32::try_from(line_start + callee_start + callee_text.len() - 1).ok()
}

fn retarget_v_for_source_queries(
    virtual_ts: &TypeAwareDocument,
    sinks: &mut TemplateQuerySinks<'_>,
    first_query: usize,
) {
    if let Some(queries) = sinks.template_queries.as_deref_mut() {
        for query in queries.iter_mut().skip(first_query) {
            if query.kind == TemplateQueryKind::Expression
                && let Some(offset) =
                    v_for_source_binding_offset(&virtual_ts.content, query.generated_offset)
            {
                query.generated_offset = offset;
                query.kind = TemplateQueryKind::ForSource;
            }
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "explicit per-block state")]
fn collect_expression_query_sets(
    virtual_ts: &TypeAwareDocument,
    expression: &ExpressionNode<'_>,
    template_offset: u32,
    context: TemplateContext,
    allow_statement_fallback: bool,
    allocator: &mut OxcAllocator,
    source_type: SourceType,
    sinks: &mut TemplateQuerySinks<'_>,
) {
    let Some((source_start, source_end)) = absolute_expression_range(expression, template_offset)
    else {
        return;
    };
    let source_text = expression.loc().span.slice(sinks.source);
    let include_template_queries = sinks.template_queries.is_some();
    let include_template_promise_queries = sinks.template_promise_queries.is_some();
    // Resolve the full expression's generated offset once, then reuse the parsed
    // call-range result for both callee probes and floating-Promise probes. This
    // is the key optimization from the type-aware perf work: template expressions
    // are often short but numerous, so reparsing per rule dominated Corsa time.
    let expression_generated_offset = include_template_queries
        .then(|| generated_offset_for_text(virtual_ts, source_start, source_text))
        .flatten();
    let include_call_callees = expression_generated_offset.is_some();
    let call_ranges = profile!(
        "patina.type_aware.template_query_sets.call_ranges",
        collect_template_call_ranges(
            allocator,
            source_type,
            source_text,
            allow_statement_fallback,
            include_call_callees,
            include_template_promise_queries,
        )
    );
    // `call_ranges` owns every range (plain integers / bools) and borrows nothing
    // from the arena, so the arena can be recycled before the ranges are consumed.
    // Resetting here reuses the parse memory for the next template expression.
    allocator.reset();

    if let Some(queries) = sinks.template_queries.as_deref_mut() {
        if call_ranges.expression_consumes_source
            && let Some(generated_offset) = expression_generated_offset
        {
            let generated_offset =
                expression_binding_generated_offset(&virtual_ts.content, generated_offset)
                    .unwrap_or(generated_offset);
            queries.push(TemplateQuery {
                kind: TemplateQueryKind::Expression,
                context,
                generated_offset,
                source_start,
                source_end,
                owner_start: source_start,
                owner_end: source_end,
            });
        }
        for callee in call_ranges.callees {
            let callee_start = source_start + callee.start;
            let callee_end = source_start + callee.end;
            let Some(callee_source) = source_text.get(callee.start as usize..callee.end as usize)
            else {
                continue;
            };
            let Some(generated_offset) =
                generated_offset_for_text(virtual_ts, callee_start, callee_source)
            else {
                continue;
            };
            queries.push(TemplateQuery {
                kind: TemplateQueryKind::CallCallee,
                context,
                generated_offset,
                source_start: callee_start,
                source_end: callee_end,
                owner_start: source_start,
                owner_end: source_end,
            });
        }
    }

    if let Some(queries) = sinks.template_promise_queries.as_deref_mut() {
        for candidate in call_ranges.floating_promises {
            let candidate_start = source_start + candidate.start;
            let candidate_end = source_start + candidate.end;
            let probe_start = source_start + candidate.probe_start;
            let Some(probe_source) =
                source_text.get(candidate.probe_start as usize..candidate.probe_end as usize)
            else {
                continue;
            };
            let Some(generated_offset) =
                generated_offset_for_text(virtual_ts, probe_start, probe_source)
            else {
                continue;
            };
            let generated_offset = match candidate.probe_target {
                FloatingPromiseProbeTarget::SourceText => generated_offset,
                FloatingPromiseProbeTarget::ExpressionBinding => {
                    let Some(binding_offset) =
                        expression_binding_generated_offset(&virtual_ts.content, generated_offset)
                    else {
                        continue;
                    };
                    binding_offset
                }
            };
            queries.push(TemplatePromiseQuery {
                context,
                generated_offset,
                source_start: candidate_start,
                source_end: candidate_end,
            });
        }
    }
}

fn expression_binding_generated_offset(generated: &str, expression_offset: u32) -> Option<u32> {
    super::super::expression_bindings::binding_offset(generated, expression_offset)
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests;

use super::{
    absolute_expression_range,
    calls::{collect_template_call_ranges, FloatingPromiseProbeTarget},
    generated_offset_for_text, TemplateContext, TemplatePromiseQuery, TemplateQuery,
    TemplateQueryKind,
};
use vize_carton::profile;
use vize_croquis::virtual_ts::VirtualTsOutput;
use vize_relief::ast::{
    DirectiveNode, ExpressionNode, ForNode, IfNode, PropNode, RootNode, TemplateChildNode,
    TextCallContent,
};

pub(super) fn collect_template_query_sets(
    virtual_ts: &VirtualTsOutput,
    template_ast: &RootNode<'_>,
    template_offset: u32,
    include_template_queries: bool,
    include_template_promise_queries: bool,
) -> (Vec<TemplateQuery>, Vec<TemplatePromiseQuery>) {
    let mut template_queries = Vec::new();
    let mut template_promise_queries = Vec::new();
    let mut sinks = TemplateQuerySinks {
        template_queries: include_template_queries.then_some(&mut template_queries),
        template_promise_queries: include_template_promise_queries
            .then_some(&mut template_promise_queries),
    };

    profile!(
        "patina.type_aware.template_query_sets.walk",
        collect_children(
            virtual_ts,
            &template_ast.children,
            template_offset,
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
    virtual_ts: &VirtualTsOutput,
    children: &[TemplateChildNode<'_>],
    template_offset: u32,
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
                            collect_directive(virtual_ts, directive, template_offset, sinks)
                        );
                    }
                }
                profile!(
                    "patina.type_aware.template_query_sets.children",
                    collect_children(virtual_ts, &element.children, template_offset, sinks)
                );
            }
            TemplateChildNode::Interpolation(interpolation) => {
                collect_expression(
                    virtual_ts,
                    &interpolation.content,
                    template_offset,
                    TemplateContext::Interpolation,
                    false,
                    sinks,
                );
            }
            TemplateChildNode::If(if_node) => {
                profile!(
                    "patina.type_aware.template_query_sets.if",
                    collect_if(virtual_ts, if_node, template_offset, sinks)
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
                        sinks,
                    );
                }
                profile!(
                    "patina.type_aware.template_query_sets.children",
                    collect_children(virtual_ts, &branch.children, template_offset, sinks)
                );
            }
            TemplateChildNode::For(for_node) => {
                profile!(
                    "patina.type_aware.template_query_sets.for",
                    collect_for(virtual_ts, for_node, template_offset, sinks)
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
                        sinks,
                    );
                }
            }
            _ => {}
        }
    }
}

fn collect_if(
    virtual_ts: &VirtualTsOutput,
    if_node: &IfNode<'_>,
    template_offset: u32,
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
                sinks,
            );
        }
        profile!(
            "patina.type_aware.template_query_sets.children",
            collect_children(virtual_ts, &branch.children, template_offset, sinks)
        );
    }
}

fn collect_for(
    virtual_ts: &VirtualTsOutput,
    for_node: &ForNode<'_>,
    template_offset: u32,
    sinks: &mut TemplateQuerySinks<'_>,
) {
    collect_expression(
        virtual_ts,
        &for_node.source,
        template_offset,
        TemplateContext::Directive,
        false,
        sinks,
    );
    profile!(
        "patina.type_aware.template_query_sets.children",
        collect_children(virtual_ts, &for_node.children, template_offset, sinks)
    );
}

fn collect_expression(
    virtual_ts: &VirtualTsOutput,
    expression: &ExpressionNode<'_>,
    template_offset: u32,
    context: TemplateContext,
    allow_statement_fallback: bool,
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
                sinks,
            )
        );
    }
}

fn collect_directive(
    virtual_ts: &VirtualTsOutput,
    directive: &DirectiveNode<'_>,
    template_offset: u32,
    sinks: &mut TemplateQuerySinks<'_>,
) {
    let Some(expression) = &directive.exp else {
        return;
    };
    let context = match directive.name.as_str() {
        "bind" => TemplateContext::Binding,
        "on" => TemplateContext::Event,
        _ => TemplateContext::Directive,
    };
    collect_expression(
        virtual_ts,
        expression,
        template_offset,
        context,
        matches!(context, TemplateContext::Event),
        sinks,
    );
}

fn collect_expression_query_sets(
    virtual_ts: &VirtualTsOutput,
    expression: &ExpressionNode<'_>,
    template_offset: u32,
    context: TemplateContext,
    allow_statement_fallback: bool,
    sinks: &mut TemplateQuerySinks<'_>,
) {
    let Some((source_start, source_end)) = absolute_expression_range(expression, template_offset)
    else {
        return;
    };
    let source_text = expression.loc().source.as_str();
    let include_template_queries = sinks.template_queries.is_some();
    let include_template_promise_queries = sinks.template_promise_queries.is_some();
    let expression_generated_offset = include_template_queries
        .then(|| generated_offset_for_text(virtual_ts, source_start, source_text))
        .flatten();
    let include_call_callees = expression_generated_offset.is_some();
    let call_ranges = profile!(
        "patina.type_aware.template_query_sets.call_ranges",
        collect_template_call_ranges(
            source_text,
            allow_statement_fallback,
            include_call_callees,
            include_template_promise_queries,
        )
    );

    if let (Some(queries), Some(generated_offset)) = (
        sinks.template_queries.as_deref_mut(),
        expression_generated_offset,
    ) {
        queries.push(TemplateQuery {
            kind: TemplateQueryKind::Expression,
            context,
            generated_offset,
            source_start,
            source_end,
            owner_start: source_start,
            owner_end: source_end,
        });

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
    let offset = usize::min(expression_offset as usize, generated.len());
    let before_offset = generated.get(..offset)?;
    let after_offset = generated.get(offset..)?;
    let line_start = before_offset.rfind('\n').map_or(0, |index| index + 1);
    let line_end = after_offset
        .find('\n')
        .map_or(generated.len(), |index| offset + index);
    let line = generated.get(line_start..line_end)?;
    let const_start = line.find("const __expr_")?;
    let name_start = const_start + "const ".len();
    let name_end = line.get(name_start..)?.find(" = ")? + name_start;
    (name_end > name_start).then_some((line_start + name_end - 1) as u32)
}

#[cfg(test)]
mod tests {
    use super::expression_binding_generated_offset;

    #[test]
    fn finds_expression_binding_offset_on_generated_line() {
        let generated = "  const __expr_42 = actions[method];\n";
        let expression_offset = generated.find("method").unwrap() as u32;

        let binding_offset = expression_binding_generated_offset(generated, expression_offset)
            .expect("binding offset");

        assert_eq!(
            &generated[binding_offset as usize..binding_offset as usize + 1],
            "2"
        );
    }
}

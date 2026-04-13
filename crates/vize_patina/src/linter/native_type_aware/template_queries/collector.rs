use super::{
    absolute_expression_range, calls::collect_call_callee_ranges, generated_offset_for_text,
    TemplateContext, TemplateQuery, TemplateQueryKind,
};
use vize_carton::profile;
use vize_croquis::virtual_ts::VirtualTsOutput;
use vize_relief::ast::{
    DirectiveNode, ExpressionNode, ForNode, IfNode, PropNode, RootNode, TemplateChildNode,
    TextCallContent,
};

pub(super) fn collect_template_queries(
    virtual_ts: &VirtualTsOutput,
    template_ast: &RootNode<'_>,
    template_offset: u32,
) -> Vec<TemplateQuery> {
    let mut queries = Vec::new();
    profile!(
        "patina.type_aware.template_queries.walk",
        collect_children(
            virtual_ts,
            &template_ast.children,
            template_offset,
            &mut queries,
        )
    );
    profile!("patina.type_aware.template_queries.dedupe", {
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
    });
    queries
}

fn collect_children(
    virtual_ts: &VirtualTsOutput,
    children: &[TemplateChildNode<'_>],
    template_offset: u32,
    queries: &mut Vec<TemplateQuery>,
) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                for prop in &element.props {
                    let PropNode::Directive(directive) = prop else {
                        continue;
                    };
                    profile!(
                        "patina.type_aware.template_queries.directive",
                        collect_directive(virtual_ts, directive, template_offset, queries)
                    );
                }
                profile!(
                    "patina.type_aware.template_queries.children",
                    collect_children(virtual_ts, &element.children, template_offset, queries)
                );
            }
            TemplateChildNode::Interpolation(interpolation) => {
                profile!(
                    "patina.type_aware.template_queries.expression",
                    collect_expression_queries(
                        virtual_ts,
                        &interpolation.content,
                        template_offset,
                        TemplateContext::Interpolation,
                        false,
                        queries,
                    )
                );
            }
            TemplateChildNode::If(if_node) => {
                profile!(
                    "patina.type_aware.template_queries.if",
                    collect_if(virtual_ts, if_node, template_offset, queries)
                )
            }
            TemplateChildNode::IfBranch(branch) => {
                if let Some(condition) = &branch.condition {
                    profile!(
                        "patina.type_aware.template_queries.expression",
                        collect_expression_queries(
                            virtual_ts,
                            condition,
                            template_offset,
                            TemplateContext::Directive,
                            false,
                            queries,
                        )
                    );
                }
                profile!(
                    "patina.type_aware.template_queries.children",
                    collect_children(virtual_ts, &branch.children, template_offset, queries)
                );
            }
            TemplateChildNode::For(for_node) => {
                profile!(
                    "patina.type_aware.template_queries.for",
                    collect_for(virtual_ts, for_node, template_offset, queries)
                )
            }
            TemplateChildNode::TextCall(text_call) => {
                if let TextCallContent::Interpolation(interpolation) = &text_call.content {
                    profile!(
                        "patina.type_aware.template_queries.expression",
                        collect_expression_queries(
                            virtual_ts,
                            &interpolation.content,
                            template_offset,
                            TemplateContext::Interpolation,
                            false,
                            queries,
                        )
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
    queries: &mut Vec<TemplateQuery>,
) {
    for branch in &if_node.branches {
        if let Some(condition) = &branch.condition {
            profile!(
                "patina.type_aware.template_queries.expression",
                collect_expression_queries(
                    virtual_ts,
                    condition,
                    template_offset,
                    TemplateContext::Directive,
                    false,
                    queries,
                )
            );
        }
        profile!(
            "patina.type_aware.template_queries.children",
            collect_children(virtual_ts, &branch.children, template_offset, queries)
        );
    }
}

fn collect_for(
    virtual_ts: &VirtualTsOutput,
    for_node: &ForNode<'_>,
    template_offset: u32,
    queries: &mut Vec<TemplateQuery>,
) {
    profile!(
        "patina.type_aware.template_queries.expression",
        collect_expression_queries(
            virtual_ts,
            &for_node.source,
            template_offset,
            TemplateContext::Directive,
            false,
            queries,
        )
    );
    profile!(
        "patina.type_aware.template_queries.children",
        collect_children(virtual_ts, &for_node.children, template_offset, queries)
    );
}

fn collect_directive(
    virtual_ts: &VirtualTsOutput,
    directive: &DirectiveNode<'_>,
    template_offset: u32,
    queries: &mut Vec<TemplateQuery>,
) {
    let Some(expression) = &directive.exp else {
        return;
    };
    let context = match directive.name.as_str() {
        "bind" => TemplateContext::Binding,
        "on" => TemplateContext::Event,
        _ => TemplateContext::Directive,
    };
    profile!(
        "patina.type_aware.template_queries.expression",
        collect_expression_queries(
            virtual_ts,
            expression,
            template_offset,
            context,
            matches!(context, TemplateContext::Event),
            queries,
        )
    );
}

fn collect_expression_queries(
    virtual_ts: &VirtualTsOutput,
    expression: &ExpressionNode<'_>,
    template_offset: u32,
    context: TemplateContext,
    allow_statement_fallback: bool,
    queries: &mut Vec<TemplateQuery>,
) {
    let Some((source_start, source_end)) = absolute_expression_range(expression, template_offset)
    else {
        return;
    };
    let source_text = expression.loc().source.as_str();
    let Some(generated_offset) = generated_offset_for_text(virtual_ts, source_start, source_text)
    else {
        return;
    };
    queries.push(TemplateQuery {
        kind: TemplateQueryKind::Expression,
        context,
        generated_offset,
        source_start,
        source_end,
        owner_start: source_start,
        owner_end: source_end,
    });

    for callee in profile!(
        "patina.type_aware.template_queries.call_callees",
        collect_call_callee_ranges(source_text, allow_statement_fallback)
    ) {
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

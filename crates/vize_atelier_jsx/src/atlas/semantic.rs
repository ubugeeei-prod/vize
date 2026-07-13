//! Croquis semantic projection for the JSX frontend.

use vize_carton::FxHashMap;
use vize_croquis::{
    CroquisSemanticSnapshot, CroquisSemanticSnapshotBuilder, SemanticScopeBindingSnapshot,
    SemanticSourceRange,
};

use crate::{
    JsxSyntaxAttribute, JsxSyntaxAttributeValue, JsxSyntaxNode, JsxSyntaxSnapshot, JsxSyntaxSpan,
};

pub(super) fn project_semantics(snapshot: &JsxSyntaxSnapshot) -> CroquisSemanticSnapshot {
    let script_semantics = semantics_with_module_scopes(snapshot);
    let mut builder = CroquisSemanticSnapshotBuilder::from_snapshot(script_semantics.clone());
    for node in &snapshot.roots {
        collect_node(node, &script_semantics, &mut builder);
    }
    let mut semantics = builder.finish();
    semantics.source_anchor = snapshot.source_anchor;
    semantics
}

fn semantics_with_module_scopes(snapshot: &JsxSyntaxSnapshot) -> CroquisSemanticSnapshot {
    let semantics = snapshot.analysis().semantic_snapshot();
    let mut next_scope_id = semantics
        .scopes
        .iter()
        .map(|scope| scope.id)
        .max()
        .map_or(0, |id| id + 1);
    let mut builder = CroquisSemanticSnapshotBuilder::from_snapshot(semantics.clone());
    for module in &snapshot.module().modules {
        let mut scope_ids = FxHashMap::default();
        for function in &module.operations.functions {
            scope_ids.insert(function.id, next_scope_id);
            next_scope_id += 1;
        }
        for function in &module.operations.functions {
            let scope_id = scope_ids[&function.id];
            let parent_id = function
                .parent
                .and_then(|parent| scope_ids.get(&parent).copied())
                .unwrap_or_else(|| {
                    scope_id_for_span(
                        &semantics,
                        JsxSyntaxSpan::new(function.span.start, function.span.end),
                    )
                });
            let bindings = function
                .local_bindings
                .iter()
                .map(|name| {
                    SemanticScopeBindingSnapshot::new(
                        name,
                        "callbackLocal",
                        function.span.start,
                        function.references.contains(name),
                        false,
                    )
                })
                .collect();
            builder.add_scope(
                scope_id,
                vec![parent_id],
                "callback",
                SemanticSourceRange::new(function.span.start, function.span.end),
                bindings,
            );
        }
    }
    builder.finish()
}

fn collect_node(
    node: &JsxSyntaxNode,
    semantics: &CroquisSemanticSnapshot,
    builder: &mut CroquisSemanticSnapshotBuilder,
) {
    match node {
        JsxSyntaxNode::Element(element) => {
            let scope_id = scope_id_for_span(semantics, element.span);
            let usage = element.component.then(|| {
                builder.add_component_usage(
                    &element.name,
                    range(element.span),
                    scope_id,
                    element
                        .attributes
                        .iter()
                        .any(|attribute| matches!(attribute, JsxSyntaxAttribute::Spread { .. })),
                )
            });
            for attribute in &element.attributes {
                match attribute {
                    JsxSyntaxAttribute::Attribute {
                        name, value, span, ..
                    } => {
                        if let JsxSyntaxAttributeValue::Expression(expression) = value {
                            builder.add_template_expression(
                                &expression.code,
                                "jsx-attribute",
                                range(expression.span),
                                scope_id_for_span(semantics, expression.span),
                            );
                        }
                        if let Some(usage) = usage {
                            add_component_attribute(builder, usage, name, value, *span);
                        }
                    }
                    JsxSyntaxAttribute::Spread { expression, .. } => builder
                        .add_template_expression(
                            &expression.code,
                            "jsx-attribute",
                            range(expression.span),
                            scope_id_for_span(semantics, expression.span),
                        ),
                }
            }
            if let Some(usage) = usage.filter(|_| !element.children.is_empty()) {
                builder.add_component_slot(usage, "default", range(element.span));
            }
            collect_nodes(&element.children, semantics, builder);
        }
        JsxSyntaxNode::Fragment { children, .. } => collect_nodes(children, semantics, builder),
        JsxSyntaxNode::Expression { expression, .. } => builder.add_template_expression(
            &expression.code,
            "jsx-expression",
            range(expression.span),
            scope_id_for_span(semantics, expression.span),
        ),
        JsxSyntaxNode::If { branches, .. } => {
            for branch in branches {
                if let Some(condition) = &branch.condition {
                    builder.add_template_expression(
                        &condition.code,
                        "jsx-condition",
                        range(condition.span),
                        scope_id_for_span(semantics, condition.span),
                    );
                }
                collect_nodes(&branch.body, semantics, builder);
            }
        }
        JsxSyntaxNode::For { source, body, .. } => {
            builder.add_template_expression(
                &source.code,
                "jsx-iteration",
                range(source.span),
                scope_id_for_span(semantics, source.span),
            );
            collect_nodes(body, semantics, builder);
        }
        JsxSyntaxNode::Text { .. } | JsxSyntaxNode::Comment { .. } => {}
    }
}

fn collect_nodes(
    nodes: &[JsxSyntaxNode],
    semantics: &CroquisSemanticSnapshot,
    builder: &mut CroquisSemanticSnapshotBuilder,
) {
    for node in nodes {
        collect_node(node, semantics, builder);
    }
}

fn add_component_attribute(
    builder: &mut CroquisSemanticSnapshotBuilder,
    usage: usize,
    name: &str,
    value: &JsxSyntaxAttributeValue,
    span: JsxSyntaxSpan,
) {
    let value_text: Option<&str> = match value {
        JsxSyntaxAttributeValue::Presence => None,
        JsxSyntaxAttributeValue::Static { value, .. } => Some(value.as_ref()),
        JsxSyntaxAttributeValue::Expression(expression) => Some(expression.code.as_ref()),
    };
    if let Some(event) = jsx_event_name(name) {
        builder.add_component_event(usage, event.as_str(), value_text, range(span));
    } else {
        builder.add_component_prop(
            usage,
            name,
            value_text,
            range(span),
            matches!(value, JsxSyntaxAttributeValue::Expression(_)),
        );
    }
}

fn jsx_event_name(name: &str) -> Option<vize_carton::String> {
    let event = name.strip_prefix("on")?;
    let first = event.chars().next()?;
    if !first.is_ascii_uppercase() {
        return None;
    }
    let mut normalized = vize_carton::String::with_capacity(event.len());
    normalized.push(first.to_ascii_lowercase());
    normalized.push_str(&event[first.len_utf8()..]);
    Some(normalized)
}

fn scope_id_for_span(semantics: &CroquisSemanticSnapshot, span: JsxSyntaxSpan) -> u32 {
    semantics
        .scopes
        .iter()
        .filter(|scope| scope.range.start <= span.start && span.end <= scope.range.end)
        .min_by_key(|scope| scope.range.end.saturating_sub(scope.range.start))
        .or_else(|| semantics.scopes.first())
        .map_or(0, |scope| scope.id)
}

const fn range(span: JsxSyntaxSpan) -> SemanticSourceRange {
    SemanticSourceRange::new(span.start, span.end)
}

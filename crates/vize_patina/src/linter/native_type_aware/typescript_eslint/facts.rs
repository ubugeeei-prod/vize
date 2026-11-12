//! Host facts the corsa rules do not infer from type text alone.
use super::tree::Collected;

mod checker;
mod destructuring;
mod enrich;
mod enums;
mod module_this;
mod scan;
mod shape;
use crate::linter::corsa_session::{ExpressionFact, ExpressionQuery};
use corsa::lint::LintNode;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

pub(super) fn prepare(source: &str, collected: &mut Collected) {
    let deprecated = scan::deprecated_names(source);
    let functions = scan::local_functions(&collected.program, source);
    let methods = scan::class_methods(&collected.program);
    let instances = scan::class_instances(&collected.program);
    for node in &mut collected.nodes {
        stamp(node, &deprecated, &functions, &methods, &instances);
    }
    enums::stamp_merged_enums(&collected.program, source, &mut collected.nodes);
}

pub(super) use checker::Needs;

pub(super) fn queries(nodes: &[LintNode], needs: Needs) -> Vec<ExpressionQuery> {
    let mut queries = Vec::new();
    for node in nodes {
        collect_queries(node, &mut queries, needs);
        checker::listener_queries(node, &mut queries, needs);
    }
    queries
}

pub(super) fn enrich(source: &str, nodes: &mut [LintNode], program: &Value) {
    enrich::apply(source, nodes, program);
}

pub(super) fn apply(nodes: &mut [LintNode], facts: &[ExpressionFact], queries: &[ExpressionQuery]) {
    let mut by_range = HashMap::<(u32, u32), &ExpressionFact>::new();
    // A callee is queried twice: once for its signature, once as a plain node.
    // Keep the signature answer apart so the plain one cannot erase it.
    let mut signatures = HashMap::<(u32, u32), &ExpressionFact>::new();
    let mut listener = checker::ListenerFacts::default();
    for (fact, query) in facts.iter().zip(queries) {
        let key = (query.start, query.end);
        if query.signatures {
            signatures.insert(key, fact);
        }
        listener.record(query, fact);
        by_range.entry(key).or_insert(fact);
    }
    for node in nodes {
        apply_node(node, &by_range, &signatures);
        checker::stamp_listener(node, &listener);
        module_this::stamp_module_this(node);
    }
}

fn stamp(
    node: &mut LintNode,
    deprecated: &HashSet<String>,
    functions: &HashMap<String, scan::LocalFn>,
    methods: &HashMap<String, HashSet<String>>,
    instances: &HashMap<String, String>,
) {
    if node.kind == "Identifier"
        && node.field_bool("__declarationName") != Some(true)
        && node
            .field_str("name")
            .is_some_and(|name| deprecated.contains(name))
    {
        node.fields.insert(
            "__symbolFacts".into(),
            json!({ "deprecated": true, "deprecationReason": "" }),
        );
    }
    if node.kind == "CallExpression" {
        stamp_call(node, functions);
    }
    if node.kind == "MemberExpression" {
        stamp_member(node, methods, instances);
    }
    for child in node.children.values_mut() {
        stamp(child, deprecated, functions, methods, instances);
    }
    for children in node.child_lists.values_mut() {
        for child in children {
            stamp(child, deprecated, functions, methods, instances);
        }
    }
}

fn stamp_call(node: &mut LintNode, functions: &HashMap<String, scan::LocalFn>) {
    let property = node
        .child("callee")
        .and_then(|callee| callee.child("property"))
        .and_then(|property| property.field_str("name"));
    if property == Some("forEach") {
        node.fields.insert(
            "__expectedArgumentReturnTypeTexts".into(),
            json!([["void"]]),
        );
    }
    let Some(name) = node
        .child("callee")
        .and_then(|callee| callee.field_str("name"))
        .map(str::to_owned)
    else {
        return;
    };
    let Some(function) = functions.get(&name) else {
        return;
    };
    let arguments = type_argument_nodes(node);
    if arguments.is_empty() || function.params.is_empty() {
        return;
    }
    let last = arguments.len() - 1;
    let ranges = arguments
        .iter()
        .map(|argument| (argument.range.start, argument.range.end))
        .collect::<Vec<_>>();
    let argument_text = arguments
        .get(last)
        .and_then(|argument| argument.text.clone());
    let range_values = ranges
        .iter()
        .map(|(start, end)| json!({ "start": start, "end": end }))
        .collect::<Vec<_>>();
    node.fields
        .insert("__typeArgumentRanges".into(), Value::Array(range_values));
    node.fields
        .insert("__typeParameterCount".into(), json!(function.params.len()));
    let default_text = function
        .params
        .get(last)
        .and_then(|(_, default)| default.clone());
    node.fields.insert(
        "__lastTypeParameterHasDefault".into(),
        json!(default_text.is_some()),
    );
    node.fields.insert(
        "__lastTypeArgumentEqualsDefault".into(),
        json!(default_text.is_some() && default_text == argument_text),
    );
}

fn type_argument_nodes(node: &LintNode) -> &[LintNode] {
    node.child("typeArguments")
        .or_else(|| node.child("typeParameters"))
        .and_then(|list| list.child_list("params"))
        .unwrap_or(&[])
}

fn stamp_member(
    node: &mut LintNode,
    methods: &HashMap<String, HashSet<String>>,
    instances: &HashMap<String, String>,
) {
    if node.field_str("__parentKind") == Some("CallExpression") {
        node.fields.insert("__safeUse".into(), json!(true));
        return;
    }
    let Some(object) = node
        .child("object")
        .and_then(|object| object.field_str("name"))
    else {
        return;
    };
    let Some(class_name) = instances.get(object) else {
        return;
    };
    let Some(property) = node
        .child("property")
        .and_then(|property| property.field_str("name"))
    else {
        return;
    };
    if methods
        .get(class_name)
        .is_some_and(|names| names.contains(property))
    {
        node.fields.insert(
            "__unboundMethodInfo".into(),
            json!({ "isMethod": true, "firstParamIsThis": false, "thisArgIsVoid": false, "isStatic": false }),
        );
    }
}

fn collect_queries(node: &LintNode, queries: &mut Vec<ExpressionQuery>, needs: Needs) {
    queries.push(ExpressionQuery {
        start: node.range.start,
        end: node.range.end,
        symbols: node.kind == "Identifier",
        signatures: false,
        call_positions: 0,
        function_returns: None,
        contextual: false,
        detail: needs.details,
        paths: Vec::new(),
    });
    let callee = match node.kind.as_str() {
        "CallExpression" | "NewExpression" => node.child("callee"),
        "TaggedTemplateExpression" => node.child("tag"),
        _ => None,
    };
    if let Some(callee) = callee {
        queries.push(ExpressionQuery {
            start: callee.range.start,
            end: callee.range.end,
            symbols: false,
            signatures: true,
            call_positions: 0,
            function_returns: None,
            contextual: false,
            detail: needs.details,
            paths: Vec::new(),
        });
    }
    for child in node.children.values() {
        collect_queries(child, queries, needs);
    }
    for children in node.child_lists.values() {
        for child in children {
            collect_queries(child, queries, needs);
        }
    }
}

fn apply_node(
    node: &mut LintNode,
    facts: &HashMap<(u32, u32), &ExpressionFact>,
    signatures: &HashMap<(u32, u32), &ExpressionFact>,
) {
    if let Some(fact) = facts.get(&(node.range.start, node.range.end)) {
        node.type_texts.clone_from(&fact.type_texts);
        node.property_names.clone_from(&fact.property_names);
        checker::stamp_detail(node, fact);
        if let Some((call, construct)) = fact.function_signatures {
            node.fields
                .insert("__callSignatureCount".into(), json!(call));
            node.fields
                .insert("__constructSignatureCount".into(), json!(construct));
        }
        if fact.deprecated && node.field_bool("__declarationName") != Some(true) {
            node.fields.insert(
                "__symbolFacts".into(),
                json!({
                    "deprecated": true,
                    "deprecationReason": fact.deprecation_reason,
                }),
            );
        }
    }
    if node.kind == "CallExpression" {
        if let Some(callee) = node.child("callee") {
            if let Some(fact) = signatures.get(&(callee.range.start, callee.range.end)) {
                if !fact.parameter_type_texts.is_empty() {
                    node.fields.insert(
                        "__expectedArgumentTypeTexts".into(),
                        json!(fact.parameter_type_texts),
                    );
                }
            }
        }
    }
    let keys = node.children.keys().cloned().collect::<Vec<_>>();
    for key in keys {
        if let Some(child) = node.children.get_mut(&key) {
            apply_node(child, facts, signatures);
        }
    }
    let lists = node.child_lists.keys().cloned().collect::<Vec<_>>();
    for key in lists {
        if let Some(children) = node.child_lists.get_mut(&key) {
            for child in children {
                apply_node(child, facts, signatures);
            }
        }
    }
}

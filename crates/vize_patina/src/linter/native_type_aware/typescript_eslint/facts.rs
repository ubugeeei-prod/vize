//! Host facts the corsa rules do not infer from type text alone.
use super::tree::Collected;

mod enrich;
mod queries;
mod scan;
mod shape;
use crate::linter::corsa_session::{ExpressionFact, ExpressionQuery};
use corsa::lint::LintNode;
pub(super) use queries::queries;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

pub(super) fn prepare(source: &str, collected: &mut Collected) {
    let deprecated = scan::deprecated_names(source);
    let functions = scan::local_functions(&collected.program, source);
    for node in &mut collected.nodes {
        stamp(node, &deprecated, &functions);
    }
}

pub(super) fn enrich(source: &str, nodes: &mut [LintNode], program: &Value) {
    enrich::apply(source, nodes, program);
}

pub(super) fn apply(nodes: &mut [LintNode], facts: &[ExpressionFact], queries: &[ExpressionQuery]) {
    let mut by_range = HashMap::<(u32, u32), &ExpressionFact>::new();
    for (fact, query) in facts.iter().zip(queries) {
        by_range.insert((query.start, query.end), fact);
    }
    for node in nodes {
        apply_node(node, &by_range);
    }
}

fn stamp(
    node: &mut LintNode,
    deprecated: &HashSet<String>,
    functions: &HashMap<String, scan::LocalFn>,
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
    for child in node.children.values_mut() {
        stamp(child, deprecated, functions);
    }
    for children in node.child_lists.values_mut() {
        for child in children {
            stamp(child, deprecated, functions);
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

/// Copies the destructured initializer's member facts onto the pattern.
fn apply_pattern_init(node: &mut LintNode, facts: &HashMap<(u32, u32), &ExpressionFact>) {
    let Some(range) = node.fields.get("__initRange") else {
        return;
    };
    let start = range.get("start").and_then(Value::as_u64);
    let end = range.get("end").and_then(Value::as_u64);
    let (Some(start), Some(end)) = (start, end) else {
        return;
    };
    let Some(fact) = facts.get(&(start as u32, end as u32)) else {
        return;
    };
    if let Some(members) = &fact.members {
        node.fields
            .insert("__initMemberMethods".into(), members.clone());
    }
    if let Some(declared) = fact.declared_in_file {
        node.fields
            .insert("__initDeclaredInFile".into(), json!(declared));
    }
}

fn apply_node(node: &mut LintNode, facts: &HashMap<(u32, u32), &ExpressionFact>) {
    if let Some(fact) = facts.get(&(node.range.start, node.range.end)) {
        node.type_texts.clone_from(&fact.type_texts);
        node.property_names.clone_from(&fact.property_names);
        if let Some(declared) = fact.declared_in_file {
            node.fields
                .insert("__declaredInFile".into(), json!(declared));
        }
        if let Some(flags) = fact.type_flags {
            node.fields.insert("__typeFlags".into(), json!(flags));
        }
        if let Some(shape) = &fact.shape {
            node.fields.insert("__typeShape".into(), shape.clone());
        }
        if let Some(members) = &fact.members {
            node.fields
                .insert("__memberMethods".into(), members.clone());
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
    if node.kind == "ObjectPattern" {
        apply_pattern_init(node, facts);
    }
    if node.kind == "CallExpression" {
        if let Some(callee) = node.child("callee") {
            if let Some(fact) = facts.get(&(callee.range.start, callee.range.end)) {
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
            apply_node(child, facts);
        }
    }
    let lists = node.child_lists.keys().cloned().collect::<Vec<_>>();
    for key in lists {
        if let Some(children) = node.child_lists.get_mut(&key) {
            for child in children {
                apply_node(child, facts);
            }
        }
    }
}

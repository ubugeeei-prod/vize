//! Host facts the corsa rules do not infer from type text alone.
use super::tree::Collected;

mod enrich;
mod scan;
mod shape;
use crate::linter::corsa_session::{ExpressionFact, ExpressionQuery};
use corsa::lint::LintNode;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use vize_l0::FxHashSet;

pub(super) fn prepare(source: &str, collected: &mut Collected) {
    let deprecated = scan::deprecated_names(source);
    let functions = scan::local_functions(&collected.program, source);
    let methods = scan::class_methods(&collected.program);
    let instances = scan::class_instances(&collected.program);
    for node in &mut collected.nodes {
        stamp(node, &deprecated, &functions, &methods, &instances);
    }
}

/// Which checker facts about type nodes the enabled rules read.
#[derive(Clone, Copy)]
pub(super) struct TypeNodeWants {
    /// Union constituents of every union/intersection member.
    parts: bool,
    /// The empty-object probe on intersections and generic references.
    empty_object: bool,
}

impl TypeNodeWants {
    /// What the enabled `type/` rules need.
    pub(super) fn for_rules(rules: &[&str]) -> Self {
        Self {
            parts: rules.iter().any(|rule| {
                matches!(
                    *rule,
                    "type/no-redundant-type-constituents" | "type/no-duplicate-type-constituents"
                )
            }),
            empty_object: rules.contains(&"type/no-generated-empty-object-type"),
        }
    }

    pub(super) fn empty_object(self) -> bool {
        self.empty_object
    }
}

pub(super) fn queries(nodes: &[LintNode], wants: TypeNodeWants) -> Vec<ExpressionQuery> {
    let mut members = FxHashSet::default();
    if wants.parts {
        for node in nodes {
            type_members(node, &mut members);
        }
    }
    let mut queries = Vec::new();
    for node in nodes {
        collect_queries(node, wants, &members, &mut queries);
    }
    queries
}

/// Ranges of every union/intersection member under `node`.
fn type_members(node: &LintNode, members: &mut FxHashSet<(u32, u32)>) {
    if matches!(node.kind.as_str(), "TSUnionType" | "TSIntersectionType") {
        for member in node.child_list("types").unwrap_or(&[]) {
            members.insert((member.range.start, member.range.end));
        }
    }
    for child in node.children.values() {
        type_members(child, members);
    }
    for children in node.child_lists.values() {
        for child in children {
            type_members(child, members);
        }
    }
}

pub(super) fn enrich(source: &str, nodes: &mut [LintNode], program: &Value) {
    enrich::apply(source, nodes, program);
}

pub(super) fn apply(nodes: &mut [LintNode], facts: &[ExpressionFact], queries: &[ExpressionQuery]) {
    let mut by_range = HashMap::<(u32, u32), &ExpressionFact>::new();
    for (fact, query) in facts.iter().zip(queries) {
        // Callee queries share a range with the callee's own query; keep the
        // richer type-node facts when two queries land on one range.
        let slot = by_range.entry((query.start, query.end)).or_insert(fact);
        if slot.type_parts.is_empty() && slot.empty_object.is_none() {
            *slot = fact;
        }
    }
    for node in nodes {
        apply_node(node, &by_range);
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

fn collect_queries(
    node: &LintNode,
    wants: TypeNodeWants,
    members: &FxHashSet<(u32, u32)>,
    queries: &mut Vec<ExpressionQuery>,
) {
    queries.push(ExpressionQuery {
        start: node.range.start,
        end: node.range.end,
        symbols: node.kind == "Identifier",
        parts: members.contains(&(node.range.start, node.range.end)),
        empty_object: wants.empty_object && may_generate_empty_object(node),
        ..ExpressionQuery::default()
    });
    if node.kind == "CallExpression" {
        if let Some(callee) = node.child("callee") {
            queries.push(ExpressionQuery {
                start: callee.range.start,
                end: callee.range.end,
                signatures: true,
                ..ExpressionQuery::default()
            });
        }
    }
    for child in node.children.values() {
        collect_queries(child, wants, members, queries);
    }
    for children in node.child_lists.values() {
        for child in children {
            collect_queries(child, wants, members, queries);
        }
    }
}

/// The nodes `no-generated-empty-object-type` inspects: intersections and
/// type references with type arguments.
fn may_generate_empty_object(node: &LintNode) -> bool {
    node.kind == "TSIntersectionType"
        || (node.kind == "TSTypeReference" && node.child("typeArguments").is_some())
}

fn apply_node(node: &mut LintNode, facts: &HashMap<(u32, u32), &ExpressionFact>) {
    if let Some(fact) = facts.get(&(node.range.start, node.range.end)) {
        node.type_texts.clone_from(&fact.type_texts);
        node.property_names.clone_from(&fact.property_names);
        type_node_fields(node, fact);
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

// TypeScript 7 TypeFlags, from microsoft/typescript-go internal/checker/types.go.
const TYPE_FLAG_UNDEFINED: u32 = 1 << 2;
const TYPE_PART_LABELS: &[(u32, &str)] = &[
    (1, "any"),
    (1 << 1, "unknown"),
    (1 << 18, "never"),
    (1 << 5, "string"),
    (1 << 6, "number"),
    (1 << 7, "bigint"),
    (1 << 8, "boolean"),
    (1 << 10, "stringLiteral"),
    (1 << 11, "numberLiteral"),
    (1 << 12, "bigintLiteral"),
    (1 << 13, "booleanLiteral"),
    (1 << 22, "templateLiteral"),
];

/// Checker facts the union/intersection rules read off one member node.
fn type_node_fields(node: &mut LintNode, fact: &ExpressionFact) {
    if let Some(empty) = fact.empty_object {
        node.fields.insert("__emptyObjectType".into(), json!(empty));
    }
    if fact.error_type {
        // An unresolved name: duplicate detection skips it. It gets no part
        // flags, because the virtual SFC module can leave names unresolved
        // that the user's TypeScript resolves (hoisted declarations, lib
        // types), and reporting those as `error` types would be wrong.
        node.fields.insert("__isErrorType".into(), json!(true));
        return;
    }
    if fact.type_parts.is_empty() {
        return;
    }
    let parts = fact
        .type_parts
        .iter()
        .map(|part| {
            let label = TYPE_PART_LABELS
                .iter()
                .find(|(mask, _)| part.flags & mask != 0)
                .map_or("other", |(_, label)| label);
            json!({ "flag": label, "flags": part.flags, "text": part.text.as_str() })
        })
        .collect::<Vec<_>>();
    node.fields
        .insert("__typePartFlags".into(), Value::Array(parts));
    if let Some(id) = &fact.type_id {
        node.fields
            .insert("__resolvedTypeId".into(), json!(id.as_str()));
    }
    if fact.type_flags & TYPE_FLAG_UNDEFINED != 0 {
        node.fields.insert("__isUndefinedType".into(), json!(true));
    }
}

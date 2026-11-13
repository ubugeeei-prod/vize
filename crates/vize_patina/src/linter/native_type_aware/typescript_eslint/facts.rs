//! Host facts the corsa rules do not infer from type text alone.
use super::tree::Collected;

mod builtins;
mod checker;
mod destructuring;
mod enrich;
mod enums;
mod module_this;
mod promise;
mod queries;
mod scan;
mod shape;
mod type_parameters;
mod type_shape;
use crate::linter::corsa_session::{ExpressionFact, ExpressionQuery};
use corsa::lint::LintNode;
pub(super) use queries::{Wants, queries};
use serde_json::{Value, json};
use vize_l0::FxHashMap;

pub(super) fn prepare(source: &str, collected: &mut Collected) {
    for node in &mut collected.nodes {
        stamp(node);
    }
    enums::stamp_merged_enums(&collected.program, source, &mut collected.nodes);
}

pub(super) fn enrich(source: &str, nodes: &mut [LintNode], program: &Value) {
    enrich::apply(source, nodes, program);
    promise::enrich(nodes, program);
    builtins::stamp(nodes, program);
}

/// Stamps each answered query's facts onto the nodes at its range.
pub(super) fn apply(nodes: &mut [LintNode], facts: &[ExpressionFact], queries: &[ExpressionQuery]) {
    let by_range = facts
        .iter()
        .zip(queries)
        .map(|(fact, query)| ((query.start, query.end), fact))
        .collect::<FxHashMap<_, _>>();
    for node in nodes {
        apply_node(node, &by_range);
        checker::stamp_listener(node, &by_range);
        type_parameters::apply(node, &by_range);
        module_this::stamp_module_this(node);
    }
}

fn stamp(node: &mut LintNode) {
    if node.kind == "CallExpression" {
        stamp_call(node);
    }
    for child in node.children.values_mut() {
        stamp(child);
    }
    for children in node.child_lists.values_mut() {
        for child in children {
            stamp(child);
        }
    }
}

fn stamp_call(node: &mut LintNode) {
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
}

/// Copies the destructured initializer's member facts onto the pattern.
fn apply_pattern_init(node: &mut LintNode, facts: &FxHashMap<(u32, u32), &ExpressionFact>) {
    let Some(range) = node.fields.get("__initRange") else {
        return;
    };
    let field = |key: &str| u32::try_from(range.get(key)?.as_u64()?).ok();
    let (Some(start), Some(end)) = (field("start"), field("end")) else {
        return;
    };
    let Some(fact) = facts.get(&(start, end)) else {
        return;
    };
    if let Some(members) = &fact.member_methods {
        node.fields
            .insert("__initMemberMethods".into(), members.clone());
    }
    if let Some(declared) = fact.declared_in_file {
        node.fields
            .insert("__initDeclaredInFile".into(), json!(declared));
    }
}

fn apply_node(node: &mut LintNode, facts: &FxHashMap<(u32, u32), &ExpressionFact>) {
    if let Some(fact) = facts.get(&(node.range.start, node.range.end)) {
        node.type_texts = fact
            .type_texts
            .iter()
            .map(|text| text.as_str().into())
            .collect();
        node.property_names = fact
            .property_names
            .iter()
            .map(|name| name.as_str().into())
            .collect();
        if let Some(flags) = fact.type_flags {
            node.fields.insert("__typeFlags".into(), json!(flags));
        }
        if let Some(declared) = fact.declared_in_file {
            node.fields
                .insert("__declaredInFile".into(), json!(declared));
        }
        if let Some(ty) = &fact.ty {
            node.fields.insert("__type".into(), ty.clone());
        }
        if let Some((call, construct)) = fact.function_signatures {
            node.fields
                .insert("__callSignatureCount".into(), json!(call));
            node.fields
                .insert("__constructSignatureCount".into(), json!(construct));
        }
        if let Some(context) = &fact.context {
            node.fields.insert("__typeContext".into(), context.clone());
        }
        if let Some(members) = &fact.member_methods {
            node.fields
                .insert("__memberMethods".into(), members.clone());
        }
        promise::apply(node, fact);
        type_node_fields(node, fact);
    }
    if node.kind == "ObjectPattern" {
        apply_pattern_init(node, facts);
    }
    let callee = match node.kind.as_str() {
        "CallExpression" | "NewExpression" => node.child("callee"),
        _ => None,
    };
    if let Some(fact) = callee.and_then(|callee| facts.get(&(callee.range.start, callee.range.end)))
    {
        promise::apply_call(node, fact);
        if node.kind == "CallExpression" && !fact.parameter_type_texts.is_empty() {
            node.fields.insert(
                "__expectedArgumentTypeTexts".into(),
                json!(fact.parameter_type_texts),
            );
        }
    }
    for child in node.children.values_mut() {
        apply_node(child, facts);
    }
    for children in node.child_lists.values_mut() {
        for child in children {
            apply_node(child, facts);
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
    if fact
        .type_flags
        .is_some_and(|flags| flags & TYPE_FLAG_UNDEFINED != 0)
    {
        node.fields.insert("__isUndefinedType".into(), json!(true));
    }
}

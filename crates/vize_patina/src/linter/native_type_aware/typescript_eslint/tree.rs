//! TS-ESTree nodes for the corsa typescript-eslint rules.
use corsa::lint::{LintNode, TextRange};
use oxc_allocator::Allocator;
use oxc_estree::{CompactSerializer, ESTree};
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

const DEPTH: u32 = 24;

const DECLARATIONS: &[&str] = &[
    "VariableDeclarator",
    "FunctionDeclaration",
    "FunctionExpression",
    "ArrowFunctionExpression",
    "ClassDeclaration",
    "ClassExpression",
    "TSEnumDeclaration",
    "TSEnumMember",
    "TSInterfaceDeclaration",
    "TSModuleDeclaration",
    "TSTypeAliasDeclaration",
    "TSTypeParameter",
    "TSMethodSignature",
    "TSPropertySignature",
    "MethodDefinition",
    "PropertyDefinition",
    "AccessorProperty",
    "TSParameterProperty",
    "Property",
    "TSImportEqualsDeclaration",
];

pub(super) struct Collected {
    pub program: Value,
    pub nodes: Vec<LintNode>,
}

pub(super) fn collect(source: &str, tsx: bool, listeners: &[&str]) -> Collected {
    let allocator = Allocator::default();
    let source_type = if tsx {
        SourceType::tsx()
    } else {
        SourceType::ts()
    };
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked {
        return Collected {
            program: Value::Null,
            nodes: Vec::new(),
        };
    }
    let mut serializer = CompactSerializer::new(true, false);
    parsed.program.serialize(&mut serializer);
    let mut program = serde_json::from_str(&serializer.into_string()).unwrap_or(Value::Null);
    normalize(&mut program);
    let mut nodes = Vec::new();
    walk(
        &program,
        source,
        listeners,
        &mut Vec::new(),
        None,
        None,
        &mut nodes,
    );
    Collected { program, nodes }
}

fn walk(
    value: &Value,
    source: &str,
    listeners: &[&str],
    ancestors: &mut Vec<Value>,
    parent_kind: Option<&str>,
    key: Option<&str>,
    out: &mut Vec<LintNode>,
) {
    let Some(kind) = value.get("type").and_then(Value::as_str) else {
        return;
    };
    if listeners.contains(&kind) {
        if let Some(mut node) = convert(value, source, DEPTH, parent_kind, key) {
            node.fields
                .insert("__ancestorFacts".into(), Value::Array(ancestors.clone()));
            node.fields
                .insert("__ancestorChain".into(), ancestor_chain(ancestors, key));
            out.push(node);
        }
    }
    ancestors.push(ancestor_fact(value, kind, key));
    for (child_key, child) in object_entries(value) {
        match child {
            Value::Object(_) => walk(
                child,
                source,
                listeners,
                ancestors,
                Some(kind),
                Some(child_key),
                out,
            ),
            Value::Array(items) => {
                for item in items {
                    walk(
                        item,
                        source,
                        listeners,
                        ancestors,
                        Some(kind),
                        Some(child_key),
                        out,
                    );
                }
            }
            _ => {}
        }
    }
    ancestors.pop();
}

fn convert(
    value: &Value,
    source: &str,
    depth: u32,
    parent_kind: Option<&str>,
    key: Option<&str>,
) -> Option<LintNode> {
    let kind = value.get("type").and_then(Value::as_str)?;
    let start = value.get("start").and_then(Value::as_u64)? as u32;
    let end = value.get("end").and_then(Value::as_u64)? as u32;
    let mut fields = BTreeMap::new();
    let mut children = BTreeMap::new();
    let mut child_lists = BTreeMap::new();
    if let Some(parent_kind) = parent_kind {
        fields.insert("__parentKind".into(), json!(parent_kind));
    }
    if key == Some("id") && parent_kind.is_some_and(|kind| DECLARATIONS.contains(&kind)) {
        fields.insert("__declarationName".into(), json!(true));
    }
    if let Some(map) = value.as_object() {
        for (name, child) in map {
            if matches!(name.as_str(), "type" | "start" | "end" | "range" | "loc") {
                continue;
            }
            match child {
                Value::Object(_) if child.get("type").is_some() && depth > 0 => {
                    if let Some(node) = convert(child, source, depth - 1, Some(kind), Some(name)) {
                        children.insert(name.clone(), node);
                    }
                }
                Value::Array(items)
                    if depth > 0 && items.iter().all(|item| item.get("type").is_some()) =>
                {
                    let nodes = items
                        .iter()
                        .filter_map(|item| convert(item, source, depth - 1, Some(kind), Some(name)))
                        .collect();
                    child_lists.insert(name.clone(), nodes);
                }
                other => {
                    fields.insert(name.clone(), other.clone());
                }
            }
        }
    }
    let text = source.get(start as usize..end as usize).map(str::to_owned);
    let mut node = LintNode {
        kind: kind.to_owned(),
        range: TextRange::new(start, end),
        text,
        type_texts: Vec::new(),
        property_names: Vec::new(),
        fields,
        children,
        child_lists,
    };
    if let Some(annotation) = node.child("typeAnnotation") {
        let inner = annotation.child("typeAnnotation").unwrap_or(annotation);
        if let Some(slice) = source.get(inner.range.start as usize..inner.range.end as usize) {
            node.fields
                .insert("__typeAnnotationText".into(), json!(slice));
        }
    }
    Some(node)
}

fn ancestor_chain(ancestors: &[Value], key: Option<&str>) -> Value {
    let mut chain = Vec::new();
    let mut child_key = key.map(str::to_owned);
    for ancestor in ancestors.iter().rev() {
        let mut entry = Map::new();
        if let Some(kind) = ancestor.get("kind") {
            entry.insert("kind".into(), kind.clone());
        }
        if let Some(operator) = ancestor.get("operator") {
            entry.insert("operator".into(), operator.clone());
        }
        let position = child_key.as_deref();
        entry.insert("childIsLeft".into(), json!(position == Some("left")));
        entry.insert("childIsRight".into(), json!(position == Some("right")));
        entry.insert(
            "childIsBranch".into(),
            json!(matches!(position, Some("consequent" | "alternate"))),
        );
        chain.push(Value::Object(entry));
        child_key = ancestor
            .get("keyInParent")
            .and_then(Value::as_str)
            .map(str::to_owned);
    }
    Value::Array(chain)
}

fn ancestor_fact(value: &Value, kind: &str, key_in_parent: Option<&str>) -> Value {
    let mut fact = Map::new();
    fact.insert("kind".into(), json!(kind));
    fact.insert(
        "async".into(),
        json!(value.get("async").and_then(Value::as_bool).unwrap_or(false)),
    );
    fact.insert(
        "start".into(),
        value.get("start").cloned().unwrap_or(json!(0)),
    );
    fact.insert("end".into(), value.get("end").cloned().unwrap_or(json!(0)));
    if let Some(operator) = value.get("operator").and_then(Value::as_str) {
        fact.insert("operator".into(), json!(operator));
    }
    if let Some(key) = key_in_parent {
        fact.insert("keyInParent".into(), json!(key));
    }
    if value.get("optional").and_then(Value::as_bool) == Some(true) {
        fact.insert("optional".into(), json!(true));
    }
    if matches!(kind, "ClassDeclaration" | "ClassExpression") {
        if let Some(name) = value
            .get("id")
            .and_then(|id| id.get("name"))
            .and_then(Value::as_str)
        {
            fact.insert("className".into(), json!(name));
        }
    }
    if kind.contains("Function") || kind == "CatchClause" {
        let names = parameter_names(value);
        if kind == "CatchClause" {
            if let Some(name) = names.first() {
                fact.insert("catchParamName".into(), json!(name));
            }
        }
        fact.insert(
            "paramNames".into(),
            Value::Array(names.into_iter().map(Value::String).collect()),
        );
    }
    Value::Object(fact)
}

fn parameter_names(value: &Value) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(params) = value.get("params").and_then(Value::as_array) {
        names.extend(params.iter().filter_map(binding_name));
    }
    if let Some(param) = value.get("param") {
        if let Some(name) = binding_name(param) {
            names.push(name);
        }
    }
    names
}

fn binding_name(value: &Value) -> Option<String> {
    match value.get("type").and_then(Value::as_str)? {
        "Identifier" => value.get("name").and_then(Value::as_str).map(str::to_owned),
        "AssignmentPattern" => value.get("left").and_then(binding_name),
        "RestElement" | "TSParameterProperty" => value
            .get("argument")
            .or_else(|| value.get("parameter"))
            .and_then(binding_name),
        _ => None,
    }
}

fn object_entries(value: &Value) -> Vec<(&str, &Value)> {
    value
        .as_object()
        .map(|map| {
            map.iter()
                .filter(|(key, _)| {
                    !matches!(key.as_str(), "type" | "start" | "end" | "range" | "loc")
                })
                .map(|(key, value)| (key.as_str(), value))
                .collect()
        })
        .unwrap_or_default()
}

fn normalize(value: &mut Value) {
    while let Some(inner) = parenthesized(value) {
        *value = inner;
    }
    if let Some(kind) = value.get("type").and_then(Value::as_str) {
        if matches!(
            kind,
            "NumericLiteral" | "StringLiteral" | "BooleanLiteral" | "NullLiteral" | "BigIntLiteral"
        ) {
            if let Some(object) = value.as_object_mut() {
                object.insert("type".into(), json!("Literal"));
            }
        }
    }
    match value {
        Value::Object(map) => {
            for child in map.values_mut() {
                normalize(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                normalize(item);
            }
        }
        _ => {}
    }
}

/// TS-ESTree has no parenthesized expression or type nodes.
fn parenthesized(value: &Value) -> Option<Value> {
    match value.get("type").and_then(Value::as_str) {
        Some("ParenthesizedExpression") => value.get("expression").cloned(),
        Some("TSParenthesizedType") => value.get("typeAnnotation").cloned(),
        _ => None,
    }
}

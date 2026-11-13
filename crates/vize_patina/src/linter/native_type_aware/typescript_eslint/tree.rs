//! TS-ESTree nodes for the corsa typescript-eslint rules.
mod ancestors;
mod type_parameters;
use ancestors::{ancestor_chain, ancestor_fact};
use corsa::lint::{LintNode, TextRange};
use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_estree::{CompactSerializer, ESTree};
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde_json::{Value, json};
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

#[cfg(test)]
pub(super) fn collect(source: &str, tsx: bool, listeners: &[&str]) -> Collected {
    collect_with(source, tsx, listeners, |_, _| ()).0
}

/// Parse `source` once, serialize the listener trees, and hand the same
/// parse to `then` (scope analysis the ESTree JSON cannot answer). `then`
/// does not run when the parse fails.
pub(super) fn collect_with<R: Default>(
    source: &str,
    tsx: bool,
    listeners: &[&str],
    then: impl FnOnce(&Collected, &Program<'_>) -> R,
) -> (Collected, R) {
    let allocator = Allocator::default();
    let source_type = if tsx {
        SourceType::tsx()
    } else {
        SourceType::ts()
    };
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked {
        let empty = Collected {
            program: Value::Null,
            nodes: Vec::new(),
        };
        return (empty, R::default());
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
        Slot::default(),
        &mut nodes,
    );
    let collected = Collected { program, nodes };
    let extra = then(&collected, &parsed.program);
    (collected, extra)
}

/// Where a node sits in its parent: the field name and, for list fields,
/// whether it is the last item.
#[derive(Clone, Copy, Default)]
struct Slot<'a> {
    key: Option<&'a str>,
    last: Option<bool>,
    /// For a destructuring pattern: the value it destructures (`init` of a
    /// declarator, `right` of an assignment or default).
    init: Option<&'a Value>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "the walk threads the parent context alongside the ancestor stack"
)]
fn walk(
    value: &Value,
    source: &str,
    listeners: &[&str],
    ancestors: &mut Vec<Value>,
    parent: Option<&Value>,
    parent_kind: Option<&str>,
    slot: Slot<'_>,
    out: &mut Vec<LintNode>,
) {
    let Some(kind) = value.get("type").and_then(Value::as_str) else {
        return;
    };
    let key = slot.key;
    if listeners.contains(&kind)
        && let Some(mut node) = convert(value, source, DEPTH, parent_kind, key)
    {
        if let Some(parent) = parent {
            function_naming(&mut node, parent);
        }
        if kind == "ObjectPattern" {
            stamp_pattern_init(&mut node, slot.init);
        }
        if kind == "TSTypeParameterDeclaration"
            && key == Some("typeParameters")
            && let (Some(owner), Some(owner_kind)) = (parent, parent_kind)
        {
            type_parameters::stamp(&mut node, owner, owner_kind, source);
        }
        node.fields
            .insert("__ancestorFacts".into(), Value::Array(ancestors.clone()));
        node.fields
            .insert("__ancestorChain".into(), ancestor_chain(ancestors, slot));
        if let Some(key) = key {
            node.fields.insert("__keyInParent".into(), json!(key));
        }
        out.push(node);
    }
    ancestors.push(ancestor_fact(value, kind, slot, parent));
    let init = match kind {
        "VariableDeclarator" => value.get("init"),
        "AssignmentPattern" | "AssignmentExpression" => value.get("right"),
        _ => None,
    };
    for (child_key, child) in object_entries(value) {
        let child_init = matches!(child_key, "id" | "left").then_some(init).flatten();
        match child {
            Value::Object(_) => walk(
                child,
                source,
                listeners,
                ancestors,
                Some(value),
                Some(kind),
                Slot {
                    key: Some(child_key),
                    last: None,
                    init: child_init,
                },
                out,
            ),
            Value::Array(items) => {
                let count = items.len();
                for (index, item) in items.iter().enumerate() {
                    walk(
                        item,
                        source,
                        listeners,
                        ancestors,
                        Some(value),
                        Some(kind),
                        Slot {
                            key: Some(child_key),
                            last: Some(index + 1 == count),
                            init: None,
                        },
                        out,
                    );
                }
            }
            _ => {}
        }
    }
    ancestors.pop();
}

/// Converts one ESTree value (and its subtree) into a [`LintNode`].
pub(super) fn node_of(value: &Value, source: &str, parent_kind: &str) -> Option<LintNode> {
    convert(value, source, DEPTH, Some(parent_kind), None)
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

/// How typescript-eslint names a function (`getFunctionNameWithKind`) and
/// where its head starts (`getFunctionHeadLoc`), from its parent.
fn function_naming(node: &mut LintNode, parent: &Value) {
    if !matches!(
        node.kind.as_str(),
        "FunctionDeclaration" | "FunctionExpression" | "ArrowFunctionExpression"
    ) {
        return;
    }
    let parent_kind = parent
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let member = matches!(
        parent_kind,
        "Property" | "MethodDefinition" | "PropertyDefinition"
    ) && parent.get("value").and_then(|value| value.get("start"))
        == Some(&json!(node.range.start));
    if member {
        let kind = match parent.get("kind").and_then(Value::as_str) {
            Some("constructor") => "constructor",
            Some("get") => "getter",
            Some("set") => "setter",
            _ => "method",
        };
        node.fields.insert("__functionKind".into(), json!(kind));
        if parent.get("static").and_then(Value::as_bool) == Some(true) {
            node.fields.insert("__functionStatic".into(), json!(true));
        }
        if let Some(start) = parent.get("start") {
            node.fields
                .insert("__functionHeadStart".into(), start.clone());
        }
        let computed = parent.get("computed").and_then(Value::as_bool) == Some(true);
        let key = parent.get("key");
        let name = match key.and_then(|key| key.get("type")).and_then(Value::as_str) {
            Some("Identifier") if !computed => key.and_then(|key| key.get("name")),
            Some("PrivateIdentifier") => key.and_then(|key| key.get("name")),
            Some("Literal") => key.and_then(|key| key.get("value")),
            _ => None,
        };
        let private = key.and_then(|key| key.get("type")).and_then(Value::as_str)
            == Some("PrivateIdentifier");
        if let Some(name) = name.and_then(Value::as_str) {
            let mut text = vize_l0::String::from(if private { "#" } else { "" });
            text.push_str(name);
            node.fields
                .insert("__functionName".into(), json!(text.as_str()));
        }
        return;
    }
    if node.field_str("__functionName").is_some() {
        return;
    }
    let own = node
        .child("id")
        .and_then(|id| id.field_str("name"))
        .map(str::to_owned);
    let inferred = (parent_kind == "VariableDeclarator")
        .then(|| parent.get("id"))
        .flatten()
        .filter(|id| id.get("type").and_then(Value::as_str) == Some("Identifier"))
        .and_then(|id| id.get("name"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    if let Some(name) = own.or(inferred) {
        node.fields.insert("__functionName".into(), json!(name));
    }
}

/// unbound-method checks destructured members against the initializer type.
fn stamp_pattern_init(node: &mut LintNode, init: Option<&Value>) {
    let Some(init) = init.filter(|init| init.get("type").is_some()) else {
        return;
    };
    let start = init.get("start").and_then(Value::as_u64);
    let end = init.get("end").and_then(Value::as_u64);
    let (Some(start), Some(end)) = (start, end) else {
        return;
    };
    node.fields
        .insert("__initRange".into(), json!({ "start": start, "end": end }));
    if init.get("type").and_then(Value::as_str) == Some("Identifier")
        && let Some(name) = init.get("name").and_then(Value::as_str)
    {
        node.fields.insert("__initName".into(), json!(name));
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
    if let Some(kind) = value.get("type").and_then(Value::as_str)
        && matches!(
            kind,
            "NumericLiteral" | "StringLiteral" | "BooleanLiteral" | "NullLiteral" | "BigIntLiteral"
        )
        && let Some(object) = value.as_object_mut()
    {
        object.insert("type".into(), json!("Literal"));
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

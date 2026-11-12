//! TS-ESTree nodes for the corsa typescript-eslint rules.
use corsa::lint::{LintNode, TextRange};
use oxc_allocator::Allocator;
use oxc_estree::{CompactSerializer, ESTree};
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use vize_l0::String;

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
        Slot::default(),
        &mut nodes,
    );
    Collected { program, nodes }
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

fn ancestor_chain(ancestors: &[Value], slot: Slot<'_>) -> Value {
    let mut chain = Vec::new();
    let mut child_key = slot.key.map(str::to_owned);
    let mut child_last = slot.last;
    for ancestor in ancestors.iter().rev() {
        let mut entry = Map::new();
        if let Some(kind) = ancestor.get("kind") {
            entry.insert("kind".into(), kind.clone());
        }
        if let Some(operator) = ancestor.get("operator") {
            entry.insert("operator".into(), operator.clone());
        }
        let position = child_key.as_deref();
        if let Some(position) = position {
            entry.insert("childKey".into(), json!(position));
        }
        for field in ["declare", "leftIsThisMember"] {
            if let Some(flag) = ancestor.get(field) {
                entry.insert(field.into(), flag.clone());
            }
        }
        entry.insert("childIsLeft".into(), json!(position == Some("left")));
        entry.insert("childIsRight".into(), json!(position == Some("right")));
        entry.insert(
            "childIsBranch".into(),
            json!(matches!(position, Some("consequent" | "alternate"))),
        );
        entry.insert("childIsTest".into(), json!(position == Some("test")));
        if let Some(last) = child_last {
            entry.insert("childIsLast".into(), json!(last));
        }
        chain.push(Value::Object(entry));
        child_key = ancestor
            .get("keyInParent")
            .and_then(Value::as_str)
            .map(str::to_owned);
        child_last = ancestor.get("lastInParent").and_then(Value::as_bool);
    }
    Value::Array(chain)
}

fn ancestor_fact(value: &Value, kind: &str, slot: Slot<'_>, parent: Option<&Value>) -> Value {
    let key_in_parent = slot.key;
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
    if let Some(last) = slot.last {
        fact.insert("lastInParent".into(), json!(last));
    }
    if value.get("declare").and_then(Value::as_bool) == Some(true) {
        fact.insert("declare".into(), json!(true));
    }
    if kind == "AssignmentExpression" {
        let left = value.get("left");
        let left_is_this_member = left
            .and_then(|left| left.get("type"))
            .and_then(Value::as_str)
            == Some("MemberExpression")
            && left
                .and_then(|left| left.get("object"))
                .and_then(|object| object.get("type"))
                .and_then(Value::as_str)
                == Some("ThisExpression");
        fact.insert("leftIsThisMember".into(), json!(left_is_this_member));
    }
    if kind == "SwitchStatement"
        && let Some(discriminant) = value.get("discriminant")
    {
        fact.insert(
            "discriminant".into(),
            json!({ "start": discriminant.get("start"), "end": discriminant.get("end") }),
        );
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
        if let Some(super_class) = value.get("superClass").filter(|value| !value.is_null()) {
            fact.insert(
                "superClass".into(),
                json!({ "start": super_class.get("start"), "end": super_class.get("end") }),
            );
        }
    }
    if kind.contains("Function") {
        fact.insert(
            "hasReturnType".into(),
            json!(
                value
                    .get("returnType")
                    .is_some_and(|annotation| !annotation.is_null())
            ),
        );
    }
    if kind.contains("Function") || kind == "CatchClause" {
        let names = parameter_names(value);
        if kind == "CatchClause"
            && let Some(name) = names.first()
        {
            fact.insert("catchParamName".into(), json!(name.as_str()));
        }
        fact.insert(
            "paramNames".into(),
            Value::Array(
                names
                    .into_iter()
                    .map(|name| Value::String(name.into()))
                    .collect(),
            ),
        );
        let first_param_kind = value
            .get("params")
            .and_then(Value::as_array)
            .and_then(|params| params.first())
            .and_then(|param| param.get("type"))
            .and_then(Value::as_str);
        if let Some(first_param_kind) = first_param_kind {
            fact.insert(
                "firstParamIsIdentifier".into(),
                json!(first_param_kind == "Identifier"),
            );
        }
        if let Some(parent) = parent
            && let Value::Object(extra) = call_parent_facts(value, parent)
        {
            fact.extend(extra);
        }
    }
    if kind == "TryStatement" {
        fact.insert(
            "hasFinalizer".into(),
            json!(
                value
                    .get("finalizer")
                    .is_some_and(|finalizer| !finalizer.is_null())
            ),
        );
    }
    Value::Object(fact)
}

/// Where a function sits when it is a call argument: the callee's static
/// member name, the argument index and whether a spread precedes it.
fn call_parent_facts(function: &Value, parent: &Value) -> Value {
    let mut fact = json!({});
    fill_call_parent(&mut fact, function, parent);
    fact
}

fn fill_call_parent(fact: &mut Value, function: &Value, parent: &Value) {
    let Some(fact) = fact.as_object_mut() else {
        return;
    };
    let Some(parent_kind) = parent.get("type").and_then(Value::as_str) else {
        return;
    };
    fact.insert("parentKind".into(), json!(parent_kind));
    if parent_kind != "CallExpression" {
        return;
    }
    let Some(arguments) = parent.get("arguments").and_then(Value::as_array) else {
        return;
    };
    let Some(index) = arguments.iter().position(|argument| {
        argument.get("start") == function.get("start") && argument.get("end") == function.get("end")
    }) else {
        return;
    };
    fact.insert("parentArgumentIndex".into(), json!(index));
    let spread = arguments
        .iter()
        .take(index)
        .any(|argument| argument.get("type").and_then(Value::as_str) == Some("SpreadElement"));
    fact.insert("parentLeadingSpreadArgument".into(), json!(spread));
    let callee = parent.get("callee");
    let callee = match callee
        .and_then(|callee| callee.get("type"))
        .and_then(Value::as_str)
    {
        Some("ChainExpression") => callee.and_then(|callee| callee.get("expression")),
        _ => callee,
    };
    let Some(callee) = callee else {
        return;
    };
    if callee.get("type").and_then(Value::as_str) != Some("MemberExpression") {
        return;
    }
    let computed = callee
        .get("computed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let property = callee.get("property");
    let name = if computed {
        property
            .filter(|property| property.get("type").and_then(Value::as_str) == Some("Literal"))
            .and_then(|property| property.get("value"))
            .and_then(Value::as_str)
    } else {
        property
            .and_then(|property| property.get("name"))
            .and_then(Value::as_str)
    };
    if let Some(name) = name {
        fact.insert("parentCalleePropertyName".into(), json!(name));
    }
}

fn parameter_names(value: &Value) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(params) = value.get("params").and_then(Value::as_array) {
        names.extend(params.iter().filter_map(binding_name));
    }
    if let Some(param) = value.get("param")
        && let Some(name) = binding_name(param)
    {
        names.push(name);
    }
    names
}

fn binding_name(value: &Value) -> Option<String> {
    match value.get("type").and_then(Value::as_str)? {
        "Identifier" => value.get("name").and_then(Value::as_str).map(String::from),
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

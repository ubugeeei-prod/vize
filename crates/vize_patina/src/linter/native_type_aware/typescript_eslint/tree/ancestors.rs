//! Ancestor facts and the ancestor chain stamped on each listener node.
use super::Slot;
use serde_json::{Map, Value, json};
use vize_l0::String;

pub(super) fn ancestor_chain(ancestors: &[Value], slot: Slot<'_>) -> Value {
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

pub(super) fn ancestor_fact(
    value: &Value,
    kind: &str,
    slot: Slot<'_>,
    parent: Option<&Value>,
) -> Value {
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
pub(super) fn call_parent_facts(function: &Value, parent: &Value) -> Value {
    let mut fact = json!({});
    fill_call_parent(&mut fact, function, parent);
    fact
}

pub(super) fn fill_call_parent(fact: &mut Value, function: &Value, parent: &Value) {
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

pub(super) fn parameter_names(value: &Value) -> Vec<String> {
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

pub(super) fn binding_name(value: &Value) -> Option<String> {
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

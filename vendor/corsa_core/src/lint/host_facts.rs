use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use super::{LintNode, TextRange};

pub(crate) fn prepare_node_for_rule_owned(mut node: LintNode) -> LintNode {
    apply_ancestor_facts(&mut node);
    apply_call_facts(&mut node);
    apply_symbol_facts(&mut node);
    apply_type_parameter_facts(&mut node);
    apply_return_type_facts(&mut node);
    node
}

fn apply_ancestor_facts(node: &mut LintNode) {
    let Some(ancestors) = node
        .fields
        .get("__ancestorFacts")
        .and_then(Value::as_array)
        .cloned()
    else {
        return;
    };

    if let Some(function) = ancestors
        .iter()
        .rev()
        .find(|ancestor| field_str(ancestor, "kind").is_some_and(|kind| kind.contains("Function")))
    {
        if let Some(async_value) = field_bool(function, "async") {
            insert_if_absent(
                &mut node.fields,
                "__nearestFunctionAsync",
                json!(async_value),
            );
            if node.kind == "ReturnStatement" && async_value {
                insert_if_absent(&mut node.fields, "__inAsyncScope", json!(true));
            }
        }
    }

    if let Some(class_name) = ancestors
        .iter()
        .rev()
        .find_map(|ancestor| field_str(ancestor, "className"))
    {
        insert_if_absent(&mut node.fields, "__nearestClassName", json!(class_name));
    }

    if node.kind == "ReturnStatement"
        && ancestors
            .iter()
            .any(|ancestor| try_contains(ancestor, node.range))
    {
        insert_if_absent(&mut node.fields, "__returnAwaitRequiresAwait", json!(true));
    }

    if node.kind == "CallExpression" && is_promise_executor_reject_call(node, ancestors.as_slice())
    {
        insert_if_absent(&mut node.fields, "__promiseExecutorRejectCall", json!(true));
    }
}

fn apply_call_facts(node: &mut LintNode) {
    let Some(call_facts) = node
        .fields
        .get("__callFacts")
        .and_then(Value::as_object)
        .cloned()
    else {
        return;
    };
    if let Some(expected) = call_facts.get("expectedArgumentTypeTexts") {
        insert_if_absent(
            &mut node.fields,
            "__expectedArgumentTypeTexts",
            expected.clone(),
        );
    }
    if let Some(required) = call_facts.get("explicitTypeArgumentsRequired") {
        insert_if_absent(
            &mut node.fields,
            "__explicitTypeArgumentsRequired",
            required.clone(),
        );
    }
    copy_call_fact(
        node,
        &call_facts,
        "typeArgumentRanges",
        "__typeArgumentRanges",
    );
    copy_call_fact(
        node,
        &call_facts,
        "typeArgumentListRange",
        "__typeArgumentListRange",
    );
    copy_call_fact(
        node,
        &call_facts,
        "typeParameterCount",
        "__typeParameterCount",
    );
    copy_call_fact(
        node,
        &call_facts,
        "lastTypeParameterHasDefault",
        "__lastTypeParameterHasDefault",
    );
    copy_call_fact(
        node,
        &call_facts,
        "lastTypeArgumentEqualsDefault",
        "__lastTypeArgumentEqualsDefault",
    );
    copy_call_fact(
        node,
        &call_facts,
        "lastTypeArgumentSameTypeFlagsAsDefault",
        "__lastTypeArgumentSameTypeFlagsAsDefault",
    );
    copy_call_fact(
        node,
        &call_facts,
        "lastTypeArgumentIdenticalToDefault",
        "__lastTypeArgumentIdenticalToDefault",
    );
    copy_call_fact(
        node,
        &call_facts,
        "lastTypeArgumentIsAny",
        "__lastTypeArgumentIsAny",
    );
    copy_call_fact(
        node,
        &call_facts,
        "lastTypeParameterDefaultIsAny",
        "__lastTypeParameterDefaultIsAny",
    );
}

fn apply_symbol_facts(node: &mut LintNode) {
    let Some(symbol_facts) = node.fields.get("__symbolFacts").and_then(Value::as_object) else {
        return;
    };
    if !symbol_facts
        .get("deprecated")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return;
    }
    let reason = symbol_facts
        .get("deprecationReason")
        .and_then(Value::as_str)
        .map(str::to_owned);
    insert_if_absent(&mut node.fields, "__deprecated", json!(true));
    if let Some(reason) = reason {
        insert_if_absent(&mut node.fields, "__deprecationReason", json!(reason));
    }
}

fn apply_type_parameter_facts(node: &mut LintNode) {
    let Some(type_parameter_facts) = node
        .fields
        .get("__typeParameterFacts")
        .and_then(Value::as_object)
    else {
        return;
    };
    let Some(name) = type_parameter_facts.get("name").and_then(Value::as_str) else {
        return;
    };
    let Some(owner_text) = type_parameter_facts
        .get("ownerText")
        .and_then(Value::as_str)
    else {
        return;
    };
    if count_identifier_occurrences(owner_text, name) <= 2 {
        insert_if_absent(&mut node.fields, "__hasSingleUseTypeParameter", json!(true));
    }
}

fn apply_return_type_facts(node: &mut LintNode) {
    let Some(return_type_facts) = node
        .fields
        .get("__returnTypeFacts")
        .and_then(Value::as_object)
        .cloned()
    else {
        return;
    };
    if let Some(texts) = return_type_facts.get("texts") {
        insert_if_absent(&mut node.fields, "__returnTypeTexts", texts.clone());
    }
}

fn is_promise_executor_reject_call(node: &LintNode, ancestors: &[Value]) -> bool {
    let Some(call_facts) = node.fields.get("__callFacts").and_then(Value::as_object) else {
        return false;
    };
    let Some(callee_name) = call_facts.get("calleeName").and_then(Value::as_str) else {
        return false;
    };
    let Some(function) = ancestors
        .iter()
        .rev()
        .find(|ancestor| field_str(ancestor, "kind").is_some_and(|kind| kind.contains("Function")))
    else {
        return false;
    };
    let Some(params) = function.get("paramNames").and_then(Value::as_array) else {
        return false;
    };
    if params
        .get(1)
        .and_then(Value::as_str)
        .is_none_or(|reject| reject != callee_name)
    {
        return false;
    }
    (field_str(function, "parentKind") == Some("NewExpression")
        && field_str(function, "parentCalleeName") == Some("Promise"))
        || (field_str(function, "parentParentKind") == Some("NewExpression")
            && field_str(function, "parentParentCalleeName") == Some("Promise"))
}

fn try_contains(value: &Value, range: TextRange) -> bool {
    if field_str(value, "kind") != Some("TryStatement") {
        return false;
    }
    let Some(start) = value.get("start").and_then(Value::as_u64) else {
        return false;
    };
    let Some(end) = value.get("end").and_then(Value::as_u64) else {
        return false;
    };
    start as u32 <= range.start && range.end <= end as u32
}

fn count_identifier_occurrences(text: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    text.match_indices(needle)
        .filter(|(index, _)| {
            !identifier_part_before(text, *index)
                && !identifier_part_at(text, index.saturating_add(needle.len()))
        })
        .count()
}

fn identifier_part_before(text: &str, index: usize) -> bool {
    text.get(..index)
        .and_then(|prefix| prefix.chars().next_back())
        .is_some_and(is_identifier_part)
}

fn identifier_part_at(text: &str, index: usize) -> bool {
    text.get(index..)
        .and_then(|suffix| suffix.chars().next())
        .is_some_and(is_identifier_part)
}

fn is_identifier_part(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()
}

fn field_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn field_bool(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

fn insert_if_absent(fields: &mut BTreeMap<String, Value>, key: &str, value: Value) {
    fields.entry(key.to_owned()).or_insert(value);
}

fn copy_call_fact(
    node: &mut LintNode,
    call_facts: &Map<String, Value>,
    source_key: &str,
    target_key: &str,
) {
    if let Some(value) = call_facts.get(source_key) {
        insert_if_absent(&mut node.fields, target_key, value.clone());
    }
}

//! Syntax facts the promise rules read beyond the checker: return contexts, executor rejects and object-literal method contexts.
use super::stamp_heritage;
use corsa::lint::LintNode;
use serde_json::{Value, json};
use vize_l0::String as CompactString;

/// Syntax facts the promise rules need beyond the checker: the error-handling
/// context of each `return` and the reject calls of Promise executors.
pub(in super::super) fn enrich(nodes: &mut [LintNode], program: &Value) {
    let mut rejects = Vec::new();
    let mut objects = Vec::new();
    for node in nodes.iter() {
        collect_executor_rejects(node, &mut rejects);
        collect_member_facts(node, &mut objects);
    }
    for node in nodes {
        let parent = parent_range(node);
        stamp(node, program, &rejects, &objects, parent);
    }
}

type MemberFacts = ((u32, u32), Value);

pub(super) fn collect_member_facts(node: &LintNode, out: &mut Vec<MemberFacts>) {
    if let Some(members) = node
        .fields
        .get("__contextualMemberVoidFunctions")
        .filter(|members| members.is_object())
    {
        out.push(((node.range.start, node.range.end), members.clone()));
    }
    for child in node.children.values() {
        collect_member_facts(child, out);
    }
    for children in node.child_lists.values() {
        for child in children {
            collect_member_facts(child, out);
        }
    }
}

/// Range of a root node's parent, from its ancestor facts.
pub(super) fn parent_range(node: &LintNode) -> Option<(u32, u32)> {
    let parent = node.fields.get("__ancestorFacts")?.as_array()?.last()?;
    let start = u32::try_from(parent.get("start")?.as_u64()?).ok()?;
    let end = u32::try_from(parent.get("end")?.as_u64()?).ok()?;
    Some((start, end))
}

/// Object literal methods take `__contextualVoidFunction` from their object's
/// contextual property type.
pub(super) fn stamp_method(
    node: &mut LintNode,
    objects: &[MemberFacts],
    parent: Option<(u32, u32)>,
) {
    if node.kind != "Property" || node.field_bool("method") != Some(true) {
        return;
    }
    let Some(parent) = parent else {
        return;
    };
    let Some(name) = node
        .child("key")
        .and_then(|key| key.field_str("name").or_else(|| key.field_str("value")))
        .map(CompactString::from)
    else {
        return;
    };
    let void = objects
        .iter()
        .find(|(range, _)| *range == parent)
        .and_then(|(_, members)| members.get(name.as_str()))
        .and_then(Value::as_bool);
    if let (Some(void), Some(value)) = (void, node.children.get_mut("value")) {
        value
            .fields
            .insert("__contextualVoidFunction".into(), json!(void));
    }
}

pub(super) fn stamp(
    node: &mut LintNode,
    program: &Value,
    rejects: &[(u32, u32)],
    objects: &[MemberFacts],
    parent: Option<(u32, u32)>,
) {
    stamp_method(node, objects, parent);
    stamp_heritage(node);
    if node.kind == "ReturnStatement"
        && !node.fields.contains_key("__returnAwaitRequiresAwait")
        && let Some(affects) = return_affects_error_handling(program, node)
    {
        node.fields
            .insert("__returnAwaitRequiresAwait".into(), json!(affects));
    }
    if node.kind == "CallExpression" && rejects.contains(&(node.range.start, node.range.end)) {
        node.fields
            .insert("__promiseExecutorRejectCall".into(), json!(true));
    }
    let range = Some((node.range.start, node.range.end));
    for child in node.children.values_mut() {
        stamp(child, program, rejects, objects, range);
    }
    for children in node.child_lists.values_mut() {
        for child in children {
            stamp(child, program, rejects, objects, range);
        }
    }
}

/// `new P((resolve, reject) => ...)` where `P` is a Promise constructor: every
/// call whose callee is the `reject` parameter.
pub(super) fn collect_executor_rejects(node: &LintNode, out: &mut Vec<(u32, u32)>) {
    if node.kind == "NewExpression" && is_promise_constructor(node) {
        let executor = node
            .child_list("arguments")
            .and_then(|arguments| arguments.first())
            .filter(|executor| {
                matches!(
                    executor.kind.as_str(),
                    "ArrowFunctionExpression" | "FunctionExpression"
                )
            });
        if let Some(executor) = executor {
            let reject = executor
                .child_list("params")
                .and_then(|params| params.get(1))
                .filter(|param| param.kind == "Identifier")
                .and_then(|param| param.field_str("name"));
            if let Some(reject) = reject {
                for param in executor.child_list("params").unwrap_or_default() {
                    collect_calls_to(param, reject, out);
                }
                if let Some(body) = executor.child("body") {
                    collect_calls_to(body, reject, out);
                }
            }
        }
    }
    for child in node.children.values() {
        collect_executor_rejects(child, out);
    }
    for children in node.child_lists.values() {
        for child in children {
            collect_executor_rejects(child, out);
        }
    }
}

pub(super) fn is_promise_constructor(node: &LintNode) -> bool {
    let mut callee = node.child("callee");
    while let Some(current) = callee.filter(|callee| callee.kind == "ChainExpression") {
        callee = current.child("expression");
    }
    callee
        .and_then(|callee| callee.fields.get("__promiseFacts"))
        .and_then(|facts| facts.get("promiseConstructor"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// Calls of `name` that still refer to the executor parameter.
pub(super) fn collect_calls_to(node: &LintNode, name: &str, out: &mut Vec<(u32, u32)>) {
    if is_function(node) && binds_parameter(node, name) {
        return;
    }
    if node.kind == "BlockStatement" && declares(node, name) {
        return;
    }
    if node.kind == "CallExpression"
        && node.child("callee").is_some_and(|callee| {
            callee.kind == "Identifier" && callee.field_str("name") == Some(name)
        })
    {
        out.push((node.range.start, node.range.end));
    }
    for child in node.children.values() {
        collect_calls_to(child, name, out);
    }
    for children in node.child_lists.values() {
        for child in children {
            collect_calls_to(child, name, out);
        }
    }
}

pub(super) fn is_function(node: &LintNode) -> bool {
    matches!(
        node.kind.as_str(),
        "ArrowFunctionExpression" | "FunctionExpression" | "FunctionDeclaration"
    )
}

pub(super) fn binds_parameter(function: &LintNode, name: &str) -> bool {
    function
        .child_list("params")
        .unwrap_or_default()
        .iter()
        .any(|param| binds(param, name))
        || (function.kind != "ArrowFunctionExpression" && name == "arguments")
}

pub(super) fn binds(pattern: &LintNode, name: &str) -> bool {
    match pattern.kind.as_str() {
        "Identifier" => pattern.field_str("name") == Some(name),
        "AssignmentPattern" => pattern.child("left").is_some_and(|left| binds(left, name)),
        "RestElement" => pattern
            .child("argument")
            .is_some_and(|argument| binds(argument, name)),
        "ArrayPattern" => pattern
            .child_list("elements")
            .unwrap_or_default()
            .iter()
            .any(|element| binds(element, name)),
        "ObjectPattern" => pattern
            .child_list("properties")
            .unwrap_or_default()
            .iter()
            .any(|property| {
                property
                    .child("value")
                    .or_else(|| property.child("argument"))
                    .is_some_and(|value| binds(value, name))
            }),
        _ => false,
    }
}

pub(super) fn declares(block: &LintNode, name: &str) -> bool {
    block
        .child_list("body")
        .unwrap_or_default()
        .iter()
        .any(|statement| match statement.kind.as_str() {
            "VariableDeclaration" => statement
                .child_list("declarations")
                .unwrap_or_default()
                .iter()
                .any(|declarator| declarator.child("id").is_some_and(|id| binds(id, name))),
            "FunctionDeclaration" | "ClassDeclaration" => statement
                .child("id")
                .is_some_and(|id| id.field_str("name") == Some(name)),
            _ => false,
        })
}

/// return-await's `affectsExplicitErrorHandling || affectsExplicitResourceManagement`.
pub(super) fn return_affects_error_handling(program: &Value, node: &LintNode) -> Option<bool> {
    let mut path = Vec::new();
    if !find_path(program, None, node.range.start, node.range.end, &mut path) {
        return None;
    }
    let expression_start = node
        .child("argument")
        .map_or(node.range.start, |argument| argument.range.start);
    // `path` ends at the return statement; walk its ancestors outward up to
    // the enclosing function. A try block always affects control flow, a
    // catch clause only when a finally block follows; otherwise the search
    // continues past the try statement.
    for index in (0..path.len().saturating_sub(1)).rev() {
        let (ancestor, _) = path.get(index)?;
        let (_, child_key) = path.get(index + 1)?;
        let kind = ancestor
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if matches!(
            kind,
            "FunctionDeclaration" | "FunctionExpression" | "ArrowFunctionExpression"
        ) {
            break;
        }
        if has_using_before(ancestor, expression_start) {
            return Some(true);
        }
        if kind == "TryStatement" {
            let has_finalizer = ancestor
                .get("finalizer")
                .is_some_and(|finalizer| !finalizer.is_null());
            match *child_key {
                Some("block") => return Some(true),
                Some("handler") if has_finalizer => return Some(true),
                _ => {}
            }
        }
    }
    Some(false)
}

pub(super) fn has_using_before(block: &Value, before: u32) -> bool {
    let statements = block
        .get("body")
        .and_then(Value::as_array)
        .or_else(|| block.get("consequent").and_then(Value::as_array));
    statements.into_iter().flatten().any(|statement| {
        statement.get("type").and_then(Value::as_str) == Some("VariableDeclaration")
            && matches!(
                statement.get("kind").and_then(Value::as_str),
                Some("using" | "await using")
            )
            && statement
                .get("declarations")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .any(|declarator| {
                    declarator
                        .get("end")
                        .and_then(Value::as_u64)
                        .is_some_and(|end| end < u64::from(before))
                })
    })
}

/// Path from the root to the node with `start..end` and a `ReturnStatement`
/// type, as `(node, key in parent)` pairs.
pub(super) fn find_path<'a>(
    value: &'a Value,
    key: Option<&'a str>,
    start: u32,
    end: u32,
    path: &mut Vec<(&'a Value, Option<&'a str>)>,
) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    let node_start = object.get("start").and_then(Value::as_u64);
    let node_end = object.get("end").and_then(Value::as_u64);
    if let (Some(node_start), Some(node_end)) = (node_start, node_end)
        && (node_start > u64::from(start) || node_end < u64::from(end))
    {
        return false;
    }
    path.push((value, key));
    if object.get("type").and_then(Value::as_str) == Some("ReturnStatement")
        && node_start == Some(u64::from(start))
        && node_end == Some(u64::from(end))
    {
        return true;
    }
    for (child_key, child) in object {
        let found = match child {
            Value::Object(_) => find_path(child, Some(child_key.as_str()), start, end, path),
            Value::Array(items) => items
                .iter()
                .any(|item| find_path(item, Some(child_key.as_str()), start, end, path)),
            _ => false,
        };
        if found {
            return true;
        }
    }
    path.pop();
    false
}

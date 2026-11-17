//! The typescript-eslint checks corsa does not ship.
use super::tree::Collected;
use corsa::lint::LintNode;
use serde_json::Value;
use vize_l0::String as CompactString;

pub(super) struct Gap {
    pub rule: &'static str,
    pub message: CompactString,
    pub start: u32,
    pub end: u32,
}

/// Node kinds `no-generated-empty-object-type` listens to.
pub(super) const EMPTY_OBJECT_LISTENERS: [&str; 2] = ["TSIntersectionType", "TSTypeReference"];

/// `misused` holds no-misused-promises' configured options when it is on.
pub(super) fn reports(collected: &Collected, misused: Option<Option<&Value>>) -> Vec<Gap> {
    let checks_variables = misused.is_some_and(checks_void_return_variables);
    let mut gaps = Vec::new();
    for node in &collected.nodes {
        if let Some(gap) = generated_empty(node) {
            gaps.push(gap);
        }
        unsafe_enums(node, &mut gaps);
        if checks_variables && let Some(gap) = thenable_dispose(node) {
            gaps.push(gap);
        }
    }
    gaps
}

/// `checksVoidReturn` (default on), as a boolean or with its `variables` key.
fn checks_void_return_variables(options: Option<&Value>) -> bool {
    match options
        .and_then(|options| options.get(0))
        .and_then(|options| options.get("checksVoidReturn"))
    {
        Some(Value::Bool(enabled)) => *enabled,
        Some(Value::Object(checks)) => checks
            .get("variables")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        _ => true,
    }
}

fn dispose_fact(node: Option<&LintNode>, key: &str) -> bool {
    node.and_then(|node| node.fields.get("__promiseFacts"))
        .and_then(|facts| facts.get(key))
        .and_then(Value::as_bool)
        == Some(true)
}

/// typescript-eslint `no-misused-promises`' `checkVariableDeclaration` on
/// `[Symbol.dispose]`: a `using` declaration whose initializer disposes with a
/// thenable, or an annotated variable whose type disposes with `void` while
/// its initializer disposes with a thenable.
fn thenable_dispose(node: &LintNode) -> Option<Gap> {
    if node.kind != "VariableDeclarator" {
        return None;
    }
    let init = node.child("init")?;
    if !dispose_fact(Some(init), "disposeThenable") {
        return None;
    }
    let id = node.child("id");
    let using = super::facts::declaration_kind(node) == Some("using");
    let annotated = id.is_some_and(|id| id.child("typeAnnotation").is_some());
    if !using && !(annotated && dispose_fact(id, "disposeVoid")) {
        return None;
    }
    Some(Gap {
        rule: "type/no-misused-promises",
        message:
            "Promise-returning function provided to variable where a void return was expected."
                .into(),
        start: init.range.start,
        end: init.range.end,
    })
}

/// typescript-eslint `no-generated-empty-object-type`: an intersection, or a
/// type reference with type arguments outside an intersection, whose type is
/// (or holds) the empty object type. `__emptyObjectType` is the checker's
/// verdict on the type (see `corsa_session::type_node_facts`).
fn generated_empty(node: &LintNode) -> Option<Gap> {
    let inspected = match node.kind.as_str() {
        "TSIntersectionType" => true,
        // The intersection itself is checked, so its members are not.
        "TSTypeReference" => {
            node.child("typeArguments").is_some()
                && node.field_str("__parentKind") != Some("TSIntersectionType")
        }
        _ => false,
    };
    if !inspected || node.field_bool("__emptyObjectType") != Some(true) {
        return None;
    }
    Some(Gap {
        rule: "type/no-generated-empty-object-type",
        message:
            "This type resolves to `{}`, the empty object type. This was likely not intentional."
                .into(),
        start: node.range.start,
        end: node.range.end,
    })
}

fn unsafe_enums(node: &LintNode, gaps: &mut Vec<Gap>) {
    for report in node
        .fields
        .get("__unsafeEnumReports")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(start) = report
            .get("start")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
        else {
            continue;
        };
        let Some(end) = report
            .get("end")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
        else {
            continue;
        };
        let Some(message) = report.get("message").and_then(Value::as_str) else {
            continue;
        };
        gaps.push(Gap {
            rule: "type/no-unsafe-enum-assignment",
            message: CompactString::from(message),
            start,
            end,
        });
    }
}

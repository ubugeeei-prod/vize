//! The typescript-eslint check corsa does not ship.
use super::tree::Collected;
use corsa::lint::LintNode;
use vize_l0::String as CompactString;

pub(super) struct Gap {
    pub rule: &'static str,
    pub message: CompactString,
    pub start: u32,
    pub end: u32,
}

/// Node kinds `no-generated-empty-object-type` listens to.
pub(super) const EMPTY_OBJECT_LISTENERS: [&str; 2] = ["TSIntersectionType", "TSTypeReference"];

pub(super) fn reports(collected: &Collected) -> Vec<Gap> {
    let mut gaps = Vec::new();
    for node in &collected.nodes {
        if let Some(gap) = generated_empty(node) {
            gaps.push(gap);
        }
        unsafe_enums(node, &mut gaps);
    }
    gaps
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
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(start) = report
            .get("start")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
        else {
            continue;
        };
        let Some(end) = report
            .get("end")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
        else {
            continue;
        };
        let Some(message) = report.get("message").and_then(serde_json::Value::as_str) else {
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

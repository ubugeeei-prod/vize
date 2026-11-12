//! The typescript-eslint check corsa does not ship.
use super::tree::Collected;
use corsa::lint::LintNode;

pub(super) struct Gap {
    pub rule: &'static str,
    pub message: &'static str,
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
        message: "This type resolves to `{}`, the empty object type. This was likely not intentional.",
        start: node.range.start,
        end: node.range.end,
    })
}

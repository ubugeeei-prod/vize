//! Member types reached by destructuring patterns (`const { a: [b] } = value`),
//! which `no-unsafe-assignment` checks per target.
use super::checker::ListenerFacts;
use crate::linter::corsa_session::PathStep;
use corsa::lint::LintNode;
use serde_json::Value;
use vize_l0::FxHashMap;
use vize_l0::String as CompactString;

/// The `(sender, pattern)` of a destructuring assignment.
pub(super) fn destructuring(node: &LintNode) -> Option<(&LintNode, &LintNode)> {
    let (pattern, sender) = match node.kind.as_str() {
        "VariableDeclarator" => (node.child("id")?, node.child("init")?),
        "AssignmentExpression" | "AssignmentPattern" => (node.child("left")?, node.child("right")?),
        _ => return None,
    };
    matches!(pattern.kind.as_str(), "ObjectPattern" | "ArrayPattern").then_some((sender, pattern))
}

/// Every pattern target with its path of member steps from the sender.
pub(super) fn pattern_paths(
    pattern: &LintNode,
    prefix: &mut Vec<PathStep>,
    out: &mut Vec<((u32, u32), Vec<PathStep>)>,
) {
    let mut visit = |step: PathStep, target: &LintNode, out: &mut Vec<_>| {
        prefix.push(step);
        out.push(((target.range.start, target.range.end), prefix.clone()));
        if matches!(target.kind.as_str(), "ObjectPattern" | "ArrayPattern") {
            pattern_paths(target, prefix, out);
        }
        prefix.pop();
    };
    match pattern.kind.as_str() {
        "ObjectPattern" => {
            for property in pattern.child_list("properties").unwrap_or(&[]) {
                let (Some(name), Some(value)) = (static_key(property), property.child("value"))
                else {
                    continue;
                };
                visit(PathStep::Property(name), value, out);
            }
        }
        "ArrayPattern" => {
            for (index, element) in pattern
                .child_list("elements")
                .unwrap_or(&[])
                .iter()
                .enumerate()
            {
                if element.kind != "RestElement" {
                    visit(PathStep::Element(index), element, out);
                }
            }
        }
        _ => {}
    }
}

/// typescript-eslint's static key of a destructured property.
fn static_key(property: &LintNode) -> Option<CompactString> {
    if property.kind != "Property" {
        return None;
    }
    let key = property.child("key")?;
    let computed = property.field_bool("computed") == Some(true);
    match key.kind.as_str() {
        "Identifier" if !computed => key.field_str("name").map(CompactString::new),
        "Literal" => key
            .field_stringish("value")
            .map(|value| CompactString::new(value.as_str())),
        "TemplateLiteral" if computed => {
            let quasis = key.child_list("quasis")?;
            match quasis {
                [quasi] => quasi
                    .fields
                    .get("value")
                    .and_then(|value| value.get("cooked"))
                    .and_then(Value::as_str)
                    .map(CompactString::new),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Stamps `__destructuredType` on the pattern targets of a destructuring
/// listener.
pub(super) fn stamp_destructuring(node: &mut LintNode, plain: &ListenerFacts<'_>) {
    let Some((sender, pattern)) = destructuring(node) else {
        return;
    };
    let mut targets = Vec::new();
    pattern_paths(pattern, &mut Vec::new(), &mut targets);
    let Some(fact) = plain.get(&(sender.range.start, sender.range.end)) else {
        return;
    };
    let ranges = targets
        .iter()
        .map(|(range, _)| *range)
        .zip(fact.paths.iter())
        .filter_map(|(range, detail)| Some((range, detail.clone()?)))
        .collect::<FxHashMap<_, _>>();
    let key = if node.kind == "VariableDeclarator" {
        "id"
    } else {
        "left"
    };
    if let Some(pattern) = node.children.get_mut(key) {
        mark_targets(pattern, &ranges);
    }
}

fn mark_targets(node: &mut LintNode, details: &FxHashMap<(u32, u32), Value>) {
    for child in node.children.values_mut() {
        if let Some(detail) = details.get(&(child.range.start, child.range.end)) {
            child
                .fields
                .insert("__destructuredType".into(), detail.clone());
        }
        mark_targets(child, details);
    }
    for children in node.child_lists.values_mut() {
        for child in children {
            if let Some(detail) = details.get(&(child.range.start, child.range.end)) {
                child
                    .fields
                    .insert("__destructuredType".into(), detail.clone());
            }
            mark_targets(child, details);
        }
    }
}

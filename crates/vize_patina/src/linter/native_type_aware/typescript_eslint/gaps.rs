//! The typescript-eslint check corsa does not ship.
use super::tree::Collected;
use corsa::lint::LintNode;

pub(super) struct Gap {
    pub rule: &'static str,
    pub message: &'static str,
    pub start: u32,
    pub end: u32,
}

pub(super) fn reports(collected: &Collected) -> Vec<Gap> {
    let mut gaps = Vec::new();
    for node in &collected.nodes {
        if node.kind == "TSTypeAliasDeclaration" {
            if let Some(gap) = generated_empty(node) {
                gaps.push(gap);
            }
        }
    }
    gaps
}

fn generated_empty(node: &LintNode) -> Option<Gap> {
    let annotation = node.child("typeAnnotation")?;
    let inner = annotation.child("typeAnnotation").unwrap_or(annotation);
    if inner.kind == "TSTypeLiteral" {
        return None;
    }
    let empty = inner
        .type_texts
        .iter()
        .chain(
            node.child("id")
                .map(|id| id.type_texts.as_slice())
                .into_iter()
                .flatten(),
        )
        .any(|text| empty_object_text(text))
        || omit_drops_every_key(inner);
    if !empty {
        return None;
    }
    Some(Gap {
        rule: "type/no-generated-empty-object-type",
        message: "Do not generate an empty object type.",
        start: inner.range.start,
        end: inner.range.end,
    })
}

fn empty_object_text(text: &str) -> bool {
    let compact: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    compact == "{}" || compact == "Record<string,never>" || compact == "Record<never,never>"
}

fn omit_drops_every_key(node: &LintNode) -> bool {
    if node.kind != "TSTypeReference" {
        return false;
    }
    let name = node
        .child("typeName")
        .and_then(|type_name| type_name.field_str("name"));
    if name != Some("Omit") {
        return false;
    }
    let Some(params) = node
        .child("typeArguments")
        .or_else(|| node.child("typeParameters"))
        .and_then(|list| list.child_list("params"))
    else {
        return false;
    };
    if params.len() < 2 {
        return false;
    }
    let keys = object_keys(&params[0]);
    if keys.is_empty() {
        return false;
    }
    let dropped = literal_keys(&params[1]);
    keys.iter()
        .all(|key| dropped.iter().any(|item| item == key))
}

fn object_keys(node: &LintNode) -> Vec<String> {
    if node.kind != "TSTypeLiteral" {
        return Vec::new();
    }
    node.child_list("members")
        .unwrap_or(&[])
        .iter()
        .filter_map(|member| {
            member
                .child("key")
                .and_then(|key| key.field_str("name"))
                .map(str::to_owned)
        })
        .collect()
}

fn literal_keys(node: &LintNode) -> Vec<String> {
    match node.kind.as_str() {
        "TSUnionType" => node
            .child_list("types")
            .unwrap_or(&[])
            .iter()
            .flat_map(literal_keys)
            .collect(),
        "TSLiteralType" => node
            .child("literal")
            .and_then(|literal| literal.field_str("value"))
            .map(|value| vec![value.to_owned()])
            .unwrap_or_default(),
        "Literal" => node
            .field_str("value")
            .map(|value| vec![value.to_owned()])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

//! Host facts for `no-unnecessary-type-parameters`.
//!
//! Corsa owns the rule verdict. This module supplies the signature-local
//! symbol observations that its Rust rule consumes.
use corsa::lint::LintNode;
use serde_json::{Value, json};

pub(super) fn enrich(source: &str, nodes: &mut [LintNode], program: &Value) {
    for node in nodes {
        if node.kind == "TSTypeParameterDeclaration" {
            enrich_declaration(source, node, program);
        }
    }
}

fn enrich_declaration(source: &str, declaration: &mut LintNode, program: &Value) {
    let Some((owner_kind, owner_start, owner_end)) = owner(declaration) else {
        return;
    };
    let Some(owner) = find_node(program, owner_kind, owner_start, owner_end) else {
        return;
    };
    declaration
        .fields
        .insert("__ownerKind".into(), json!(owner_kind));

    let Some(params) = declaration.child_lists.get_mut("params") else {
        return;
    };
    for param in params {
        let Some(name) = param
            .child("name")
            .and_then(|name| name.field_str("name"))
            .or_else(|| param.field_str("name"))
            .map(str::to_owned)
        else {
            continue;
        };
        let mut references = Vec::new();
        collect_references(owner, &name, false, true, &mut references);
        references.retain(|(start, end)| *start < param.range.start || *end > param.range.end);
        param
            .fields
            .insert("__typeParameterName".into(), json!(name));
        param.fields.insert(
            "__typeParameterUsageCount".into(),
            json!(references.len() + 1),
        );
        param
            .fields
            .insert("__repeatedInAst".into(), json!(references.len() >= 2));
        param.fields.insert(
            "__references".into(),
            Value::Array(
                references
                    .iter()
                    .map(|(start, end)| json!({ "start": start, "end": end }))
                    .collect(),
            ),
        );
        let constraint = param.child("constraint").and_then(|constraint| {
            source
                .get(constraint.range.start as usize..constraint.range.end as usize)
                .map(|text| {
                    (
                        text.to_owned(),
                        matches!(
                            constraint.kind.as_str(),
                            "TSArrayType"
                                | "TSIndexedAccessType"
                                | "TSIntersectionType"
                                | "TSUnionType"
                        ),
                    )
                })
        });
        if let Some((text, complex)) = constraint {
            param.fields.insert("__constraintText".into(), json!(text));
            param
                .fields
                .insert("__constraintIsComplex".into(), json!(complex));
        }
    }
}

fn owner(declaration: &LintNode) -> Option<(&str, u32, u32)> {
    declaration
        .fields
        .get("__ancestorFacts")?
        .as_array()?
        .iter()
        .rev()
        .find_map(|ancestor| {
            let kind = ancestor.get("kind")?.as_str()?;
            is_signature_owner(kind).then(|| {
                Some((
                    kind,
                    u32::try_from(ancestor.get("start")?.as_u64()?).ok()?,
                    u32::try_from(ancestor.get("end")?.as_u64()?).ok()?,
                ))
            })?
        })
}

fn is_signature_owner(kind: &str) -> bool {
    kind.contains("Function")
        || matches!(
            kind,
            "ClassDeclaration"
                | "ClassExpression"
                | "MethodDefinition"
                | "TSCallSignatureDeclaration"
                | "TSConstructSignatureDeclaration"
                | "TSDeclareFunction"
                | "TSFunctionType"
                | "TSInterfaceDeclaration"
                | "TSMethodSignature"
                | "TSTypeAliasDeclaration"
        )
}

fn find_node<'a>(value: &'a Value, kind: &str, start: u32, end: u32) -> Option<&'a Value> {
    if value.get("type").and_then(Value::as_str) == Some(kind)
        && value.get("start").and_then(Value::as_u64) == Some(u64::from(start))
        && value.get("end").and_then(Value::as_u64) == Some(u64::from(end))
    {
        return Some(value);
    }
    match value {
        Value::Object(fields) => fields
            .values()
            .find_map(|child| find_node(child, kind, start, end)),
        Value::Array(items) => items
            .iter()
            .find_map(|child| find_node(child, kind, start, end)),
        _ => None,
    }
}

fn collect_references(
    value: &Value,
    name: &str,
    type_context: bool,
    root: bool,
    out: &mut Vec<(u32, u32)>,
) {
    let kind = value.get("type").and_then(Value::as_str);
    if !root
        && kind.is_some_and(|kind| {
            kind.contains("Function") || matches!(kind, "ClassDeclaration" | "ClassExpression")
        })
    {
        return;
    }
    if kind == Some("Identifier")
        && type_context
        && value.get("name").and_then(Value::as_str) == Some(name)
        && let (Some(start), Some(end)) = (
            value.get("start").and_then(Value::as_u64),
            value.get("end").and_then(Value::as_u64),
        )
        && let (Ok(start), Ok(end)) = (u32::try_from(start), u32::try_from(end))
    {
        out.push((start, end));
    }
    let next_type_context = type_context || kind.is_some_and(|kind| kind.starts_with("TS"));
    match value {
        Value::Object(fields) => {
            for (key, child) in fields {
                if key == "body"
                    && child.get("type").and_then(Value::as_str) == Some("BlockStatement")
                {
                    continue;
                }
                let child_type_context = next_type_context
                    || matches!(
                        key.as_str(),
                        "typeAnnotation"
                            | "returnType"
                            | "typeArguments"
                            | "typeParameters"
                            | "superTypeArguments"
                            | "implements"
                    );
                collect_references(child, name, child_type_context, false, out);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_references(child, name, next_type_context, false, out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use corsa::lint::run_default_type_aware_rule_owned;

    use super::enrich;
    use crate::linter::native_type_aware::typescript_eslint::tree;

    #[test]
    fn reports_a_single_use_and_keeps_a_relational_parameter() {
        let source = "function logValue<T>(value: T): void {}\nfunction identity<T>(value: T): T { return value; }\n";
        let mut collected = tree::collect(source, false, &["TSTypeParameterDeclaration"]);
        enrich(source, &mut collected.nodes, &collected.program);
        let reports = collected
            .nodes
            .into_iter()
            .flat_map(|node| {
                run_default_type_aware_rule_owned("no-unnecessary-type-parameters", node)
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();
        assert_eq!(reports.len(), 1, "{reports:#?}");
        assert_eq!(
            source.get(reports[0].range.start as usize..reports[0].range.end as usize),
            Some("T")
        );
    }
}

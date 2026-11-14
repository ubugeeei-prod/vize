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
    if !root && declares_type_parameter(value, name) {
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
                if should_skip_child(
                    kind,
                    fields
                        .get("computed")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    key,
                ) {
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

fn declares_type_parameter(value: &Value, name: &str) -> bool {
    value
        .get("typeParameters")
        .and_then(|parameters| parameters.get("params"))
        .and_then(Value::as_array)
        .is_some_and(|parameters| {
            parameters
                .iter()
                .any(|parameter| type_parameter_name(parameter) == Some(name))
        })
        || (value.get("type").and_then(Value::as_str) == Some("TSMappedType")
            && value
                .get("key")
                .is_some_and(|parameter| type_parameter_name(parameter) == Some(name)))
}

fn type_parameter_name(parameter: &Value) -> Option<&str> {
    parameter
        .get("name")
        .and_then(|name| name.as_str().or_else(|| name.get("name")?.as_str()))
}

fn should_skip_child(owner_kind: Option<&str>, computed: bool, key: &str) -> bool {
    if key == "body" && owner_kind.is_some_and(|kind| kind.contains("Function")) {
        return true;
    }
    key == "key"
        && !computed
        && owner_kind.is_some_and(|kind| {
            matches!(
                kind,
                "AccessorProperty"
                    | "MethodDefinition"
                    | "Property"
                    | "PropertyDefinition"
                    | "TSMethodSignature"
                    | "TSPropertySignature"
            )
        })
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
            reports[0].range.start as usize,
            source.find("logValue<T>").unwrap() + "logValue<".len()
        );
        assert_eq!(
            source.get(reports[0].range.start as usize..reports[0].range.end as usize),
            Some("T")
        );
    }

    #[test]
    fn collects_nested_signatures_but_not_shadowed_type_parameters() {
        let source = concat!(
            "function wrap<T>(value: T): () => T { return () => value; }\n",
            "class Box<T> { get(value: T): T { return value; } }\n",
            "function outer<T>(value: T): <T>(inner: T) => T { return inner => inner; }\n",
        );
        let reports = report_ranges(source);
        assert_eq!(reports.len(), 1, "{reports:#?}");
        assert_eq!(
            reports[0].0,
            source.find("outer<T>").unwrap() + "outer<".len()
        );
    }

    #[test]
    fn excludes_property_keys_and_mapped_type_binders() {
        let source = concat!(
            "function property<T>(value: { T: T }): void {}\n",
            "function mapped<T>(value: { [T in 'x']: T }): void {}\n",
        );
        let reports = report_ranges(source);
        assert_eq!(reports.len(), 2, "{reports:#?}");
    }

    #[test]
    fn excludes_concise_arrow_function_bodies() {
        let source = "const log = <T>(value: T): void => void (value as T);\n";
        let reports = report_ranges(source);
        assert_eq!(reports.len(), 1, "{reports:#?}");
    }

    fn report_ranges(source: &str) -> Vec<(usize, usize)> {
        let mut collected = tree::collect(source, false, &["TSTypeParameterDeclaration"]);
        enrich(source, &mut collected.nodes, &collected.program);
        collected
            .nodes
            .into_iter()
            .flat_map(|node| {
                run_default_type_aware_rule_owned("no-unnecessary-type-parameters", node)
                    .unwrap_or_default()
            })
            .map(|report| (report.range.start as usize, report.range.end as usize))
            .collect()
    }
}

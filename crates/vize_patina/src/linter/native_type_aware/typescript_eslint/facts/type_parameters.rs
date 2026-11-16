//! Host facts for `no-unnecessary-type-parameters`.
//!
//! Corsa owns the rule verdict. This module supplies the signature-local
//! symbol observations that its Rust rule consumes.
use crate::linter::corsa_session::ExpressionFact;
use corsa::lint::LintNode;
use serde_json::{Value, json};
use vize_l0::{FxHashMap, FxHashSet, String};

pub(super) fn prepare(source: &str, nodes: &mut [LintNode], program: &Value) {
    let merged = merged_owners(program);
    for node in nodes {
        if node.kind == "TSTypeParameterDeclaration" {
            enrich_declaration(source, node, program, &merged);
        }
    }
}

fn enrich_declaration(
    source: &str,
    declaration: &mut LintNode,
    program: &Value,
    merged: &MergedOwners,
) {
    let Some((owner_kind, owner_start, owner_end)) =
        owner(declaration).map(|(kind, start, end)| (kind.to_owned(), start, end))
    else {
        return;
    };
    let Some(owner) = find_node(program, owner_kind.as_str(), owner_start, owner_end) else {
        return;
    };
    declaration
        .fields
        .insert("__ownerKind".into(), json!(owner_kind.as_str()));
    declaration.fields.insert(
        "__typeParameterOwnerRange".into(),
        json!({ "start": owner_start, "end": owner_end }),
    );
    let usage_queries = usage_queries(owner, owner_kind.as_str());
    let merged_owner = merged.contains(&(owner_start, owner_end));

    let Some(params) = declaration.child_lists.get_mut("params") else {
        return;
    };
    let mut needs_checker = false;
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
        let base_usage_count = references.len() + 1;
        let repeated = merged_owner
            || references.len() >= 2
            || reference_implies_multiple(owner, &name, owner_kind.as_str(), false, false, true);
        needs_checker |= !repeated && !usage_queries.is_empty();
        if let Some(reference) = inferred_return_reference(owner, &name) {
            references.push(reference);
        }
        param
            .fields
            .insert("__typeParameterName".into(), json!(name));
        param.fields.insert(
            "__typeParameterUsageCount".into(),
            json!(references.len() + 1),
        );
        param.fields.insert(
            "__typeParameterBaseUsageCount".into(),
            json!(base_usage_count),
        );
        param
            .fields
            .insert("__repeatedInAst".into(), json!(repeated));
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
    if needs_checker {
        declaration.fields.insert(
            "__typeParameterUsageQueries".into(),
            Value::Array(usage_queries),
        );
    }
}

type MergedOwners = FxHashSet<(u32, u32)>;

fn merged_owners(program: &Value) -> MergedOwners {
    let mut interfaces = FxHashSet::default();
    let mut value_references = FxHashSet::default();
    let mut candidates = Vec::new();
    collect_merged_owner_candidates(
        program,
        None,
        &mut interfaces,
        &mut value_references,
        &mut candidates,
    );
    candidates
        .into_iter()
        .filter_map(|(range, name)| {
            (interfaces.contains(&name) && value_references.contains(&name)).then_some(range)
        })
        .collect()
}

fn collect_merged_owner_candidates(
    value: &Value,
    enclosing_variable: Option<&str>,
    interfaces: &mut FxHashSet<String>,
    value_references: &mut FxHashSet<String>,
    candidates: &mut Vec<((u32, u32), String)>,
) {
    let kind = value.get("type").and_then(Value::as_str);
    if kind == Some("TSInterfaceDeclaration")
        && let Some(name) = value
            .get("id")
            .and_then(|id| id.get("name"))
            .and_then(Value::as_str)
    {
        interfaces.insert(name.into());
    }
    if matches!(kind, Some("ClassDeclaration" | "ClassExpression"))
        && let Some(name) = value
            .get("superClass")
            .and_then(|super_class| super_class.get("name"))
            .and_then(Value::as_str)
    {
        value_references.insert(name.into());
    }

    let enclosing_variable = if kind == Some("VariableDeclarator") {
        value
            .get("id")
            .and_then(|id| id.get("name"))
            .and_then(Value::as_str)
            .or(enclosing_variable)
    } else {
        enclosing_variable
    };
    let candidate_name = if matches!(kind, Some("ClassDeclaration" | "ClassExpression")) {
        value
            .get("id")
            .and_then(|id| id.get("name"))
            .and_then(Value::as_str)
    } else if matches!(
        kind,
        Some("TSConstructorType" | "TSConstructSignatureDeclaration")
    ) {
        enclosing_variable
    } else {
        None
    };
    if let Some((range, name)) = node_range(value).zip(candidate_name) {
        candidates.push((range, name.into()));
    }

    match value {
        Value::Object(fields) => {
            for child in fields.values() {
                collect_merged_owner_candidates(
                    child,
                    enclosing_variable,
                    interfaces,
                    value_references,
                    candidates,
                );
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_merged_owner_candidates(
                    child,
                    enclosing_variable,
                    interfaces,
                    value_references,
                    candidates,
                );
            }
        }
        _ => {}
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
                | "TSConstructorType"
                | "TSDeclareFunction"
                | "TSFunctionType"
                | "TSMethodSignature"
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
    let next_type_context = type_context || kind.is_some_and(is_type_only_kind);
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
                if kind == Some("TSConditionalType")
                    && key == "trueType"
                    && fields
                        .get("extendsType")
                        .is_some_and(|extends| contains_infer_binder(extends, name))
                {
                    continue;
                }
                let child_type_context = if (kind == Some("TSParameterProperty")
                    && key == "parameter")
                    || (kind.is_some_and(is_signature_owner) && key == "params")
                    || key == "key"
                {
                    false
                } else {
                    next_type_context
                        || matches!(
                            key.as_str(),
                            "typeAnnotation"
                                | "returnType"
                                | "typeArguments"
                                | "typeParameters"
                                | "superTypeArguments"
                                | "implements"
                        )
                };
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

fn is_type_only_kind(kind: &str) -> bool {
    kind.starts_with("TS") && !is_signature_owner(kind) && kind != "TSParameterProperty"
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
        || (value.get("type").and_then(Value::as_str) == Some("TSInferType")
            && value
                .get("typeParameter")
                .is_some_and(|parameter| type_parameter_name(parameter) == Some(name)))
}

fn contains_infer_binder(value: &Value, name: &str) -> bool {
    if value.get("type").and_then(Value::as_str) == Some("TSInferType")
        && value
            .get("typeParameter")
            .is_some_and(|parameter| type_parameter_name(parameter) == Some(name))
    {
        return true;
    }
    match value {
        Value::Object(fields) => fields
            .values()
            .any(|child| contains_infer_binder(child, name)),
        Value::Array(items) => items.iter().any(|child| contains_infer_binder(child, name)),
        _ => false,
    }
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

fn inferred_return_reference(owner: &Value, type_parameter: &str) -> Option<(u32, u32)> {
    let kind = owner.get("type").and_then(Value::as_str)?;
    if !matches!(
        kind,
        "ArrowFunctionExpression" | "FunctionDeclaration" | "FunctionExpression"
    ) || owner
        .get("returnType")
        .is_some_and(|return_type| !return_type.is_null())
    {
        return None;
    }
    let mut bindings = Vec::new();
    for parameter in owner.get("params")?.as_array()? {
        let mut references = Vec::new();
        collect_references(parameter, type_parameter, false, true, &mut references);
        if !references.is_empty() {
            collect_binding_names(parameter, &mut bindings);
        }
    }
    if bindings.is_empty() {
        return None;
    }
    let body = owner.get("body")?;
    if kind == "ArrowFunctionExpression"
        && body.get("type").and_then(Value::as_str) != Some("BlockStatement")
    {
        return expression_binding_reference(body, &bindings);
    }
    return_binding_reference(body, &bindings, true)
}

fn collect_binding_names(value: &Value, out: &mut Vec<String>) {
    match value.get("type").and_then(Value::as_str) {
        Some("Identifier") => {
            if let Some(name) = value.get("name").and_then(Value::as_str) {
                out.push(name.into());
            }
        }
        Some("AssignmentPattern") => {
            if let Some(left) = value.get("left") {
                collect_binding_names(left, out);
            }
        }
        Some("RestElement") => {
            if let Some(argument) = value.get("argument") {
                collect_binding_names(argument, out);
            }
        }
        Some("ObjectPattern") => {
            if let Some(properties) = value.get("properties").and_then(Value::as_array) {
                for property in properties {
                    let binding = property.get("value").unwrap_or(property);
                    collect_binding_names(binding, out);
                }
            }
        }
        Some("ArrayPattern") => {
            if let Some(elements) = value.get("elements").and_then(Value::as_array) {
                for element in elements {
                    collect_binding_names(element, out);
                }
            }
        }
        Some("TSParameterProperty") => {
            if let Some(parameter) = value.get("parameter") {
                collect_binding_names(parameter, out);
            }
        }
        _ => {}
    }
}

include!("type_parameters/semantic.rs");

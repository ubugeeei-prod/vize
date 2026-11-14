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
                | "TSConstructorType"
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
                if kind == Some("TSConditionalType")
                    && key == "trueType"
                    && fields
                        .get("extendsType")
                        .is_some_and(|extends| contains_infer_binder(extends, name))
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
                out.push(name.to_owned());
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

fn return_binding_reference(value: &Value, bindings: &[String], root: bool) -> Option<(u32, u32)> {
    let kind = value.get("type").and_then(Value::as_str);
    if !root
        && kind.is_some_and(|kind| {
            kind.contains("Function") || matches!(kind, "ClassDeclaration" | "ClassExpression")
        })
    {
        return None;
    }
    if kind == Some("ReturnStatement") {
        return value
            .get("argument")
            .and_then(|argument| expression_binding_reference(argument, bindings));
    }
    match value {
        Value::Object(fields) => fields
            .values()
            .find_map(|child| return_binding_reference(child, bindings, false)),
        Value::Array(items) => items
            .iter()
            .find_map(|child| return_binding_reference(child, bindings, false)),
        _ => None,
    }
}

fn expression_binding_reference(value: &Value, bindings: &[String]) -> Option<(u32, u32)> {
    if value.get("type").and_then(Value::as_str) == Some("Identifier")
        && value
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| bindings.iter().any(|binding| binding == name))
    {
        let start = u32::try_from(value.get("start")?.as_u64()?).ok()?;
        let end = u32::try_from(value.get("end")?.as_u64()?).ok()?;
        return Some((start, end));
    }
    match value {
        Value::Object(fields) => fields
            .values()
            .find_map(|child| expression_binding_reference(child, bindings)),
        Value::Array(items) => items
            .iter()
            .find_map(|child| expression_binding_reference(child, bindings)),
        _ => None,
    }
}

#[cfg(test)]
mod tests;

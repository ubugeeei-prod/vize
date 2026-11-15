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

/// Replaces the conservative syntax fallback with the checker's inferred
/// return/member observations. The base count excludes the old return-flow
/// guess, so `return x.length` does not pretend to return `T`, while a true
/// inferred `T`/`Map<K, V>` return still contributes the same count upstream
/// observes.
pub(super) fn apply(declaration: &mut LintNode, facts: &FxHashMap<(u32, u32), &ExpressionFact>) {
    if declaration.kind != "TSTypeParameterDeclaration" {
        return;
    }
    let Some(queries) = declaration
        .fields
        .get("__typeParameterUsageQueries")
        .and_then(Value::as_array)
    else {
        return;
    };
    let from_class = matches!(
        declaration.field_str("__ownerKind"),
        Some("ClassDeclaration" | "ClassExpression")
    );
    let Some(params) = declaration.child_lists.get_mut("params") else {
        return;
    };
    for param in params {
        if param.field_bool("__repeatedInAst") == Some(true) {
            continue;
        }
        let Some(name) = param.field_str("__typeParameterName").map(str::to_owned) else {
            continue;
        };
        let Some(base) = param
            .fields
            .get("__typeParameterBaseUsageCount")
            .and_then(Value::as_u64)
        else {
            continue;
        };
        let mut inferred = 0u64;
        let mut answered = false;
        for query in queries {
            let (Some(start), Some(end), Some(kind)) = (
                field_u32(query, "start"),
                field_u32(query, "end"),
                query.get("kind").and_then(Value::as_str),
            ) else {
                continue;
            };
            let Some(fact) = facts.get(&(start, end)) else {
                continue;
            };
            if query
                .get("shadows")
                .and_then(Value::as_array)
                .is_some_and(|shadows| {
                    shadows
                        .iter()
                        .any(|shadow| shadow.as_str() == Some(name.as_str()))
                })
            {
                continue;
            }
            let usage = if kind == "return" {
                let Some(returns) = fact.returns.as_ref() else {
                    continue;
                };
                answered = true;
                return_usage(returns, &name, from_class)
            } else {
                answered = true;
                rendered_usage(&fact.type_texts, &name, true, false)
            };
            inferred = (inferred + usage).min(2);
            if inferred >= 2 {
                break;
            }
        }
        if answered {
            param.fields.insert(
                "__typeParameterUsageCount".into(),
                json!((base + inferred).min(3)),
            );
        }
    }
}

fn usage_queries(owner: &Value, owner_kind: &str) -> Vec<Value> {
    if matches!(owner_kind, "ClassDeclaration" | "ClassExpression") {
        return class_usage_queries(owner);
    }
    if needs_semantic_return(owner) {
        return node_range(owner)
            .map(|(start, end)| vec![json!({ "start": start, "end": end, "kind": "return" })])
            .unwrap_or_default();
    }
    Vec::new()
}

fn class_usage_queries(owner: &Value) -> Vec<Value> {
    let Some(members) = owner
        .get("body")
        .and_then(|body| body.get("body"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };
    let mut queries = Vec::new();
    for member in members {
        let function = if is_function_like(member) {
            Some(member)
        } else {
            member.get("value").filter(|value| is_function_like(value))
        };
        if let Some(function) = function {
            if needs_semantic_return(function)
                && let Some((start, end)) = node_range(function)
            {
                queries.push(json!({
                    "start": start,
                    "end": end,
                    "kind": "return",
                    "shadows": declared_type_parameters(function),
                }));
            }
            continue;
        }
        let kind = member.get("type").and_then(Value::as_str);
        if matches!(kind, Some("AccessorProperty" | "PropertyDefinition"))
            && !has_type_annotation(member)
            && let Some((start, end)) = node_range(member)
        {
            queries.push(json!({ "start": start, "end": end, "kind": "type" }));
        }
    }
    queries
}

fn declared_type_parameters(value: &Value) -> Vec<&str> {
    value
        .get("typeParameters")
        .and_then(|parameters| parameters.get("params"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(type_parameter_name)
        .collect()
}

fn needs_semantic_return(owner: &Value) -> bool {
    if !is_function_like(owner) {
        return false;
    }
    !has_type_annotation_key(owner, "returnType")
        || owner
            .get("returnType")
            .is_some_and(|return_type| contains_kind(return_type, "TSTypeQuery"))
}

fn is_function_like(value: &Value) -> bool {
    value
        .get("type")
        .and_then(Value::as_str)
        .is_some_and(|kind| {
            kind.contains("Function")
                || matches!(
                    kind,
                    "TSCallSignatureDeclaration"
                        | "TSConstructSignatureDeclaration"
                        | "TSConstructorType"
                        | "TSDeclareFunction"
                        | "TSMethodSignature"
                )
        })
}

fn has_type_annotation(value: &Value) -> bool {
    has_type_annotation_key(value, "typeAnnotation")
}

fn has_type_annotation_key(value: &Value, key: &str) -> bool {
    value
        .get(key)
        .is_some_and(|annotation| !annotation.is_null())
}

fn node_range(value: &Value) -> Option<(u32, u32)> {
    Some((
        u32::try_from(value.get("start")?.as_u64()?).ok()?,
        u32::try_from(value.get("end")?.as_u64()?).ok()?,
    ))
}

fn field_u32(value: &Value, key: &str) -> Option<u32> {
    u32::try_from(value.get(key)?.as_u64()?).ok()
}

fn contains_kind(value: &Value, wanted: &str) -> bool {
    if value.get("type").and_then(Value::as_str) == Some(wanted) {
        return true;
    }
    match value {
        Value::Object(fields) => fields.values().any(|child| contains_kind(child, wanted)),
        Value::Array(items) => items.iter().any(|child| contains_kind(child, wanted)),
        _ => false,
    }
}

fn reference_implies_multiple(
    value: &Value,
    name: &str,
    owner_kind: &str,
    in_return: bool,
    readonly: bool,
    root: bool,
) -> bool {
    if !root && declares_type_parameter(value, name) {
        return false;
    }
    let kind = value.get("type").and_then(Value::as_str);
    let from_class = matches!(owner_kind, "ClassDeclaration" | "ClassExpression");
    let contains = |child: Option<&Value>| {
        let Some(child) = child else {
            return false;
        };
        let mut references = Vec::new();
        collect_references(child, name, false, true, &mut references);
        !references.is_empty()
    };
    if from_class
        && root
        && (contains(value.get("superTypeArguments")) || contains(value.get("implements")))
    {
        return true;
    }
    match kind {
        Some("TSTypeReference") => {
            let arguments = value
                .get("typeArguments")
                .or_else(|| value.get("typeParameters"));
            if contains(arguments) {
                let generic = type_reference_name(value);
                if !matches!(generic, Some("Array" | "ReadonlyArray")) {
                    return true;
                }
                if from_class || (in_return && generic == Some("Array")) {
                    return true;
                }
            }
        }
        Some("TSArrayType" | "TSTupleType") if contains(Some(value)) => {
            if from_class || (in_return && !readonly) {
                return true;
            }
        }
        Some("TSParameterProperty") if contains(Some(value)) => return true,
        Some("TSInstantiationExpression") if contains(value.get("typeArguments")) => return true,
        Some("TSTypeQuery") if contains(value.get("typeArguments")) => return true,
        _ => {}
    }
    let next_readonly = readonly
        || (kind == Some("TSTypeOperator")
            && value.get("operator").and_then(Value::as_str) == Some("readonly"));
    match value {
        Value::Object(fields) => fields.iter().any(|(key, child)| {
            if should_skip_child(
                kind,
                fields
                    .get("computed")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                key,
            ) {
                return false;
            }
            let child_in_return = in_return || (is_function_like(value) && key == "returnType");
            reference_implies_multiple(
                child,
                name,
                owner_kind,
                child_in_return,
                next_readonly,
                false,
            )
        }),
        Value::Array(items) => items.iter().any(|child| {
            reference_implies_multiple(child, name, owner_kind, in_return, next_readonly, false)
        }),
        _ => false,
    }
}

fn type_reference_name(value: &Value) -> Option<&str> {
    let name = value.get("typeName")?;
    name.get("name").and_then(Value::as_str)
}

fn return_usage(returns: &Value, name: &str, from_class: bool) -> u64 {
    let Some(signatures) = returns.get("signatures").and_then(Value::as_array) else {
        return 0;
    };
    signatures
        .iter()
        .map(|signature| {
            let texts = signature
                .get("texts")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str);
            rendered_str_usage(texts, name, from_class, true)
        })
        .max()
        .unwrap_or(0)
}

fn rendered_usage(texts: &[String], name: &str, from_class: bool, is_return: bool) -> u64 {
    rendered_str_usage(
        texts.iter().map(|text| text.as_str()),
        name,
        from_class,
        is_return,
    )
}

fn rendered_str_usage<'a>(
    texts: impl Iterator<Item = &'a str>,
    name: &str,
    from_class: bool,
    is_return: bool,
) -> u64 {
    texts
        .map(|text| text_usage(text, name, from_class, is_return))
        .max()
        .unwrap_or(0)
}

fn text_usage(text: &str, name: &str, from_class: bool, is_return: bool) -> u64 {
    if locally_binds(text, name) {
        return 0;
    }
    let positions = identifier_positions(text, name);
    if positions.is_empty() {
        return 0;
    }
    if positions.len() >= 2 {
        return 2;
    }
    let Some(&position) = positions.first() else {
        return 0;
    };
    if let Some(wrapper) = generic_wrapper(text, position) {
        if wrapper == "ReadonlyArray" && !from_class {
            return 1;
        }
        if wrapper == "Array" && !from_class && !is_return {
            return 1;
        }
        return 2;
    }
    let before = text.get(..position).unwrap_or_default().trim_end();
    let after = text
        .get(position.saturating_add(name.len())..)
        .unwrap_or_default()
        .trim_start();
    let readonly_collection = before.ends_with("readonly") || before.ends_with("readonly [");
    let array_or_tuple = after.starts_with("[]")
        || before.ends_with('[')
        || (before.ends_with("readonly [") && after.starts_with(']'));
    if array_or_tuple && (from_class || (is_return && !readonly_collection)) {
        2
    } else {
        1
    }
}

fn locally_binds(text: &str, name: &str) -> bool {
    let Some(parameters) = text
        .trim_start()
        .strip_prefix('<')
        .and_then(|tail| tail.split_once('>').map(|(parameters, _)| parameters))
    else {
        return false;
    };
    !identifier_positions(parameters, name).is_empty()
}

fn identifier_positions(text: &str, name: &str) -> Vec<usize> {
    text.match_indices(name)
        .filter_map(|(index, _)| {
            let before = index
                .checked_sub(1)
                .and_then(|before| text.as_bytes().get(before))
                .copied();
            let after = text.as_bytes().get(index + name.len()).copied();
            let identifier =
                |byte: u8| byte == b'_' || byte == b'$' || byte.is_ascii_alphanumeric();
            (!before.is_some_and(identifier) && !after.is_some_and(identifier)).then_some(index)
        })
        .collect()
}

fn generic_wrapper(text: &str, position: usize) -> Option<&str> {
    let prefix = text.get(..position)?;
    let open = prefix.rfind('<')?;
    let close_before = prefix.rfind('>');
    if close_before.is_some_and(|close| close > open) || !text.get(position..)?.contains('>') {
        return None;
    }
    let before = prefix.get(..open)?.trim_end();
    let start = before
        .char_indices()
        .rev()
        .find(|(_, character)| !character.is_ascii_alphanumeric() && *character != '_')
        .map_or(0, |(index, character)| index + character.len_utf8());
    before.get(start..)
}

#[cfg(test)]
fn enrich(source: &str, nodes: &mut [LintNode], program: &Value) {
    prepare(source, nodes, program);
}

#[cfg(test)]
mod tests;

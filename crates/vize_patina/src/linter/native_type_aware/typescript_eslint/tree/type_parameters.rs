//! Syntax facts for no-unnecessary-type-parameters (#12947).
//!
//! typescript-eslint first counts a type parameter's written references in
//! its owner's signature (`isTypeParameterRepeatedInAST`) and asks the
//! checker only when that is not enough. The declaration node gets its owner
//! and, per parameter, the name, the written references (with what replacing
//! them needs), the constraint text and the range the suggestion removes.
use corsa::lint::LintNode;
use serde_json::{Value, json};

/// The declarations typescript-eslint checks, by TS-ESTree kind.
const OWNERS: &[&str] = &[
    "ArrowFunctionExpression",
    "FunctionDeclaration",
    "FunctionExpression",
    "TSCallSignatureDeclaration",
    "TSConstructorType",
    "TSDeclareFunction",
    "TSEmptyBodyFunctionExpression",
    "TSFunctionType",
    "TSMethodSignature",
    "ClassDeclaration",
    "ClassExpression",
];

/// Constraints that need parentheses inside an array, union, and the like.
const WEAK_CONSTRAINTS: &[&str] = &[
    "TSUnionType",
    "TSIntersectionType",
    "TSConditionalType",
    "TSTypeOperator",
    "TSFunctionType",
    "TSConstructorType",
];

/// Reference parents where such a constraint needs them.
const WEAK_PARENTS: &[&str] = &[
    "TSArrayType",
    "TSIndexedAccessType",
    "TSIntersectionType",
    "TSUnionType",
    "TSConditionalType",
    "TSTypeOperator",
    "TSOptionalType",
];

/// Stamps `node` (a `TSTypeParameterDeclaration` under `owner`) with the
/// facts the rule and the checker query read.
pub(super) fn stamp(node: &mut LintNode, owner: &Value, owner_kind: &str, source: &str) {
    if !OWNERS.contains(&owner_kind) {
        return;
    }
    let (Some(start), Some(end)) = (offset(owner, "start"), offset(owner, "end")) else {
        return;
    };
    node.fields.insert("__ownerKind".into(), json!(owner_kind));
    node.fields
        .insert("__ownerRange".into(), json!({ "start": start, "end": end }));
    if owner_kind == "TSCallSignatureDeclaration" {
        node.fields.insert("__ownerSignature".into(), json!(true));
    }
    if owner_kind.starts_with("Class") {
        let members = owner
            .get("body")
            .and_then(|body| body.get("body"))
            .and_then(Value::as_array)
            .map(|members| {
                members
                    .iter()
                    .filter_map(|member| {
                        let constructor =
                            member.get("kind").and_then(Value::as_str) == Some("constructor");
                        Some(json!({
                            "start": offset(member, "start")?,
                            "end": offset(member, "end")?,
                            "constructor": constructor,
                        }))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        node.fields
            .insert("__ownerMembers".into(), Value::Array(members));
    }
    // Upstream counts written references up to the body (a class body, a
    // function body) or, without one, the end of the return type.
    let body_start = owner
        .get("body")
        .and_then(|body| offset(body, "start"))
        .or_else(|| owner.get("returnType").and_then(|ty| offset(ty, "end")));
    let declared = owner
        .get("typeParameters")
        .and_then(|list| list.get("params"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let list_range = (node.range.start, node.range.end);
    let count = declared.len();
    let Some(params) = node.child_lists.get_mut("params") else {
        return;
    };
    for (index, (param, value)) in params.iter_mut().zip(&declared).enumerate() {
        let Some(name) = value
            .get("name")
            .and_then(|name| name.get("name"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        let mut references = Vec::new();
        collect_references(owner, name, &mut Vec::new(), &mut references);
        let own = (param.range.start, param.range.end);
        let repeated = repeated_in_signature(&references, own, body_start);
        let constraint = value.get("constraint").filter(|value| !value.is_null());
        let constraint_kind = constraint
            .and_then(|constraint| constraint.get("type"))
            .and_then(Value::as_str);
        let constraint_text = match (constraint, constraint_kind) {
            (Some(constraint), Some(kind)) if kind != "TSAnyKeyword" => {
                text(source, constraint).unwrap_or("unknown")
            }
            _ => "unknown",
        };
        let fields = &mut param.fields;
        fields.insert("__typeParameterName".into(), json!(name));
        fields.insert("__repeatedInAst".into(), json!(repeated));
        fields.insert("__constraintText".into(), json!(constraint_text));
        fields.insert(
            "__constraintIsComplex".into(),
            json!(constraint_kind.is_some_and(|kind| WEAK_CONSTRAINTS.contains(&kind))),
        );
        let written = references
            .iter()
            .map(|reference| {
                json!({
                    "start": reference.start,
                    "end": reference.end,
                    "hasMatchingAncestorType": reference.weak_parent,
                })
            })
            .collect::<Vec<_>>();
        fields.insert("__references".into(), Value::Array(written));
        if let Some((start, end)) = removal_range(source, list_range, own, index, count) {
            fields.insert(
                "__listRemovalRange".into(),
                json!({ "start": start, "end": end }),
            );
        }
        if let Some(default) = value.get("default").filter(|value| !value.is_null())
            && let (Some(start), Some(end)) = (offset(default, "start"), offset(default, "end"))
        {
            fields.insert(
                "__defaultRange".into(),
                json!({ "start": start, "end": end }),
            );
        }
    }
}

/// One written type reference to the parameter (the `TSTypeReference`).
struct Reference {
    start: u32,
    end: u32,
    /// Its parent is an array, union or similar type (`hasMatchingAncestorType`).
    weak_parent: bool,
    /// It is a direct type argument of something other than `Array<..>` or
    /// `ReadonlyArray<..>`, which upstream takes as a repeated use.
    type_argument: bool,
}

/// Type references named `name` under `value`, outside nested declarations
/// that declare their own `name`. `ancestors` holds the path to `value`.
fn collect_references<'v>(
    value: &'v Value,
    name: &str,
    ancestors: &mut Vec<&'v Value>,
    out: &mut Vec<Reference>,
) {
    let kind = value.get("type").and_then(Value::as_str);
    if !ancestors.is_empty() && declares(value, name) {
        return;
    }
    if kind == Some("TSTypeReference")
        && value
            .get("typeName")
            .filter(|type_name| type_name.get("type").and_then(Value::as_str) == Some("Identifier"))
            .and_then(|type_name| type_name.get("name"))
            .and_then(Value::as_str)
            == Some(name)
        && let (Some(start), Some(end)) = (offset(value, "start"), offset(value, "end"))
    {
        let parent = ancestors.last().copied();
        let parent_kind = parent
            .and_then(|parent| parent.get("type"))
            .and_then(Value::as_str);
        out.push(Reference {
            start,
            end,
            weak_parent: parent_kind.is_some_and(|kind| WEAK_PARENTS.contains(&kind)),
            type_argument: is_type_argument(value, ancestors),
        });
    }
    let children = match value {
        Value::Object(map) => map
            .iter()
            .filter(|(key, _)| !matches!(key.as_str(), "type" | "start" | "end" | "range" | "loc"))
            .map(|(_, child)| child)
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    if kind.is_some() {
        ancestors.push(value);
    }
    for child in children {
        match child {
            Value::Object(_) => collect_references(child, name, ancestors, out),
            Value::Array(items) => {
                for item in items {
                    collect_references(item, name, ancestors, out);
                }
            }
            _ => {}
        }
    }
    if kind.is_some() {
        ancestors.pop();
    }
}

/// Whether `value` declares its own type parameter `name`, shadowing ours
/// (a nested generic, a mapped type's key, an `infer` type).
fn declares(value: &Value, name: &str) -> bool {
    let named = |param: &Value| {
        param
            .get("name")
            .and_then(|name| name.get("name").or(Some(name)))
            .and_then(Value::as_str)
            == Some(name)
    };
    let own_list = value
        .get("typeParameters")
        .and_then(|list| list.get("params"))
        .and_then(Value::as_array)
        .is_some_and(|params| params.iter().any(named));
    let kind = value.get("type").and_then(Value::as_str);
    let mapped = kind == Some("TSMappedType")
        && (value
            .get("key")
            .is_some_and(|key| key.get("name").and_then(Value::as_str) == Some(name))
            || value.get("typeParameter").is_some_and(named));
    let inferred = kind == Some("TSInferType") && value.get("typeParameter").is_some_and(named);
    own_list || mapped || inferred
}

/// Upstream's type-argument shortcut: the reference, through any unions and
/// intersections, is an argument of a type parameter instantiation that does
/// not belong to `Array` or `ReadonlyArray`.
fn is_type_argument(reference: &Value, ancestors: &[&Value]) -> bool {
    let mut index = ancestors.len();
    let mut child = reference;
    while let Some(parent) = index.checked_sub(1).and_then(|at| ancestors.get(at)) {
        match kind_of(parent) {
            Some("TSUnionType" | "TSIntersectionType") => {
                child = parent;
                index -= 1;
            }
            Some("TSTypeParameterInstantiation") => {
                let direct = std::ptr::eq(child, reference)
                    && parent
                        .get("params")
                        .and_then(Value::as_array)
                        .is_some_and(|params| {
                            params.iter().any(|param| std::ptr::eq(param, reference))
                        });
                let owner = index.checked_sub(2).and_then(|at| ancestors.get(at));
                let array = owner.is_some_and(|owner| {
                    kind_of(owner) == Some("TSTypeReference")
                        && owner
                            .get("typeName")
                            .and_then(|name| name.get("name"))
                            .and_then(Value::as_str)
                            .is_some_and(|name| matches!(name, "Array" | "ReadonlyArray"))
                });
                return direct && !array;
            }
            _ => return false,
        }
    }
    false
}

/// `isTypeParameterRepeatedInAST`: two written uses in the signature, or one
/// as a type argument.
fn repeated_in_signature(references: &[Reference], own: (u32, u32), body: Option<u32>) -> bool {
    let mut total = 0;
    for reference in references {
        // Uses inside the parameter's own declaration, and in the body,
        // do not count.
        if reference.start < own.1 && reference.end > own.0 {
            continue;
        }
        if body.is_some_and(|body| reference.start > body) {
            continue;
        }
        if reference.type_argument {
            return true;
        }
        total += 1;
        if total >= 2 {
            return true;
        }
    }
    false
}

/// The range the suggestion removes: the whole `<T>` for a single parameter,
/// the parameter and the following comma for the first, otherwise the
/// preceding comma and the parameter.
fn removal_range(
    source: &str,
    list: (u32, u32),
    own: (u32, u32),
    index: usize,
    count: usize,
) -> Option<(u32, u32)> {
    if count == 1 {
        return Some(list);
    }
    let bytes = source.as_bytes();
    if index == 0 {
        let after = source.get(own.1 as usize..list.1 as usize)?;
        let comma = own.1 as usize + after.find(',')? + 1;
        let next = bytes
            .get(comma..)?
            .iter()
            .position(|byte| !byte.is_ascii_whitespace())?;
        return Some((own.0, u32::try_from(comma + next).ok()?));
    }
    let before = source.get(list.0 as usize..own.0 as usize)?;
    let comma = list.0 as usize + before.rfind(',')?;
    Some((u32::try_from(comma).ok()?, own.1))
}

fn kind_of(value: &Value) -> Option<&str> {
    value.get("type").and_then(Value::as_str)
}

fn offset(value: &Value, key: &str) -> Option<u32> {
    u32::try_from(value.get(key)?.as_u64()?).ok()
}

fn text<'s>(source: &'s str, value: &Value) -> Option<&'s str> {
    source.get(offset(value, "start")? as usize..offset(value, "end")? as usize)
}

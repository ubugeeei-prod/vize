//! Whether `String(x)` / `Number(x)` / `Boolean(x)` / `BigInt(x)` call the
//! global conversion functions or a binding the script declares.
use corsa::lint::LintNode;
use serde_json::{Value, json};
use vize_l0::FxHashSet;

const CONVERSIONS: &[&str] = &["String", "Number", "Boolean", "BigInt"];

/// Stamps `__calleeIsGlobalBuiltin` on conversion calls. typescript-eslint
/// skips a callee with any definition in scope; a same-named declaration
/// anywhere in the script is treated the same way (it can only hide reports).
pub(super) fn stamp(nodes: &mut [LintNode], program: &Value) {
    let mut declared = FxHashSet::default();
    collect_declared(program, &mut declared);
    for node in nodes {
        walk(node, &declared);
    }
}

fn walk(node: &mut LintNode, declared: &FxHashSet<&'static str>) {
    if node.kind == "CallExpression" {
        let name = node
            .child("callee")
            .filter(|callee| callee.kind == "Identifier")
            .and_then(|callee| callee.field_str("name"))
            .and_then(|name| CONVERSIONS.iter().find(|known| **known == name));
        if let Some(name) = name {
            let global = !declared.contains(name);
            node.fields
                .insert("__calleeIsGlobalBuiltin".into(), json!(global));
        }
    }
    for child in node.children.values_mut() {
        walk(child, declared);
    }
    for children in node.child_lists.values_mut() {
        for child in children {
            walk(child, declared);
        }
    }
}

fn collect_declared(value: &Value, declared: &mut FxHashSet<&'static str>) {
    match value {
        Value::Object(map) => {
            match map.get("type").and_then(Value::as_str) {
                Some("VariableDeclarator") => binding_names(map.get("id"), declared),
                Some(
                    "FunctionDeclaration"
                    | "FunctionExpression"
                    | "ArrowFunctionExpression"
                    | "TSDeclareFunction",
                ) => {
                    binding_names(map.get("id"), declared);
                    if let Some(params) = map.get("params").and_then(Value::as_array) {
                        for param in params {
                            binding_names(Some(param), declared);
                        }
                    }
                }
                Some(
                    "ClassDeclaration"
                    | "ClassExpression"
                    | "TSEnumDeclaration"
                    | "TSModuleDeclaration"
                    | "TSImportEqualsDeclaration",
                ) => binding_names(map.get("id"), declared),
                Some("ImportSpecifier" | "ImportDefaultSpecifier" | "ImportNamespaceSpecifier") => {
                    binding_names(map.get("local"), declared);
                }
                Some("CatchClause") => binding_names(map.get("param"), declared),
                _ => {}
            }
            for child in map.values() {
                collect_declared(child, declared);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_declared(item, declared);
            }
        }
        _ => {}
    }
}

fn binding_names(pattern: Option<&Value>, declared: &mut FxHashSet<&'static str>) {
    let Some(pattern) = pattern else {
        return;
    };
    match pattern.get("type").and_then(Value::as_str) {
        Some("Identifier") => {
            let name = pattern.get("name").and_then(Value::as_str);
            if let Some(known) = CONVERSIONS.iter().find(|known| Some(**known) == name) {
                declared.insert(known);
            }
        }
        Some("ObjectPattern") => {
            for property in pattern
                .get("properties")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                binding_names(property.get("value").or(Some(property)), declared);
            }
        }
        Some("ArrayPattern") => {
            for element in pattern
                .get("elements")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                binding_names(Some(element), declared);
            }
        }
        Some("RestElement") => binding_names(pattern.get("argument"), declared),
        Some("AssignmentPattern") => binding_names(pattern.get("left"), declared),
        Some("TSParameterProperty") => binding_names(pattern.get("parameter"), declared),
        _ => {}
    }
}

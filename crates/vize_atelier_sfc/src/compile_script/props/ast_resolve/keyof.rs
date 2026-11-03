//! Runtime type of a `keyof` operator.

use oxc_ast::ast::TSType;
use vize_carton::{FxHashMap, String, ToCompactString};

use super::simple_type_name;

/// Keys of a type this file declares are strings. Keys of a generic parameter
/// or an unresolved import may be numbers or symbols, so like Vue the runtime
/// type is unknown.
pub(super) fn runtime_type(
    operand: &TSType<'_>,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
) -> String {
    if let TSType::TSTypeReference(type_ref) = operand
        && let Some(name) = simple_type_name(&type_ref.type_name)
        && !interfaces.is_some_and(|interfaces| interfaces.contains_key(name))
        && !type_aliases.is_some_and(|aliases| aliases.contains_key(name))
        && !is_builtin_operand(name)
    {
        return "null".to_compact_string();
    }
    "String".to_compact_string()
}

/// Built-ins Vue's `inferRuntimeType` resolves under `keyof`.
fn is_builtin_operand(name: &str) -> bool {
    matches!(
        name,
        "String"
            | "Array"
            | "ArrayLike"
            | "ReadonlyArray"
            | "Parameters"
            | "ConstructorParameters"
            | "Record"
            | "Partial"
            | "Required"
            | "Readonly"
            | "Pick"
            | "Extract"
            | "Function"
            | "Object"
            | "Set"
            | "Map"
            | "WeakSet"
            | "WeakMap"
            | "Date"
            | "Promise"
            | "Error"
            | "Uppercase"
            | "Lowercase"
            | "Capitalize"
            | "Uncapitalize"
    )
}

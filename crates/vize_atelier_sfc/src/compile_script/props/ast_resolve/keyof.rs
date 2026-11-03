//! Runtime type of a `keyof` operator.

use oxc_ast::ast::TSType;
use vize_carton::{FxHashMap, String, ToCompactString};

use super::{collect_props_from_ts_type, simple_type_name};

/// Keys of a type this file declares are strings. Keys of a generic parameter
/// or an unresolved import may be numbers or symbols, so like Vue the runtime
/// type is unknown.
pub(super) fn runtime_type(
    operand: &TSType<'_>,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
) -> String {
    match operand {
        TSType::TSParenthesizedType(paren) => {
            runtime_type(&paren.type_annotation, interfaces, type_aliases)
        }
        // A qualified name (`Types.External`) is never a local declaration.
        TSType::TSTypeReference(type_ref) => match simple_type_name(&type_ref.type_name) {
            Some(name)
                if interfaces.is_some_and(|interfaces| interfaces.contains_key(name))
                    || type_aliases.is_some_and(|aliases| aliases.contains_key(name))
                    || is_builtin_operand(name) =>
            {
                "String".to_compact_string()
            }
            _ => "null".to_compact_string(),
        },
        _ => "String".to_compact_string(),
    }
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

/// Literal keys of a `keyof` operand, for mapped and indexed types.
pub(super) fn literal_values(
    ts_type: &TSType<'_>,
    source: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    seen: &mut Vec<String>,
) -> Option<Vec<String>> {
    let mut props = Vec::new();
    if collect_props_from_ts_type(ts_type, source, interfaces, type_aliases, seen, &mut props) {
        Some(props.into_iter().map(|(name, _)| name).collect())
    } else {
        None
    }
}

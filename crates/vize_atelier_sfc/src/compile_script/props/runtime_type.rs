//! Runtime prop-type mapping: converting TypeScript type text into JavaScript
//! runtime constructors and related string-level helpers.

use vize_carton::FxHashMap;
use vize_carton::{String, ToCompactString};

pub(crate) fn runtime_prop_key(name: &str) -> String {
    if is_valid_identifier(name) {
        return name.to_compact_string();
    }

    serde_json::to_string(name)
        .map(|escaped| escaped.as_str().to_compact_string())
        .unwrap_or_else(|_| {
            let mut escaped = String::with_capacity(name.len() + 2);
            escaped.push('"');
            escaped.push_str(name);
            escaped.push('"');
            escaped
        })
}

#[allow(dead_code)]
pub(super) fn type_includes_top_level_undefined(ts_type: &str) -> bool {
    split_type_at_top_level(ts_type.trim(), '|')
        .into_iter()
        .any(|part| part.trim() == "undefined")
}

pub(super) fn type_includes_top_level_null(ts_type: &str) -> bool {
    split_type_at_top_level(ts_type.trim(), '|')
        .into_iter()
        .any(|part| part.trim() == "null")
}

pub fn add_null_to_runtime_type(js_type: &str, nullable: bool) -> String {
    if !nullable || js_type == "null" {
        return js_type.to_compact_string();
    }

    if js_type.starts_with('[') && js_type.ends_with(']') {
        let inner = &js_type[1..js_type.len() - 1];
        if inner
            .split(',')
            .map(|part| part.trim())
            .any(|part| part == "null")
        {
            return js_type.to_compact_string();
        }

        let mut result = String::with_capacity(js_type.len() + 6);
        result.push('[');
        result.push_str(inner);
        if !inner.trim().is_empty() {
            result.push_str(", ");
        }
        result.push_str("null");
        result.push(']');
        return result;
    }

    let mut result = String::with_capacity(js_type.len() + 8);
    result.push('[');
    result.push_str(js_type);
    result.push_str(", null]");
    result
}

/// Split a type string at a delimiter only at the top level (depth 0),
/// respecting nested `<>`, `()`, `[]`, `{}` and `=>` arrows.
///
/// This intentionally streams over `chars()` with a one-character lookbehind.
/// Prop codegen calls it recursively for union members, so avoiding an
/// intermediate `Vec<char>` prevents repeated heap churn in large prop types.
pub(super) fn split_type_at_top_level(s: &str, delimiter: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::default();
    let mut depth: i32 = 0;
    let mut prev = '\0';

    for c in s.chars() {
        match c {
            '(' | '[' | '{' | '<' => {
                depth += 1;
                current.push(c);
            }
            ')' | ']' | '}' => {
                if depth > 0 {
                    depth -= 1;
                }
                current.push(c);
            }
            '>' => {
                // Don't count > as closing angle bracket when preceded by = (arrow =>)
                if prev == '=' {
                    current.push(c);
                } else {
                    if depth > 0 {
                        depth -= 1;
                    }
                    current.push(c);
                }
            }
            c2 if c2 == delimiter && depth == 0 => {
                parts.push(std::mem::take(&mut current));
            }
            _ => current.push(c),
        }
        prev = c;
    }
    if !current.is_empty() || !parts.is_empty() {
        parts.push(current);
    }
    parts
}

/// Check if a type string contains a top-level `=>` (arrow function signature).
///
/// Like `split_type_at_top_level`, this is a zero-intermediate scanner because
/// it sits on the recursive type-to-runtime-constructor path.
pub(super) fn contains_top_level_arrow(s: &str) -> bool {
    let mut depth: i32 = 0;
    let mut prev = '\0';
    for c in s.chars() {
        match c {
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            '>' => {
                if prev == '=' {
                    // This is `=>`
                    if depth == 0 {
                        return true;
                    }
                    // Inside nested structure — don't change depth
                } else if depth > 0 {
                    depth -= 1;
                }
            }
            _ => {}
        }
        prev = c;
    }
    false
}

/// Convert TypeScript type to JavaScript type constructor
pub(crate) fn ts_type_to_js_type(ts_type: &str) -> String {
    let ts_type = ts_type.trim();

    // Strip `readonly` prefix: `readonly T[]` → `T[]`
    let ts_type = if let Some(rest) = ts_type.strip_prefix("readonly ") {
        rest.trim()
    } else {
        ts_type
    };

    // Handle string literal types: "foo" or 'bar' -> String
    if (ts_type.starts_with('"') && ts_type.ends_with('"'))
        || (ts_type.starts_with('\'') && ts_type.ends_with('\''))
    {
        return "String".to_compact_string();
    }

    // Handle numeric literal types: 123, 1.5 -> Number
    if ts_type.parse::<f64>().is_ok() {
        return "Number".to_compact_string();
    }

    // Handle boolean literal types: true, false -> Boolean
    if ts_type == "true" || ts_type == "false" {
        return "Boolean".to_compact_string();
    }

    // Arrow function types must be detected BEFORE union splitting,
    // because `(x: T) => A | B` is a single function type (return type is `A | B`),
    // not a union of `(x: T) => A` and `B`.
    // Also must come before array/object checks because `(items: T[]) => T[]`
    // ends with `[]` and contains `:`.
    if contains_top_level_arrow(ts_type) {
        return "Function".to_compact_string();
    }

    // Handle union types — split at top level only (respecting nesting).
    // For mixed types like `string | number`, produce `[String, Number]`.
    {
        let parts = split_type_at_top_level(ts_type, '|');
        if parts.len() > 1 {
            let meaningful: Vec<&str> = parts
                .iter()
                .map(|p| p.trim())
                .filter(|p| !p.is_empty() && *p != "undefined" && *p != "null")
                .collect();

            if meaningful.is_empty() {
                return "null".to_compact_string();
            }

            // Collect unique JS types for each union member
            let mut js_types: Vec<String> = Vec::new();
            for part in &meaningful {
                let jt = ts_type_to_js_type(part);
                if !js_types.contains(&jt) {
                    if jt == "null" {
                        return jt;
                    }
                    js_types.push(jt);
                }
            }

            if js_types.len() == 1
                && let Some(only) = js_types.pop()
            {
                return only;
            }

            // Multiple distinct types → array form: [String, Number]
            let joined = js_types.join(", ");
            let mut result = String::with_capacity(joined.len() + 2);
            result.push('[');
            result.push_str(&joined);
            result.push(']');
            return result;
        }
    }

    // Map TypeScript types to JavaScript constructors
    match ts_type.to_lowercase().as_str() {
        "string" => "String".to_compact_string(),
        "number" => "Number".to_compact_string(),
        "boolean" => "Boolean".to_compact_string(),
        "object" => "Object".to_compact_string(),
        "function" => "Function".to_compact_string(),
        "symbol" => "Symbol".to_compact_string(),
        _ => {
            // Handle array types
            if ts_type.ends_with("[]") || ts_type.starts_with("Array<") {
                "Array".to_compact_string()
            } else if ts_type.starts_with('{') || contains_top_level_colon(ts_type) {
                // Object literal type
                "Object".to_compact_string()
            } else if ts_type.starts_with('(') && ts_type.contains("=>") {
                // Function type (fallback, already handled above)
                "Function".to_compact_string()
            } else {
                // Check if this is a built-in JavaScript constructor type
                let type_name = ts_type.split('<').next().unwrap_or(ts_type).trim();
                match type_name {
                    // Built-in JavaScript types that exist at runtime
                    "Date" | "RegExp" | "Error" | "Map" | "Set" | "WeakMap" | "WeakSet"
                    | "Promise" | "ArrayBuffer" | "DataView" | "Int8Array" | "Uint8Array"
                    | "Int16Array" | "Uint16Array" | "Int32Array" | "Uint32Array"
                    | "Float32Array" | "Float64Array" | "BigInt64Array" | "BigUint64Array"
                    | "URL" | "URLSearchParams" | "FormData" | "Blob" | "File" => {
                        type_name.to_compact_string()
                    }
                    // Vue reactive types that are objects at runtime
                    "Ref"
                    | "ShallowRef"
                    | "ComputedRef"
                    | "WritableComputedRef"
                    | "MaybeRef"
                    | "MaybeRefOrGetter"
                    | "Readonly"
                    | "UnwrapRef"
                    | "Reactive"
                    | "ShallowReactive"
                    | "ToRef"
                    | "ToRefs" => "Object".to_compact_string(),
                    // User-defined interface/type or generic type parameter
                    // - Single uppercase letter (T, U, K, V) = generic param → null
                    // - Otherwise = user-defined type → null (types don't exist at runtime)
                    _ => "null".to_compact_string(),
                }
            }
        }
    }
}

/// Check if a type string contains a `:` at the top level (not inside generics/parens).
/// Used to detect object literal types like `{ key: string }` vs types like `Record<K, V>`.
pub(super) fn contains_top_level_colon(s: &str) -> bool {
    let mut depth: i32 = 0;
    let mut prev = '\0';
    for c in s.chars() {
        match c {
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            '>' => {
                if prev == '=' {
                    // Arrow =>, don't change depth
                } else if depth > 0 {
                    depth -= 1;
                }
            }
            ':' if depth == 0 => return true,
            _ => {}
        }
        prev = c;
    }
    false
}

/// Resolve prop type references using type alias/interface maps.
/// For a prop type like `ButtonVariant`, resolves it using the type_aliases and interfaces
/// to determine the correct JS type constructor.
pub fn resolve_prop_js_type(
    ts_type: &str,
    interfaces: &FxHashMap<String, String>,
    type_aliases: &FxHashMap<String, String>,
) -> Option<String> {
    let trimmed = ts_type.trim();
    // Check if it's a simple type reference (identifier, no generics/brackets/arrows/pipes)
    // that would resolve to `null` by default
    if trimmed.is_empty() {
        return None;
    }

    // First try the normal resolution
    let js_type = ts_type_to_js_type(trimmed);
    if js_type != "null" {
        return None; // Normal resolution works fine
    }

    // It resolved to null - try to look up the type name and resolve based on the actual definition
    let base_name = if let Some(idx) = trimmed.find('<') {
        trimmed[..idx].trim()
    } else {
        trimmed
    };

    // Look up in type aliases first
    if let Some(body) = type_aliases.get(base_name) {
        let resolved_type = ts_type_to_js_type(body.trim());
        if resolved_type != "null" {
            return Some(resolved_type);
        }
        // If the alias body contains braces, it's an object type
        if body.contains('{') {
            return Some("Object".to_compact_string());
        }
    }

    // Look up in interfaces
    if let Some(body) = interfaces.get(base_name) {
        // Interfaces always resolve to Object
        let _ = body;
        return Some("Object".to_compact_string());
    }

    None
}

/// Strip the `readonly` keyword from a TypeScript type.
/// Handles patterns like `readonly { value: string }[]` → `{ value: string }[]`
pub fn strip_readonly_prefix(ts_type: &str) -> &str {
    let trimmed = ts_type.trim();
    if let Some(rest) = trimmed.strip_prefix("readonly ") {
        rest.trim()
    } else {
        trimmed
    }
}

/// Check if a string is a valid JS identifier
pub fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }

    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

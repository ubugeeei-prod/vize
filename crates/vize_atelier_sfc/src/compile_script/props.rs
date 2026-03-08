//! Props and emit type extraction utilities.
//!
//! This module handles extracting prop types from TypeScript type definitions
//! and processing withDefaults defaults.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, Expression, ObjectPropertyKind, PropertyKey, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::FxHashMap;
use vize_carton::{String, ToCompactString};

/// Prop type information
#[derive(Debug, Clone)]
pub struct PropTypeInfo {
    /// JavaScript type constructor name (String, Number, Boolean, Array, Object, Function)
    pub js_type: String,
    /// Original TypeScript type (for PropType<T> usage)
    pub ts_type: Option<String>,
    /// Whether the prop is optional
    pub optional: bool,
}

/// Strip TypeScript comments from source while preserving string literals.
fn strip_ts_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    let mut in_string = false;
    let mut string_char = b'"';

    while i < bytes.len() {
        if in_string {
            if bytes[i] == string_char && (i == 0 || bytes[i - 1] != b'\\') {
                in_string = false;
            }
            result.push(bytes[i] as char);
            i += 1;
            continue;
        }

        match bytes[i] {
            b'\'' | b'"' | b'`' => {
                in_string = true;
                string_char = bytes[i];
                result.push(bytes[i] as char);
                i += 1;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                // Line comment: skip until newline
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                // Block comment: skip until */
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 < bytes.len() {
                    i += 2; // skip */
                }
            }
            _ => {
                result.push(bytes[i] as char);
                i += 1;
            }
        }
    }
    result
}

/// Join multi-line type definitions where continuation lines start with `|` or `&`.
/// For example:
/// ```text
/// type?:
///     | 'input'
///     | 'text';
/// ```
/// becomes: `type?: | 'input' | 'text';`
fn join_union_continuation_lines(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut result = String::with_capacity(input.len());
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('|') || trimmed.starts_with('&') {
            // Join to previous line with a space
            result.push(' ');
            result.push_str(trimmed);
        } else {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(line);
        }
    }
    result
}

/// Extract prop types from TypeScript type definition.
/// Returns a Vec to preserve definition order (important for matching Vue's output).
pub fn extract_prop_types_from_type(type_args: &str) -> Vec<(String, PropTypeInfo)> {
    let mut props = Vec::new();

    // Strip comments before parsing
    let stripped = strip_ts_comments(type_args);
    // Join multi-line union/intersection types (lines starting with | or &)
    let joined = join_union_continuation_lines(&stripped);
    let content = joined.trim();
    let content = if content.starts_with('{') && content.ends_with('}') {
        &content[1..content.len() - 1]
    } else {
        content
    };

    // Split by commas/semicolons/newlines (but not inside nested braces)
    let mut depth: i32 = 0;
    let mut current = String::default();
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        match c {
            '{' | '<' | '(' | '[' => {
                depth += 1;
                current.push(c);
            }
            '}' | ')' | ']' => {
                if depth > 0 {
                    depth -= 1;
                }
                current.push(c);
            }
            '>' => {
                // Don't count `>` as closing angle bracket when preceded by `=` (arrow function `=>`)
                if i > 0 && chars[i - 1] == '=' {
                    current.push(c);
                } else {
                    if depth > 0 {
                        depth -= 1;
                    }
                    current.push(c);
                }
            }
            ',' | ';' if depth <= 0 => {
                extract_prop_type_info(&current, &mut props);
                current.clear();
                depth = 0;
            }
            '\n' if depth <= 0 => {
                // Don't split on newline if the current segment ends with ':' (type on next line)
                let trimmed_current = current.trim();
                if !trimmed_current.is_empty() && !trimmed_current.ends_with(':') {
                    extract_prop_type_info(&current, &mut props);
                    current.clear();
                    depth = 0;
                }
                // If ends with ':', keep accumulating (type continues on next line)
            }
            _ => current.push(c),
        }
        i += 1;
    }
    extract_prop_type_info(&current, &mut props);

    props
}

fn extract_prop_type_info(segment: &str, props: &mut Vec<(String, PropTypeInfo)>) {
    let trimmed = segment.trim();
    if trimmed.is_empty() {
        return;
    }

    // Parse "name?: Type" or "name: Type"
    if let Some(colon_pos) = trimmed.find(':') {
        let name_part = &trimmed[..colon_pos];
        let type_part = &trimmed[colon_pos + 1..];

        let optional = name_part.ends_with('?');
        let name = name_part.trim().trim_end_matches('?').trim();

        if !name.is_empty() && is_valid_identifier(name) {
            let ts_type_str = type_part.trim().to_compact_string();
            let js_type = ts_type_to_js_type(&ts_type_str);
            // Avoid duplicates (intersection types may have overlapping props)
            if !props.iter().any(|(n, _)| n == name) {
                props.push((
                    name.to_compact_string(),
                    PropTypeInfo {
                        js_type,
                        ts_type: Some(ts_type_str),
                        optional,
                    },
                ));
            }
        }
    }
}

/// Split a type string at a delimiter only at the top level (depth 0),
/// respecting nested `<>`, `()`, `[]`, `{}` and `=>` arrows.
fn split_type_at_top_level(s: &str, delimiter: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::default();
    let mut depth: i32 = 0;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
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
                if i > 0 && chars[i - 1] == '=' {
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
        i += 1;
    }
    if !current.is_empty() || !parts.is_empty() {
        parts.push(current);
    }
    parts
}

/// Check if a type string contains a top-level `=>` (arrow function signature).
fn contains_top_level_arrow(s: &str) -> bool {
    let mut depth: i32 = 0;
    let chars: Vec<char> = s.chars().collect();
    for i in 0..chars.len() {
        match chars[i] {
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            '>' => {
                if i > 0 && chars[i - 1] == '=' {
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
    }
    false
}

/// Convert TypeScript type to JavaScript type constructor
fn ts_type_to_js_type(ts_type: &str) -> String {
    let ts_type = ts_type.trim();

    // Strip `readonly` prefix: `readonly T[]` → `T[]`
    let ts_type = if ts_type.starts_with("readonly ") {
        ts_type.strip_prefix("readonly ").unwrap().trim()
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
                .filter(|p| *p != "undefined" && *p != "null")
                .collect();

            if meaningful.is_empty() {
                return "null".to_compact_string();
            }

            // Collect unique JS types for each union member
            let mut js_types: Vec<String> = Vec::new();
            for part in &meaningful {
                let jt = ts_type_to_js_type(part);
                if !js_types.contains(&jt) {
                    js_types.push(jt);
                }
            }

            if js_types.len() == 1 {
                return js_types.into_iter().next().unwrap();
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
fn contains_top_level_colon(s: &str) -> bool {
    let mut depth: i32 = 0;
    let chars: Vec<char> = s.chars().collect();
    for i in 0..chars.len() {
        match chars[i] {
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            '>' => {
                if i > 0 && chars[i - 1] == '=' {
                    // Arrow =>, don't change depth
                } else if depth > 0 {
                    depth -= 1;
                }
            }
            ':' if depth == 0 => return true,
            _ => {}
        }
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

/// Extract emit names from TypeScript type definition
pub fn extract_emit_names_from_type(type_args: &str) -> Vec<String> {
    let mut emits = Vec::new();

    // First, try Vue 3.3+ shorthand format:
    //   { change: [value: string]; submit: []; update: [id: number] }
    // Property names before `:` followed by `[` are event names
    let trimmed = type_args.trim();
    let is_shorthand = trimmed.starts_with('{')
        && trimmed.contains('[')
        && !trimmed.contains("(e:")
        && !trimmed.contains("(event:");

    if is_shorthand {
        // Extract property names from { name: [...], name: [...] } format
        let inner = if trimmed.starts_with('{') && trimmed.ends_with('}') {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        };

        // Split by lines or semicolons and extract property names
        for segment in inner.split([';', '\n']) {
            let seg = segment.trim();
            if seg.is_empty() {
                continue;
            }
            // Find the property name before the first ':'
            if let Some(colon_pos) = seg.find(':') {
                let name = seg[..colon_pos].trim();
                // Remove quotes if present
                let name = name.trim_matches(|c| c == '\'' || c == '"');
                if !name.is_empty()
                    && name
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                {
                    emits.push(name.to_compact_string());
                }
            }
        }

        if !emits.is_empty() {
            return emits;
        }
    }

    // Fall back to call signature format:
    //   (e: 'eventName'): void; (e: 'otherEvent', value: string): void
    // Match quoted string literals in (e: 'name') patterns
    let mut in_string = false;
    let mut quote_char = ' ';
    let mut current_string = String::default();

    for c in type_args.chars() {
        if !in_string && (c == '\'' || c == '"') {
            in_string = true;
            quote_char = c;
            current_string.clear();
        } else if in_string && c == quote_char {
            in_string = false;
            if !current_string.is_empty() {
                emits.push(current_string.clone());
            }
        } else if in_string {
            current_string.push(c);
        }
    }

    emits
}

/// Extract default values from withDefaults second argument
/// Input: "withDefaults(defineProps<{...}>(), { prop1: default1, prop2: default2 })"
/// Returns: HashMap of prop name to default value string
pub fn extract_with_defaults_defaults(with_defaults_args: &str) -> FxHashMap<String, String> {
    let mut defaults = FxHashMap::default();
    let trimmed = with_defaults_args.trim();
    if trimmed.is_empty() {
        return defaults;
    }

    const WRAP_PREFIX: &str = "const __vize_defaults__ = ";
    let mut wrapped = String::with_capacity(WRAP_PREFIX.len() + trimmed.len() + 1);
    wrapped.push_str(WRAP_PREFIX);
    wrapped.push_str(trimmed);
    wrapped.push(';');

    let allocator = Allocator::default();
    let parse_result = Parser::new(
        &allocator,
        &wrapped,
        SourceType::default().with_typescript(true),
    )
    .parse();
    if !parse_result.errors.is_empty() {
        return defaults;
    }

    let Some(Statement::VariableDeclaration(var_decl)) = parse_result.program.body.first() else {
        return defaults;
    };
    let Some(declarator) = var_decl.declarations.first() else {
        return defaults;
    };
    let Some(Expression::CallExpression(call)) = declarator.init.as_ref() else {
        return defaults;
    };
    let Expression::Identifier(callee) = &call.callee else {
        return defaults;
    };
    if callee.name.as_str() != "withDefaults" {
        return defaults;
    }

    let Some(Argument::ObjectExpression(obj)) = call.arguments.get(1) else {
        return defaults;
    };

    for property in obj.properties.iter() {
        let ObjectPropertyKind::ObjectProperty(prop) = property else {
            continue;
        };

        let key = match &prop.key {
            PropertyKey::StaticIdentifier(id) => id.name.to_compact_string(),
            PropertyKey::StringLiteral(lit) => lit.value.to_compact_string(),
            PropertyKey::NumericLiteral(lit) => lit.value.to_compact_string(),
            _ => continue,
        };

        let Some(value_start) = (prop.value.span().start as usize).checked_sub(WRAP_PREFIX.len())
        else {
            continue;
        };
        let Some(value_end) = (prop.value.span().end as usize).checked_sub(WRAP_PREFIX.len())
        else {
            continue;
        };
        if let Some(value_src) = trimmed.get(value_start..value_end) {
            defaults.insert(key, value_src.to_compact_string());
        }
    }

    defaults
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

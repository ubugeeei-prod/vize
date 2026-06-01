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

use crate::script::ScriptCompileContext;
use crate::types::SfcError;

/// Prop type information
#[derive(Debug, Clone)]
pub struct PropTypeInfo {
    /// JavaScript type constructor name (String, Number, Boolean, Array, Object, Function)
    pub js_type: String,
    /// Original TypeScript type (for PropType<T> usage)
    pub ts_type: Option<String>,
    /// Whether the prop is optional
    pub optional: bool,
    /// Whether the prop accepts null at runtime
    pub nullable: bool,
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
    let mut prev = '\0';

    for c in content.chars() {
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
                if prev == '=' {
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
        prev = c;
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

        let optional = name_part.ends_with('?') || type_includes_top_level_undefined(type_part);
        let nullable = type_includes_top_level_null(type_part);
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
                        nullable,
                    },
                ));
            }
        }
    }
}

fn type_includes_top_level_undefined(ts_type: &str) -> bool {
    split_type_at_top_level(ts_type.trim(), '|')
        .into_iter()
        .any(|part| part.trim() == "undefined")
}

fn type_includes_top_level_null(ts_type: &str) -> bool {
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
fn split_type_at_top_level(s: &str, delimiter: char) -> Vec<String> {
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
fn contains_top_level_arrow(s: &str) -> bool {
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
fn contains_top_level_colon(s: &str) -> bool {
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

/// Normalize default values from reactive props destructure for runtime props.
///
/// Vue treats array/object destructure defaults as per-instance factories, while
/// function defaults are already factories/values and must not be wrapped.
pub(crate) fn normalize_destructure_default_value(default_value: &str) -> String {
    let trimmed = default_value.trim();
    if trimmed.starts_with('[') {
        let mut wrapped = String::with_capacity(trimmed.len() + 6);
        wrapped.push_str("() => ");
        wrapped.push_str(trimmed);
        return wrapped;
    }

    if trimmed.starts_with('{') {
        let mut wrapped = String::with_capacity(trimmed.len() + 8);
        wrapped.push_str("() => (");
        wrapped.push_str(trimmed);
        wrapped.push(')');
        return wrapped;
    }

    default_value.to_compact_string()
}

/// Validate Vue-specific script-setup semantics that the TypeScript checker
/// cannot derive on its own (currently: prop destructure defaults whose value
/// disagrees with the declared prop type).
///
/// This runs only the analysis needed to drive the validators — no codegen,
/// no template compile — so check/LSP can call it on every SFC without paying
/// the cost of `compile_sfc`. Returns `Ok(())` when there is no `<script
/// setup>` or no actionable issue.
pub fn validate_script_setup_semantics(script_setup_content: &str) -> Result<(), SfcError> {
    // Delegating with a zero block offset yields block-relative diagnostic
    // locations. Callers that know where the `<script setup>` block sits in
    // the full SFC should prefer `validate_script_setup_semantics_located` so
    // diagnostics map to the correct document position.
    validate_script_setup_semantics_located(script_setup_content, 0, script_setup_content)
}

/// Like [`validate_script_setup_semantics`], but rebases diagnostic locations
/// onto the full SFC: `block_start` is the byte offset of the script-setup
/// block content within `sfc_source` (i.e. `script_setup.loc.start`), and
/// `sfc_source` is the whole SFC text used to resolve line/column. This lets
/// editor/check diagnostics point at the offending span (e.g. a destructure
/// default) instead of the start of the `<script setup>` block.
pub fn validate_script_setup_semantics_located(
    script_setup_content: &str,
    block_start: usize,
    sfc_source: &str,
) -> Result<(), SfcError> {
    if script_setup_content.is_empty() {
        return Ok(());
    }
    let mut ctx = ScriptCompileContext::new(script_setup_content);
    ctx.analyze();
    validate_props_destructure_default_types(&ctx, block_start, sfc_source)
}

/// Build a 1-based [`BlockLocation`] for an absolute byte span in `source`.
/// Columns count UTF-16 code units to match the LSP position convention.
fn block_location_for_span(source: &str, start: usize, end: usize) -> crate::types::BlockLocation {
    fn line_col_1based(source: &str, offset: usize) -> (usize, usize) {
        let target = offset.min(source.len());
        let mut line = 1usize;
        let mut column = 1usize;
        for (index, ch) in source.char_indices() {
            if index >= target {
                break;
            }
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += ch.len_utf16();
            }
        }
        (line, column)
    }
    let (start_line, start_column) = line_col_1based(source, start);
    let (end_line, end_column) = line_col_1based(source, end);
    crate::types::BlockLocation {
        start,
        end,
        tag_start: start,
        tag_end: end,
        start_line,
        start_column,
        end_line,
        end_column,
    }
}

/// Locate the byte span `(start, end)` of a reactive props-destructure
/// default's value expression for `prop_key`, relative to
/// `script_setup_content` (e.g. the `0` in `const { msg = 0 } = defineProps<…>()`).
///
/// Used to point the default-type-mismatch diagnostic at the offending
/// default rather than the start of the `<script setup>` block. Re-parses the
/// script-setup content; only the rare diagnostic error path calls this, so
/// the extra parse is not on the hot path. The span lives on the AST, not on
/// the public `PropsDestructureBinding`, to keep that type's API stable.
fn props_destructure_default_span(
    script_setup_content: &str,
    prop_key: &str,
) -> Option<(u32, u32)> {
    use oxc_ast::ast::BindingPattern;

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script_setup_content, SourceType::ts()).parse();
    for stmt in &parsed.program.body {
        let Statement::VariableDeclaration(var_decl) = stmt else {
            continue;
        };
        for decl in &var_decl.declarations {
            // Match `const { … } = defineProps(…)`.
            let BindingPattern::ObjectPattern(obj_pat) = &decl.id else {
                continue;
            };
            let is_define_props = matches!(
                &decl.init,
                Some(Expression::CallExpression(call))
                    if matches!(&call.callee, Expression::Identifier(id) if id.name == "defineProps")
            );
            if !is_define_props {
                continue;
            }
            for prop in obj_pat.properties.iter() {
                let key = match &prop.key {
                    PropertyKey::StaticIdentifier(id) => id.name.as_str(),
                    PropertyKey::StringLiteral(lit) => lit.value.as_str(),
                    _ => continue,
                };
                if key != prop_key {
                    continue;
                }
                if let BindingPattern::AssignmentPattern(assign) = &prop.value {
                    let span = assign.right.span();
                    return Some((span.start, span.end));
                }
            }
        }
    }
    None
}

/// Validate reactive props destructure defaults against inferred runtime prop types.
pub(crate) fn validate_props_destructure_default_types(
    ctx: &ScriptCompileContext,
    block_start: usize,
    sfc_source: &str,
) -> Result<(), SfcError> {
    let Some(destructure) = ctx.macros.props_destructure.as_ref() else {
        return Ok(());
    };
    let Some(props_macro) = ctx.macros.define_props.as_ref() else {
        return Ok(());
    };
    let Some(type_args) = props_macro.type_args.as_ref() else {
        return Ok(());
    };

    let resolved_type_args =
        crate::script::resolve_type_args(type_args, &ctx.interfaces, &ctx.type_aliases);
    let prop_types = extract_prop_types_from_type(&resolved_type_args);

    for (name, prop_type) in &prop_types {
        let Some(binding) = destructure.bindings.get(name.as_str()) else {
            continue;
        };
        let Some(default_value) = binding.default.as_deref() else {
            continue;
        };

        let resolved_js_type = if prop_type.js_type == "null" {
            prop_type
                .ts_type
                .as_ref()
                .and_then(|ts_type| {
                    resolve_prop_js_type(ts_type, &ctx.interfaces, &ctx.type_aliases)
                })
                .unwrap_or_else(|| prop_type.js_type.clone())
        } else {
            prop_type.js_type.clone()
        };

        if resolved_js_type == "null" || prop_type.nullable {
            continue;
        }

        let Some(default_type) =
            infer_default_value_runtime_type(default_value, resolved_js_type.as_str())
        else {
            continue;
        };

        if !runtime_type_includes(resolved_js_type.as_str(), default_type) {
            // Point the diagnostic at the offending default expression when we
            // can locate it, rebasing the block-relative span onto the full
            // SFC. Falls back to no location (callers then pick a block-start
            // fallback) only when the span cannot be resolved.
            let loc =
                props_destructure_default_span(&ctx.source, name).map(|(rel_start, rel_end)| {
                    block_location_for_span(
                        sfc_source,
                        block_start + rel_start as usize,
                        block_start + rel_end as usize,
                    )
                });
            return Err(SfcError {
                message: {
                    let mut message = String::from("Default value of prop \"");
                    message.push_str(name);
                    message.push_str("\" does not match declared type.");
                    message
                },
                code: Some("DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE".into()),
                loc,
            });
        }
    }

    Ok(())
}

fn infer_default_value_runtime_type(
    default_value: &str,
    expected_runtime_type: &str,
) -> Option<&'static str> {
    const WRAP_PREFIX: &str = "const __vize_default__ = ";
    let mut wrapped = String::with_capacity(WRAP_PREFIX.len() + default_value.len() + 1);
    wrapped.push_str(WRAP_PREFIX);
    wrapped.push_str(default_value);
    wrapped.push(';');

    let allocator = Allocator::default();
    let parse_result = Parser::new(
        &allocator,
        &wrapped,
        SourceType::default().with_typescript(true),
    )
    .parse();
    if !parse_result.errors.is_empty() {
        return None;
    }

    let Some(Statement::VariableDeclaration(var_decl)) = parse_result.program.body.first() else {
        return None;
    };
    let declarator = var_decl.declarations.first()?;
    let value = declarator.init.as_ref()?;

    infer_expression_runtime_type(unwrap_ts_expression(value), expected_runtime_type)
}

fn unwrap_ts_expression<'a>(expr: &'a Expression<'a>) -> &'a Expression<'a> {
    match expr {
        Expression::ParenthesizedExpression(paren) => unwrap_ts_expression(&paren.expression),
        Expression::TSAsExpression(ts_as) => unwrap_ts_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            unwrap_ts_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            unwrap_ts_expression(&ts_non_null.expression)
        }
        _ => expr,
    }
}

fn infer_expression_runtime_type(
    expr: &Expression<'_>,
    expected_runtime_type: &str,
) -> Option<&'static str> {
    match expr {
        Expression::StringLiteral(_) => Some("String"),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => Some("String"),
        Expression::NumericLiteral(_) => Some("Number"),
        Expression::BooleanLiteral(_) => Some("Boolean"),
        Expression::ObjectExpression(_) => Some("Object"),
        Expression::ArrayExpression(_) => Some("Array"),
        Expression::ArrowFunctionExpression(arrow) => {
            if runtime_type_includes(expected_runtime_type, "Function") {
                return Some("Function");
            }
            arrow_return_expression(arrow)
                .and_then(|expr| infer_expression_runtime_type(expr, expected_runtime_type))
        }
        Expression::FunctionExpression(func) => {
            if runtime_type_includes(expected_runtime_type, "Function") {
                return Some("Function");
            }
            function_body_return_expression(&func.body.as_ref()?.statements)
                .and_then(|expr| infer_expression_runtime_type(expr, expected_runtime_type))
        }
        _ => None,
    }
}

fn arrow_return_expression<'a>(
    arrow: &'a oxc_ast::ast::ArrowFunctionExpression<'a>,
) -> Option<&'a Expression<'a>> {
    if !arrow.expression {
        return function_body_return_expression(&arrow.body.statements);
    }

    let Statement::ExpressionStatement(expr_stmt) = arrow.body.statements.first()? else {
        return None;
    };
    Some(&expr_stmt.expression)
}

fn function_body_return_expression<'a>(
    statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
) -> Option<&'a Expression<'a>> {
    for stmt in statements.iter() {
        if let Statement::ReturnStatement(ret) = stmt
            && let Some(argument) = &ret.argument
        {
            return Some(argument);
        }
    }
    None
}

fn runtime_type_includes(runtime_type: &str, value_type: &str) -> bool {
    runtime_type
        .trim_matches(|c| c == '[' || c == ']')
        .split(',')
        .map(str::trim)
        .any(|part| part == value_type)
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

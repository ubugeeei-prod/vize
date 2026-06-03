//! Props type generation for virtual TypeScript.
//!
//! Generates `Props` type definitions and template-level prop variable
//! declarations from Vue SFC macro analysis.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, Expression, ObjectPropertyKind, PropertyKey};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::profile;
use vize_croquis::BindingType;
use vize_croquis::Croquis;
use vize_croquis::macros::MacroKind;

use super::helpers::{is_reserved_identifier, to_safe_identifier};

#[inline]
fn should_skip_template_prop_binding(summary: &Croquis, prop_name: &str) -> bool {
    if summary
        .macros
        .props_destructure()
        .and_then(|destructure| destructure.get(prop_name))
        .is_some_and(|binding| binding.local.as_str() == prop_name)
    {
        return true;
    }

    summary
        .bindings
        .get(prop_name)
        .is_some_and(|binding_type| !matches!(binding_type, BindingType::Props))
}

fn emit_template_prop_binding(
    ts: &mut String,
    props_type_ref: &str,
    prop_name: &str,
    has_default: bool,
) {
    let binding_name = to_safe_identifier(prop_name);
    if has_default {
        append!(
            *ts,
            "  const {binding_name} = props[\"{prop_name}\"] as Exclude<{props_type_ref}[\"{prop_name}\"], undefined>;\n"
        );
    } else {
        append!(*ts, "  const {binding_name} = props[\"{prop_name}\"];\n");
    }
    append!(*ts, "  void {binding_name};\n");
}

fn collect_with_defaults_default_names(summary: &Croquis) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    for call in summary.macros.all_calls() {
        if call.kind != MacroKind::WithDefaults {
            continue;
        }
        let Some(runtime_args) = &call.runtime_args else {
            continue;
        };
        collect_with_defaults_default_names_from_source(runtime_args.as_str(), &mut names);
    }
    names
}

fn collect_with_defaults_default_names_from_source(source: &str, names: &mut FxHashSet<String>) {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let Ok(Expression::CallExpression(call)) =
        Parser::new(&allocator, source, source_type).parse_expression()
    else {
        return;
    };
    let Expression::Identifier(callee) = &call.callee else {
        return;
    };
    if callee.name.as_str() != "withDefaults" {
        return;
    }
    let Some(Argument::ObjectExpression(defaults)) = call.arguments.get(1) else {
        return;
    };

    for prop in &defaults.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        if prop.computed {
            continue;
        }
        let Some(name) = property_key_name(&prop.key) else {
            continue;
        };
        names.insert(name.into());
    }
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(s) => Some(s.value.as_str()),
        _ => None,
    }
}

fn template_props_type_ref(
    base_type_ref: &str,
    defaulted_prop_names: &FxHashSet<String>,
) -> String {
    if defaulted_prop_names.is_empty() {
        return base_type_ref.into();
    }

    let mut names: Vec<&str> = defaulted_prop_names
        .iter()
        .map(|name| name.as_str())
        .collect();
    names.sort_unstable();

    let mut default_keys = String::default();
    for name in names {
        if !default_keys.is_empty() {
            default_keys.push_str(" | ");
        }
        append!(default_keys, "\"{name}\"");
    }

    cstr!("__WithDefaultsResult<{base_type_ref}, Pick<{base_type_ref}, {default_keys}>>")
}

/// Generate Props type definition at module level.
/// When `generic_param` is present (e.g., `"T extends Foo, P extends Bar"`),
/// the Props type is emitted with generic parameters: `export type Props<T, P> = ...;`
pub(crate) fn generate_props_type(ts: &mut String, summary: &Croquis, generic_param: Option<&str>) {
    let props = summary.macros.props();
    let has_props = !props.is_empty();
    let define_props_type_args = summary
        .macros
        .define_props()
        .and_then(|m| m.type_args.as_ref());
    let props_already_defined = summary
        .type_exports
        .iter()
        .any(|te| te.name.as_str() == "Props");

    // Build generic suffix for Props type declaration (with `= any` defaults)
    let generic_decl = generic_param
        .map(|g| {
            let with_defaults = add_generic_defaults(g);
            cstr!("<{with_defaults}>")
        })
        .unwrap_or_default();

    ts.push_str("// ========== Exported Types ==========\n");

    if props_already_defined {
        // User defined Props, no need to re-export
    } else if let Some(type_args) = define_props_type_args {
        let inner_type = type_args
            .strip_prefix('<')
            .and_then(|s| s.strip_suffix('>'))
            .unwrap_or(type_args.as_str());
        // Always emit Props alias so it's available in template and default export.
        append!(*ts, "export type Props{generic_decl} = {inner_type};\n");
    } else if has_props {
        append!(*ts, "export type Props{generic_decl} = {{\n");
        for prop in props {
            let prop_type = prop.prop_type.as_deref().unwrap_or("unknown");
            let optional = if prop.required { "" } else { "?" };
            append!(*ts, "  {}{optional}: {prop_type};\n", prop.name);
        }
        ts.push_str("};\n");
    } else {
        append!(*ts, "export type Props{generic_decl} = {{}};\n");
    }

    ts.push('\n');
}

/// Generate props variables inside template closure.
/// When `generic_param` is present, uses `Props<T, P>` instead of `Props`.
pub(crate) fn generate_props_variables(
    ts: &mut String,
    summary: &Croquis,
    script_content: Option<&str>,
    generic_param: Option<&str>,
) {
    let props = summary.macros.props();
    let has_props = !props.is_empty();
    let define_props_type_args = summary
        .macros
        .define_props()
        .and_then(|m| m.type_args.as_ref());

    // Build Props type reference with generic names (strip constraints)
    let props_type_ref = generic_param
        .map(|g| {
            let names = extract_generic_names(g);
            cstr!("Props<{names}>")
        })
        .unwrap_or_else(|| "Props".into());
    let defaulted_prop_names = collect_with_defaults_default_names(summary);
    let template_props_type_ref =
        template_props_type_ref(props_type_ref.as_str(), &defaulted_prop_names);

    if has_props || define_props_type_args.is_some() {
        ts.push_str("  // Props are available in template as variables\n");
        ts.push_str("  // Access via `propName` or `props.propName`\n");
        append!(
            *ts,
            "  const props: {template_props_type_ref} = {{}} as {template_props_type_ref};\n"
        );
        ts.push_str("  void props; // Mark as used to avoid TS6133\n");

        if has_props {
            // Runtime-declared props: generate individual variables
            for prop in props {
                if should_skip_template_prop_binding(summary, prop.name.as_str()) {
                    continue;
                }
                emit_template_prop_binding(
                    ts,
                    template_props_type_ref.as_str(),
                    prop.name.as_str(),
                    prop.default_value.is_some() || defaulted_prop_names.contains(&prop.name),
                );
            }
        } else if let Some(type_args) = define_props_type_args {
            // Type-only defineProps<TypeName>(): extract fields
            // type_args may include angle brackets (e.g., "<Props>", "<Foo<T>>"), strip outer pair
            let type_name = strip_outer_angle_brackets(type_args.trim());

            // Try TypeResolver first (handles inline object types and registered types)
            let type_properties = summary.types.extract_properties(type_name);
            if !type_properties.is_empty() {
                for prop in &type_properties {
                    if should_skip_template_prop_binding(summary, prop.name.as_str()) {
                        continue;
                    }
                    emit_template_prop_binding(
                        ts,
                        template_props_type_ref.as_str(),
                        prop.name.as_str(),
                        defaulted_prop_names.contains(&prop.name),
                    );
                }
            } else if let Some(script) = script_content {
                // Fallback: extract field names from script text (for local interfaces)
                let field_names = profile!(
                    "canon.virtual_ts.extract_interface_fields",
                    extract_interface_fields(script, type_name)
                );
                for field in &field_names {
                    if should_skip_template_prop_binding(summary, field.as_str()) {
                        continue;
                    }
                    emit_template_prop_binding(
                        ts,
                        template_props_type_ref.as_str(),
                        field.as_str(),
                        defaulted_prop_names.contains(field),
                    );
                }
            }
        }
        ts.push('\n');
    }
}

pub(crate) fn collect_template_prop_names(
    summary: &Croquis,
    script_content: Option<&str>,
) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    let props = summary.macros.props();
    if !props.is_empty() {
        for prop in props {
            if should_skip_template_prop_binding(summary, prop.name.as_str()) {
                continue;
            }
            if !is_reserved_identifier(prop.name.as_str()) {
                continue;
            }
            names.insert(prop.name.as_str().into());
        }
        return names;
    }

    let Some(type_args) = summary
        .macros
        .define_props()
        .and_then(|m| m.type_args.as_ref())
    else {
        return names;
    };
    let type_name = strip_outer_angle_brackets(type_args.trim());
    let type_properties = summary.types.extract_properties(type_name);
    if !type_properties.is_empty() {
        for prop in &type_properties {
            if should_skip_template_prop_binding(summary, prop.name.as_str()) {
                continue;
            }
            if !is_reserved_identifier(prop.name.as_str()) {
                continue;
            }
            names.insert(prop.name.as_str().into());
        }
        return names;
    }

    let Some(script) = script_content else {
        return names;
    };
    let field_names = profile!(
        "canon.virtual_ts.extract_interface_fields_for_expressions",
        extract_interface_fields(script, type_name)
    );
    for field in &field_names {
        if should_skip_template_prop_binding(summary, field.as_str()) {
            continue;
        }
        if !is_reserved_identifier(field.as_str()) {
            continue;
        }
        names.insert(field.clone());
    }
    names
}

/// Extract field names from an interface or type literal in script content.
/// Fallback for when TypeResolver doesn't have the type registered.
pub(crate) fn extract_interface_fields(script: &str, type_name: &str) -> Vec<String> {
    let mut fields = Vec::new();

    let body = if type_name.starts_with('{') {
        Some(type_name)
    } else {
        find_type_body(script, type_name)
    };

    if let Some(body) = body {
        let inner = if let Some(start) = body.find('{') {
            let end = find_matching_brace(body, start);
            &body[start + 1..end]
        } else {
            body
        };

        for line in inner.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with("//")
                || trimmed.starts_with("/*")
                || trimmed == "}"
                || trimmed == "};"
            {
                continue;
            }
            let trimmed = trimmed.strip_prefix("readonly ").unwrap_or(trimmed);
            if let Some(colon_pos) = trimmed.find(':') {
                let field_name = trimmed[..colon_pos].trim().trim_end_matches('?');
                if !field_name.is_empty()
                    && field_name
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
                {
                    fields.push(field_name.into());
                }
            }
        }
    }

    fields
}

/// Strip the outermost `<...>` pair from a type_args string, handling nested generics.
/// e.g., `"<Props>"` → `"Props"`, `"<Foo<T>>"` → `"Foo<T>"`, `"Props"` → `"Props"`
fn strip_outer_angle_brackets(s: &str) -> &str {
    let s = s.trim();
    if !s.starts_with('<') {
        return s;
    }
    // Find the matching '>' for the opening '<'
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 && i == s.len() - 1 {
                    // The opening '<' matches the final '>' — strip them
                    return &s[1..i];
                }
            }
            _ => {}
        }
    }
    s
}

/// Strip generic parameters from a type name for interface lookup.
/// e.g., `"ContextMenuContentProps<T>"` → `"ContextMenuContentProps"`
fn strip_generic_params(type_name: &str) -> &str {
    match type_name.find('<') {
        Some(pos) => &type_name[..pos],
        None => type_name,
    }
}

/// Find the body of an interface or type declaration in script content.
fn find_type_body<'a>(script: &'a str, type_name: &str) -> Option<&'a str> {
    // Strip generic params from name for searching (e.g., "Foo<T>" → "Foo")
    let base_name = strip_generic_params(type_name);
    for pattern in &[
        cstr!("interface {base_name} "),
        cstr!("interface {base_name}{{"),
        cstr!("interface {base_name}<"),
        cstr!("type {base_name} "),
        cstr!("type {base_name}<"),
    ] {
        if let Some(pos) = script.find(pattern.as_str()) {
            let rest = &script[pos..];
            if let Some(brace_start) = rest.find('{') {
                let end = find_matching_brace(rest, brace_start);
                return Some(&rest[..end + 1]);
            }
        }
    }
    None
}

/// Find the matching closing brace for an opening brace at `start`.
fn find_matching_brace(s: &str, start: usize) -> usize {
    let mut depth = 0;
    for (i, c) in s[start..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return start + i;
                }
            }
            _ => {}
        }
    }
    s.len().saturating_sub(1)
}

/// Extract just the generic parameter names from a full generic declaration.
/// e.g., `"T extends Foo, P extends Bar"` → `"T, P"`
/// e.g., `"T"` → `"T"`
/// e.g., `"T extends Record<string, any>, U"` → `"T, U"`
pub(crate) fn extract_generic_names(generic_param: &str) -> String {
    let mut names = String::default();
    let mut depth = 0i32; // track <> nesting
    let mut current_name = String::default();
    let mut in_extends = false;

    for ch in generic_param.chars() {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                let trimmed = current_name.trim();
                if !trimmed.is_empty() {
                    // Extract just the name (before "extends")
                    let name = trimmed.split_whitespace().next().unwrap_or(trimmed);
                    if !names.is_empty() {
                        names.push_str(", ");
                    }
                    names.push_str(name);
                }
                current_name = String::default();
                in_extends = false;
                continue;
            }
            _ => {}
        }
        if depth == 0 {
            current_name.push(ch);
        }
    }

    // Handle the last parameter
    let trimmed = current_name.trim();
    if !trimmed.is_empty() {
        let name = trimmed.split_whitespace().next().unwrap_or(trimmed);
        if !names.is_empty() {
            names.push_str(", ");
        }
        names.push_str(name);
    }

    let _ = in_extends;
    names
}

/// Add `= any` defaults to each generic parameter that doesn't already have a default.
/// e.g., `"T extends Foo, P"` → `"T extends Foo = any, P = any"`
/// e.g., `"T = string"` → `"T = string"` (unchanged, already has default)
pub(crate) fn add_generic_defaults(generic_param: &str) -> String {
    let mut result = String::default();
    let mut depth = 0i32;
    let mut current_param = String::default();

    for ch in generic_param.chars() {
        match ch {
            '<' => {
                depth += 1;
                current_param.push(ch);
            }
            '>' => {
                depth -= 1;
                current_param.push(ch);
            }
            ',' if depth == 0 => {
                append_param_with_default(&mut result, current_param.trim());
                result.push_str(", ");
                current_param = String::default();
            }
            _ => {
                current_param.push(ch);
            }
        }
    }

    // Handle the last parameter
    let trimmed = current_param.trim();
    if !trimmed.is_empty() {
        append_param_with_default(&mut result, trimmed);
    }

    result
}

/// Append a single generic parameter with `= any` default if it doesn't have one.
fn append_param_with_default(result: &mut String, param: &str) {
    result.push_str(param);
    // Check if this param already has a default (contains `=` at depth 0)
    let mut depth = 0i32;
    let has_default = param.chars().any(|ch| {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            '=' if depth == 0 => return true,
            _ => {}
        }
        false
    });
    if !has_default {
        result.push_str(" = any");
    }
}

#[cfg(test)]
mod tests {
    use super::{collect_with_defaults_default_names_from_source, template_props_type_ref};
    use vize_carton::{FxHashSet, String};

    #[test]
    fn collects_with_defaults_object_keys() {
        let mut names = FxHashSet::default();
        collect_with_defaults_default_names_from_source(
            r#"withDefaults(defineProps<Props>(), {
  thickness: 0.1,
  "label": "ok",
  ...moreDefaults,
  [dynamicKey]: 1,
})"#,
            &mut names,
        );

        assert!(names.contains("thickness"));
        assert!(names.contains("label"));
        assert!(!names.contains("dynamicKey"));
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn builds_deterministic_with_defaults_props_type() {
        let mut names: FxHashSet<String> = FxHashSet::default();
        names.insert("label".into());
        names.insert("thickness".into());

        assert_eq!(
            template_props_type_ref("Props", &names),
            r#"__WithDefaultsResult<Props, Pick<Props, "label" | "thickness">>"#
        );
    }
}

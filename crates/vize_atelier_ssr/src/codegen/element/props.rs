//! Shared JavaScript expression and prop-object builders for SSR element codegen.

use super::*;

/// Build an object literal from normalized prop entries.
pub(super) fn component_props_object(entries: &[VNodePropEntry]) -> String {
    let mut out = String::from("{ ");
    for (index, entry) in entries.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_component_prop_entry(&mut out, entry);
    }
    out.push_str(" }");
    out
}

pub(super) fn slot_props_pattern_to_string(expr: &ExpressionNode) -> String {
    let source = match expr {
        ExpressionNode::Simple(simple) => simple.loc.source.clone(),
        ExpressionNode::Compound(compound) => compound.loc.source.clone(),
    };
    vize_atelier_core::transforms::strip_typescript_from_expression(&source)
}

pub(super) fn component_prop_entry(key: &str, value: &str, dynamic: bool) -> VNodePropEntry {
    VNodePropEntry {
        key: key.to_compact_string(),
        value: value.to_compact_string(),
        dynamic,
    }
}

pub(super) fn push_component_prop_entry(out: &mut String, entry: &VNodePropEntry) {
    if entry.dynamic {
        out.push('[');
        out.push_str(&entry.key);
        out.push_str(" || \"\"");
        out.push_str("]: ");
    } else {
        push_js_object_key(out, &entry.key);
        out.push_str(": ");
    }
    out.push_str(&entry.value);
}

/// Merge static `class` and `style` entries so Vue sees one canonical value.
pub(super) fn normalize_prop_entries(
    entries: std::vec::Vec<VNodePropEntry>,
) -> std::vec::Vec<VNodePropEntry> {
    let mut normalized = std::vec::Vec::with_capacity(entries.len());
    let mut class_values = std::vec::Vec::new();
    let mut style_values = std::vec::Vec::new();

    for entry in entries {
        if !entry.dynamic && entry.key == "class" {
            class_values.push(entry.value);
        } else if !entry.dynamic && entry.key == "style" {
            style_values.push(entry.value);
        } else {
            normalized.push(entry);
        }
    }

    if !class_values.is_empty() {
        normalized.push(component_prop_entry(
            "class",
            &merge_prop_values(class_values),
            false,
        ));
    }
    if !style_values.is_empty() {
        normalized.push(component_prop_entry(
            "style",
            &merge_prop_values(style_values),
            false,
        ));
    }

    normalized
}

pub(super) fn merge_prop_values(values: std::vec::Vec<String>) -> String {
    if values.len() == 1 {
        return values.into_iter().next().unwrap_or_default();
    }

    let mut out = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(value);
    }
    out.push(']');
    out
}

/// Apply `v-bind` key modifiers before emitting the JavaScript object key.
pub(super) fn transform_bound_prop_key(key: &str, dir: &DirectiveNode) -> String {
    if dir.modifiers.iter().any(|m| m.content == "camel") {
        return vize_carton::camelize(key);
    }
    if dir.modifiers.iter().any(|m| m.content == "prop") {
        let mut out = String::from(".");
        out.push_str(key);
        return out;
    }
    if dir.modifiers.iter().any(|m| m.content == "attr") {
        let mut out = String::from("^");
        out.push_str(key);
        return out;
    }
    key.to_compact_string()
}

pub(super) fn push_js_object_key(out: &mut String, key: &str) {
    if is_valid_js_identifier(key) {
        out.push_str(key);
        return;
    }

    out.push('"');
    out.push_str(&escape_js_string(key));
    out.push('"');
}

pub(super) fn is_valid_js_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
        return false;
    }

    chars.all(|c| c == '_' || c == '$' || c.is_ascii_alphanumeric())
}

pub(super) fn is_simple_identifier(value: &str) -> bool {
    is_valid_js_identifier(value) && !matches!(value, "true" | "false" | "null" | "undefined")
}

pub(super) fn is_dynamic_component_tag(tag: &str) -> bool {
    matches!(tag, "component" | "Component")
}

pub(super) fn is_static_named_prop(prop: &PropNode, name: &str) -> bool {
    match prop {
        PropNode::Attribute(attr) => attr.name == name,
        PropNode::Directive(dir) if dir.name == "bind" => {
            matches!(&dir.arg, Some(ExpressionNode::Simple(arg)) if arg.is_static && arg.content == name)
        }
        _ => false,
    }
}

/// Quote a string for generated JavaScript string-literal positions.
pub(super) fn quoted_js_string(value: &str) -> String {
    let mut out = String::from("\"");
    out.push_str(&escape_js_string(value));
    out.push('"');
    out
}

pub(super) fn escape_js_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

pub(super) fn wrap_call(callee: &str, arg: &str) -> String {
    let mut out = String::from(callee);
    out.push('(');
    out.push_str(arg);
    out.push(')');
    out
}

use vize_carton::{String, ToCompactString};

use crate::BindingMetadata;

pub(super) fn rewrite_bound_component_resolution(
    line: &str,
    bindings: Option<&BindingMetadata>,
) -> Option<String> {
    let bindings = bindings?;
    let trimmed = line.trim_start();
    if !trimmed.starts_with("const _component_") {
        return None;
    }

    let resolve_start = trimmed.find(" = _resolveComponent(\"")?;
    let tag_start = resolve_start + " = _resolveComponent(\"".len();
    let tag_end = trimmed[tag_start..].find("\")")? + tag_start;
    let tag = &trimmed[tag_start..tag_end];
    let binding_name = resolve_component_binding_name(bindings, tag)?;
    let indent_len = line.len().saturating_sub(trimmed.len());
    let mut rewritten = String::with_capacity(line.len() + binding_name.len() + 5);
    rewritten.push_str(&line[..indent_len]);
    rewritten.push_str(&trimmed[..resolve_start]);
    rewritten.push_str(" = _ctx.");
    rewritten.push_str(&binding_name);
    Some(rewritten)
}

fn resolve_component_binding_name(bindings: &BindingMetadata, tag: &str) -> Option<String> {
    let resolve_base = |name: &str| {
        if bindings.bindings.contains_key(name) {
            return Some(name.to_compact_string());
        }
        let camel = camelize_component_name(name);
        if bindings.bindings.contains_key(camel.as_str()) {
            return Some(camel);
        }
        let pascal = capitalize_component_name(camel.as_str());
        bindings
            .bindings
            .contains_key(pascal.as_str())
            .then_some(pascal)
    };

    if let Some((base, suffix)) = tag.split_once('.') {
        let resolved_base = resolve_base(base)?;
        let mut resolved = String::with_capacity(resolved_base.len() + suffix.len() + 1);
        resolved.push_str(resolved_base.as_str());
        resolved.push('.');
        resolved.push_str(suffix);
        return Some(resolved);
    }
    resolve_base(tag)
}

fn camelize_component_name(tag: &str) -> String {
    let mut result = String::with_capacity(tag.len());
    let mut uppercase_next = false;
    for ch in tag.chars() {
        if ch == '-' {
            uppercase_next = true;
        } else if uppercase_next {
            result.push(ch.to_ascii_uppercase());
            uppercase_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

fn capitalize_component_name(tag: &str) -> String {
    let mut chars = tag.chars();
    let Some(first) = chars.next() else {
        return String::default();
    };
    let mut result = String::with_capacity(tag.len());
    result.push(first.to_ascii_uppercase());
    result.extend(chars);
    result
}

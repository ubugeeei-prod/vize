//! Vapor mode template compilation.

use super::string_tracking::{count_braces_with_state, StringTrackState};
use vize_atelier_vapor::{compile_vapor, VaporCompilerOptions};
use vize_carton::{Bump, String, ToCompactString};

use crate::types::{BindingMetadata, SfcError, SfcTemplateBlock};

/// Compile template block using Vapor mode
pub(crate) fn compile_template_block_vapor(
    template: &SfcTemplateBlock,
    scope_id: &str,
    has_scoped: bool,
    bindings: Option<&BindingMetadata>,
    custom_renderer: bool,
) -> Result<String, SfcError> {
    let allocator = Bump::new();

    // Build Vapor compiler options
    let vapor_opts = VaporCompilerOptions {
        prefix_identifiers: false,
        ssr: false,
        binding_metadata: bindings.cloned(),
        custom_renderer,
        ..Default::default()
    };

    // Compile template with Vapor
    let result = compile_vapor(&allocator, &template.content, vapor_opts);

    if !result.error_messages.is_empty() {
        let mut message = String::from("Vapor template compilation errors: ");
        use std::fmt::Write as _;
        let _ = write!(&mut message, "{:?}", result.error_messages);
        return Err(SfcError {
            message,
            code: Some("VAPOR_TEMPLATE_ERROR".to_compact_string()),
            loc: Some(template.loc.clone()),
        });
    }

    // Process the Vapor output to extract imports and render function
    let scope_attr = if has_scoped {
        let mut attr = String::with_capacity(scope_id.len() + 7);
        attr.push_str("data-v-");
        attr.push_str(scope_id);
        attr
    } else {
        String::default()
    };

    transform_vapor_template_output(
        &result.code,
        has_scoped.then_some(scope_attr.as_str()),
        template,
        bindings,
    )
}

/// Add scope ID to template string
pub(super) fn add_scope_id_to_template(template_line: &str, scope_id: &str) -> String {
    // Find the template string content and add scope_id to the first element
    if let Some(start) = template_line.find("\"<") {
        if let Some(end) = template_line.rfind(">\"") {
            let prefix = &template_line[..start + 2]; // up to and including "<"
            let content = &template_line[start + 2..end + 1]; // element content
            let suffix = &template_line[end + 1..]; // closing quote and paren

            // Find end of first tag name
            if let Some(tag_end) = content.find(|c: char| c.is_whitespace() || c == '>') {
                let tag_name = &content[..tag_end];
                let rest = &content[tag_end..];

                // Insert scope_id attribute after tag name
                let mut result = String::with_capacity(
                    prefix.len() + tag_name.len() + scope_id.len() + rest.len() + suffix.len() + 1,
                );
                result.push_str(prefix);
                result.push_str(tag_name);
                result.push(' ');
                result.push_str(scope_id);
                result.push_str(rest);
                result.push_str(suffix);
                return result;
            }
        }
    }
    template_line.to_compact_string()
}

fn rewrite_vapor_import(line: &str) -> String {
    if line.contains("'vue/vapor'") {
        line.replace("'vue/vapor'", "'vue'").into()
    } else if line.contains("\"vue/vapor\"") {
        line.replace("\"vue/vapor\"", "\"vue\"").into()
    } else {
        line.to_compact_string()
    }
}

fn is_render_signature(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("export function render(")
        || trimmed.starts_with("function render(")
        || trimmed.starts_with("export default")
}

pub(super) fn transform_vapor_template_output(
    code: &str,
    scope_attr: Option<&str>,
    template: &SfcTemplateBlock,
    bindings: Option<&BindingMetadata>,
) -> Result<String, SfcError> {
    let lines: Vec<&str> = code.lines().collect();
    let mut output = String::default();
    let mut index = 0usize;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        if trimmed.starts_with("import ") {
            output.push_str(&rewrite_vapor_import(line));
            output.push('\n');
            index += 1;
            continue;
        }
        if trimmed.is_empty() {
            index += 1;
            continue;
        }
        break;
    }

    let mut found_render = false;
    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        if is_render_signature(trimmed) {
            found_render = true;
            break;
        }

        if trimmed.starts_with("const t") && trimmed.contains("_template(") {
            if let Some(scope_id) = scope_attr {
                output.push_str(&add_scope_id_to_template(line, scope_id));
            } else {
                output.push_str(line);
            }
            output.push('\n');
        } else {
            output.push_str(line);
            output.push('\n');
        }
        index += 1;
    }

    if !found_render {
        return Err(SfcError {
            message: "Vapor template output is missing a render function".to_compact_string(),
            code: Some("VAPOR_TEMPLATE_ERROR".to_compact_string()),
            loc: Some(template.loc.clone()),
        });
    }

    output.push_str("function render(_ctx, $props, $emit, $attrs, $slots) {\n");

    let mut brace_state = StringTrackState::default();
    let mut brace_depth = count_braces_with_state(lines[index], &mut brace_state);
    index += 1;

    while index < lines.len() && brace_depth > 0 {
        let line = lines[index];
        let next_depth = brace_depth + count_braces_with_state(line, &mut brace_state);
        if !(next_depth == 0 && line.trim() == "}") {
            if let Some(rewritten) = rewrite_bound_component_resolution(line, bindings) {
                output.push_str(&rewritten);
            } else {
                output.push_str(line);
            }
            output.push('\n');
        }
        brace_depth = next_depth;
        index += 1;
    }

    output.push_str("}\n");

    Ok(output)
}

fn rewrite_bound_component_resolution(
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
    let binding_expr = {
        let mut expr = String::with_capacity(binding_name.len() + 5);
        expr.push_str("_ctx.");
        expr.push_str(&binding_name);
        expr
    };

    let mut rewritten = String::with_capacity(line.len() + binding_expr.len());
    rewritten.push_str(&line[..indent_len]);
    rewritten.push_str(&trimmed[..resolve_start]);
    rewritten.push_str(" = ");
    rewritten.push_str(&binding_expr);
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
        if bindings.bindings.contains_key(pascal.as_str()) {
            return Some(pascal);
        }

        None
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
            continue;
        }

        if uppercase_next {
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
    for ch in chars {
        result.push(ch);
    }
    result
}

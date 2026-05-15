//! Text code generation for Vapor mode.

use super::block::GenerateContext;
use crate::ir::SetTextIRNode;
use vize_carton::{cstr, String, ToCompactString};

/// Generate SetText code
pub fn generate_set_text(ctx: &mut GenerateContext, set_text: &SetTextIRNode<'_>) {
    let element = cstr!("_n{}", set_text.element);

    let values: Vec<String> = set_text
        .values
        .iter()
        .map(|v| {
            if v.is_static {
                cstr!("\"{}\"", escape_text(&v.content))
            } else {
                v.content.to_compact_string()
            }
        })
        .collect();

    if values.len() == 1 {
        ctx.push_line_fmt(format_args!("_setText({element}, {})", values[0]));
    } else if values.is_empty() {
        ctx.push_line_fmt(format_args!("_setText({element}, \"\")"));
    } else {
        ctx.push_line_fmt(format_args!("_setText({element}, {})", values.join(" + ")));
    }
}

/// Generate text content assignment
pub fn generate_text_content(element_var: &str, content: &str, is_static: bool) -> String {
    if is_static {
        cstr!("{element_var}.textContent = \"{}\"", escape_text(content))
    } else {
        cstr!("{element_var}.textContent = {content}")
    }
}

/// Generate createTextNode
pub fn generate_create_text_node(content: &str, is_static: bool) -> String {
    if is_static {
        cstr!("document.createTextNode(\"{}\")", escape_text(content))
    } else {
        cstr!("document.createTextNode({content})")
    }
}

/// Generate toDisplayString call
pub fn generate_to_display_string(expr: &str) -> String {
    cstr!("_toDisplayString({expr})")
}

/// Escape text for JavaScript string
fn escape_text(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .into()
}

/// Build text expression from multiple parts
pub fn build_text_expression(parts: &[(bool, &str)]) -> String {
    if parts.is_empty() {
        return String::from("\"\"");
    }

    if parts.len() == 1 {
        let (is_static, content) = parts[0];
        if is_static {
            return cstr!("\"{}\"", escape_text(content));
        } else {
            return generate_to_display_string(content);
        }
    }

    let exprs: Vec<String> = parts
        .iter()
        .map(|(is_static, content)| {
            if *is_static {
                cstr!("\"{}\"", escape_text(content))
            } else {
                generate_to_display_string(content)
            }
        })
        .collect();

    exprs.join(" + ").into()
}

/// Check if text node can be inlined into template
pub fn can_inline_text(content: &str) -> bool {
    // Can inline if pure text without special handling needed
    !content.contains("{{") && !content.contains('\n')
}

#[cfg(test)]
mod tests {
    use super::{build_text_expression, escape_text, generate_to_display_string};

    #[test]
    fn test_escape_text() {
        assert_eq!(escape_text("hello"), "hello");
        assert_eq!(escape_text("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_text("hello\"world"), "hello\\\"world");
    }

    #[test]
    fn test_build_text_expression_static() {
        let parts = vec![(true, "hello")];
        let result = build_text_expression(&parts);
        assert_eq!(result, "\"hello\"");
    }

    #[test]
    fn test_build_text_expression_dynamic() {
        let parts = vec![(false, "msg")];
        let result = build_text_expression(&parts);
        assert_eq!(result, "_toDisplayString(msg)");
    }

    #[test]
    fn test_build_text_expression_mixed() {
        let parts = vec![(true, "Hello "), (false, "name"), (true, "!")];
        let result = build_text_expression(&parts);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_generate_to_display_string() {
        let result = generate_to_display_string("value");
        assert_eq!(result, "_toDisplayString(value)");
    }
}

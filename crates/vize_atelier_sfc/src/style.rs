//! Style block processing and scoped CSS.

use vize_carton::{String, ToCompactString};

use crate::types::{SfcError, SfcStyleBlock, StyleCompileOptions};

/// Compile a style block
pub fn compile_style(
    style: &SfcStyleBlock,
    options: &StyleCompileOptions,
) -> Result<String, SfcError> {
    let mut output: String = style.content.to_compact_string();

    // Apply scoped transformation if needed
    if style.scoped || options.scoped {
        output = apply_scoped_css(&output, &options.id);
    }

    // Trim if requested
    if options.trim {
        output = output.trim().to_compact_string();
    }

    Ok(output)
}

/// Apply scoped CSS transformation
pub fn apply_scoped_css(css: &str, scope_id: &str) -> String {
    let mut attr_selector = String::with_capacity(scope_id.len() + 2);
    attr_selector.push('[');
    attr_selector.push_str(scope_id);
    attr_selector.push(']');
    let mut output = String::with_capacity(css.len() * 2);
    let mut chars = css.chars().peekable();
    let mut in_selector = true;
    let mut in_string = false;
    let mut string_char = '"';
    let mut in_comment = false;
    let mut in_at_rule = false; // Track if we're in an at-rule header
    let mut brace_depth: u32 = 0;
    let mut at_rule_depth: u32 = 0; // Track nested at-rule depth
    let mut last_selector_end = 0;
    let mut current = String::default();
    let mut pending_keyframes = false;
    let mut keyframes_brace_depth: Option<u32> = None;
    let mut saved_at_rule_depth: Option<u32> = None;

    while let Some(c) = chars.next() {
        current.push(c);

        if in_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                current.push(chars.next().unwrap());
                in_comment = false;
            }
            continue;
        }

        if in_string {
            if c == string_char && !current.ends_with("\\\"") && !current.ends_with("\\'") {
                in_string = false;
            }
            if !in_selector && !in_at_rule {
                output.push(c);
            }
            continue;
        }

        match c {
            '"' | '\'' => {
                in_string = true;
                string_char = c;
                if !in_selector && !in_at_rule {
                    output.push(c);
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                current.push(chars.next().unwrap());
                in_comment = true;
            }
            '{' => {
                brace_depth += 1;
                if in_at_rule {
                    // End of at-rule header (e.g., @media (...) {)
                    let at_rule_part = &current[last_selector_end..current.len() - 1];
                    output.push_str(at_rule_part.trim());
                    output.push('{');
                    in_at_rule = false;
                    if pending_keyframes {
                        saved_at_rule_depth = Some(at_rule_depth);
                        keyframes_brace_depth = Some(brace_depth);
                        pending_keyframes = false;
                    }
                    at_rule_depth = brace_depth;
                    in_selector = true;
                    last_selector_end = current.len();
                } else if keyframes_brace_depth.is_some_and(|d| brace_depth > d) {
                    // Inside @keyframes: stops (from/to/0%/100%) are not selectors
                    let kf_part = &current[last_selector_end..current.len() - 1];
                    output.push_str(kf_part.trim());
                    output.push('{');
                    in_selector = false;
                    last_selector_end = current.len();
                } else if in_selector && brace_depth == 1 {
                    // End of selector at root level, apply scope
                    let selector_part = &current[last_selector_end..current.len() - 1];
                    output.push_str(&scope_selector(selector_part.trim(), &attr_selector));
                    output.push('{');
                    in_selector = false;
                    last_selector_end = current.len();
                } else if in_selector && at_rule_depth > 0 && brace_depth > at_rule_depth {
                    // End of selector inside at-rule (e.g., inside @media), apply scope
                    let selector_part = &current[last_selector_end..current.len() - 1];
                    output.push_str(&scope_selector(selector_part.trim(), &attr_selector));
                    output.push('{');
                    in_selector = false;
                    last_selector_end = current.len();
                } else {
                    output.push(c);
                }
            }
            '}' => {
                brace_depth -= 1;
                output.push(c);
                // Check @keyframes block end — restore parent at_rule_depth
                if keyframes_brace_depth.is_some_and(|d| brace_depth < d) {
                    keyframes_brace_depth = None;
                    if let Some(saved) = saved_at_rule_depth.take() {
                        at_rule_depth = saved;
                    }
                }
                if brace_depth == 0 {
                    in_selector = true;
                    at_rule_depth = 0;
                    last_selector_end = current.len();
                } else if at_rule_depth > 0 && brace_depth >= at_rule_depth {
                    // Inside at-rule, back to selector mode for next rule
                    in_selector = true;
                    last_selector_end = current.len();
                }
            }
            '@' if in_selector => {
                // Start of at-rule (e.g., @media, @keyframes, @supports)
                in_at_rule = true;
                in_selector = false;
                // Look ahead to detect @keyframes (including vendor prefixes)
                let css_remaining = &css[current.len()..];
                pending_keyframes = css_remaining.starts_with("keyframes")
                    || css_remaining.starts_with("-webkit-keyframes")
                    || css_remaining.starts_with("-moz-keyframes")
                    || css_remaining.starts_with("-o-keyframes");
            }
            ';' if in_at_rule => {
                // Statement at-rule (e.g., @import, @charset, @namespace)
                // Flush the entire at-rule including the semicolon
                let stmt = &current[last_selector_end..];
                output.push_str(stmt.trim());
                output.push('\n');
                in_at_rule = false;
                in_selector = true;
                pending_keyframes = false;
                last_selector_end = current.len();
            }
            _ if in_selector || in_at_rule => {
                // Still building selector or at-rule header
            }
            _ => {
                output.push(c);
            }
        }
    }

    // Handle any remaining content
    if !current[last_selector_end..].is_empty() && in_selector {
        output.push_str(&current[last_selector_end..]);
    }

    output
}

/// Add scope attribute to a selector
fn scope_selector(selector: &str, attr_selector: &str) -> String {
    // Handle multiple selectors separated by comma
    selector
        .split(',')
        .map(|s| scope_single_selector(s.trim(), attr_selector))
        .collect::<Vec<_>>()
        .join(", ")
        .into()
}

/// Add scope attribute to a single selector
fn scope_single_selector(selector: &str, attr_selector: &str) -> String {
    if selector.is_empty() {
        return selector.to_compact_string();
    }

    // Handle :deep(), :slotted(), :global()
    if selector.contains(":deep(") {
        return transform_deep(selector, attr_selector);
    }

    if selector.contains(":slotted(") {
        return transform_slotted(selector, attr_selector);
    }

    if selector.contains(":global(") {
        return transform_global(selector);
    }

    // Find the last simple selector to append the attribute
    let parts: Vec<&str> = selector.split_whitespace().collect();
    if parts.is_empty() {
        return selector.to_compact_string();
    }

    // Add scope to the last part
    let mut result = String::default();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            result.push(' ');
        }

        if i == parts.len() - 1 {
            // Last part - add scope
            result.push_str(&add_scope_to_element(part, attr_selector));
        } else {
            result.push_str(part);
        }
    }

    result
}

/// Add scope attribute to an element selector
fn add_scope_to_element(selector: &str, attr_selector: &str) -> String {
    // Handle pseudo-elements and pseudo-classes
    if let Some(pseudo_pos) = selector.find("::") {
        let (before, after) = selector.split_at(pseudo_pos);
        let mut result = String::with_capacity(before.len() + attr_selector.len() + after.len());
        result.push_str(before);
        result.push_str(attr_selector);
        result.push_str(after);
        return result;
    }

    if let Some(pseudo_pos) = selector.rfind(':') {
        // Check if it's a pseudo-class (not part of element name)
        let before = &selector[..pseudo_pos];
        if !before.is_empty() && !before.ends_with('\\') {
            let after = &selector[pseudo_pos..];
            let mut result =
                String::with_capacity(before.len() + attr_selector.len() + after.len());
            result.push_str(before);
            result.push_str(attr_selector);
            result.push_str(after);
            return result;
        }
    }

    let mut result = String::with_capacity(selector.len() + attr_selector.len());
    result.push_str(selector);
    result.push_str(attr_selector);
    result
}

/// Transform :deep() to descendant selector
fn transform_deep(selector: &str, attr_selector: &str) -> String {
    // :deep(.child) -> [data-v-xxx] .child
    if let Some(start) = selector.find(":deep(") {
        let before = &selector[..start];
        let after = &selector[start + 6..];

        if let Some(end) = after.find(')') {
            let inner = &after[..end];
            let rest = &after[end + 1..];

            let scoped_before = if before.is_empty() {
                attr_selector.to_compact_string()
            } else {
                let trimmed = before.trim();
                let mut result = String::with_capacity(trimmed.len() + attr_selector.len());
                result.push_str(trimmed);
                result.push_str(attr_selector);
                result
            };

            let mut result =
                String::with_capacity(scoped_before.len() + inner.len() + rest.len() + 1);
            result.push_str(&scoped_before);
            result.push(' ');
            result.push_str(inner);
            result.push_str(rest);
            return result;
        }
    }

    selector.to_compact_string()
}

/// Transform :slotted() for slot content
fn transform_slotted(selector: &str, attr_selector: &str) -> String {
    // :slotted(.child) -> .child[data-v-xxx-s]
    if let Some(start) = selector.find(":slotted(") {
        let after = &selector[start + 9..];

        if let Some(end) = after.find(')') {
            let inner = &after[..end];
            let rest = &after[end + 1..];

            let mut result =
                String::with_capacity(inner.len() + attr_selector.len() + rest.len() + 2);
            result.push_str(inner);
            result.push_str(attr_selector);
            result.push_str("-s");
            result.push_str(rest);
            return result;
        }
    }

    selector.to_compact_string()
}

/// Transform :global() to unscoped
fn transform_global(selector: &str) -> String {
    // :global(.class) -> .class
    if let Some(start) = selector.find(":global(") {
        let before = &selector[..start];
        let after = &selector[start + 8..];

        if let Some(end) = after.find(')') {
            let inner = &after[..end];
            let rest = &after[end + 1..];

            let mut result = String::with_capacity(before.len() + inner.len() + rest.len());
            result.push_str(before);
            result.push_str(inner);
            result.push_str(rest);
            return result;
        }
    }

    selector.to_compact_string()
}

/// Extract CSS v-bind() expressions
pub fn extract_css_vars(css: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let mut search_from = 0;

    while let Some(pos) = css[search_from..].find("v-bind(") {
        let start = search_from + pos + 7;
        if let Some(end) = css[start..].find(')') {
            let expr = css[start..start + end].trim();
            // Remove quotes if present
            let expr = expr.trim_matches(|c| c == '"' || c == '\'');
            vars.push(expr.to_compact_string());
            search_from = start + end + 1;
        } else {
            break;
        }
    }

    vars
}

#[cfg(test)]
mod tests {
    use super::{
        apply_scoped_css, extract_css_vars, scope_selector, transform_deep, transform_global,
    };

    #[test]
    fn test_scope_simple_selector() {
        let result = scope_selector(".foo", "[data-v-123]");
        assert_eq!(result, ".foo[data-v-123]");
    }

    #[test]
    fn test_scope_descendant_selector() {
        let result = scope_selector(".foo .bar", "[data-v-123]");
        assert_eq!(result, ".foo .bar[data-v-123]");
    }

    #[test]
    fn test_scope_multiple_selectors() {
        let result = scope_selector(".foo, .bar", "[data-v-123]");
        assert_eq!(result, ".foo[data-v-123], .bar[data-v-123]");
    }

    #[test]
    fn test_transform_deep() {
        let result = transform_deep(":deep(.child)", "[data-v-123]");
        assert_eq!(result, "[data-v-123] .child");
    }

    #[test]
    fn test_transform_global() {
        let result = transform_global(":global(.foo)");
        assert_eq!(result, ".foo");
    }

    #[test]
    fn test_extract_css_vars() {
        let css = ".foo { color: v-bind(color); background: v-bind('bgColor'); }";
        let vars = extract_css_vars(css);
        assert_eq!(vars, vec!["color", "bgColor"]);
    }

    #[test]
    fn test_scope_media_query() {
        let css = "@media (max-width: 768px) { .foo { color: red; } }";
        let result = apply_scoped_css(css, "data-v-123");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_scope_media_query_with_comment() {
        let css = "/* Mobile responsive */\n@media (max-width: 768px) {\n  .glyph-playground {\n    grid-template-columns: 1fr;\n  }\n}";
        let result = apply_scoped_css(css, "data-v-123");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_scope_keyframes() {
        let css = "@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }";
        let result = apply_scoped_css(css, "data-v-123");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_scope_webkit_keyframes() {
        let css = "@-webkit-keyframes fade { 0% { opacity: 0; } 100% { opacity: 1; } }";
        let result = apply_scoped_css(css, "data-v-123");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_nested_css_media_passthrough() {
        // CSS nesting: @media (--mobile) inside a selector should pass through
        let css = "#pages-store {\n  display: grid;\n  row-gap: 1.5rem;\n  @media (--mobile) {\n    row-gap: 1rem;\n  }\n  h1 {\n    padding: 7.5rem 0;\n    @media (--mobile) {\n      padding: 2.5rem 0.75rem;\n    }\n  }\n}";
        let result = apply_scoped_css(css, "data-v-123");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_root_level_media_with_custom_query() {
        // Root-level @media with custom media query
        let css = ".foo { color: red; }\n@media (--mobile) { .foo { font-size: 12px; } }";
        let result = apply_scoped_css(css, "data-v-abc");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_apply_scoped_css_at_import() {
        let css = "@import \"~/assets/styles/custom-media-query.css\";\n\nfooter { width: 100%; }";
        let result = apply_scoped_css(css, "data-v-123");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_apply_scoped_css_at_import_with_nested_css() {
        let css = "@import \"custom.css\";\n\nfooter {\n  width: 100%;\n  @media (--mobile) {\n    padding: 1rem;\n  }\n}";
        let result = apply_scoped_css(css, "data-v-abc");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_scope_keyframes_inside_media() {
        let css = "@media (prefers-reduced-motion: no-preference) { @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } } .foo { color: red; } }";
        let result = apply_scoped_css(css, "data-v-123");
        insta::assert_snapshot!(result.as_str());
    }
}

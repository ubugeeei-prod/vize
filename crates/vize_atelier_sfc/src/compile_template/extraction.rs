//! Extraction of imports, hoisted consts, and render functions from compiled template code.

use vize_carton::{String, ToCompactString};

use super::string_tracking::{
    compact_render_body, count_braces_with_state, count_parens_with_state, StringTrackState,
};

/// Extract imports, hoisted consts, and render function from compiled template code
/// Returns (imports, hoisted, render_function) where render_function is the full function definition
#[cfg(test)]
pub(crate) fn extract_template_parts_full(template_code: &str) -> (String, String, String) {
    let mut imports = String::default();
    let mut hoisted = String::default();
    let mut render_fn = String::default();
    let mut in_render = false;
    let mut brace_depth = 0;
    let mut brace_state = StringTrackState::default();

    for line in template_code.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("import ") {
            imports.push_str(line);
            imports.push('\n');
        } else if trimmed.starts_with("const _hoisted_") {
            hoisted.push_str(line);
            hoisted.push('\n');
        } else if trimmed.starts_with("export function render(")
            || trimmed.starts_with("function render(")
        {
            in_render = true;
            brace_depth = 0;
            brace_state = StringTrackState::default();
            brace_depth += count_braces_with_state(line, &mut brace_state);
            render_fn.push_str(line);
            render_fn.push('\n');
        } else if in_render {
            brace_depth += count_braces_with_state(line, &mut brace_state);
            render_fn.push_str(line);
            render_fn.push('\n');

            if brace_depth == 0 {
                in_render = false;
            }
        }
    }

    (imports, hoisted, render_fn)
}

/// Extract imports, hoisted consts, preamble (component/directive resolution), and render body
/// from compiled template code.
/// Returns (imports, hoisted, preamble, render_body)
#[allow(dead_code)]
pub(crate) fn extract_template_parts(template_code: &str) -> (String, String, String, String) {
    let mut imports = String::default();
    let mut hoisted = String::default();
    let mut preamble = String::default(); // Component/directive resolution statements
    let mut render_body = String::default();
    let mut in_render = false;
    let mut in_return = false;
    let mut brace_depth = 0;
    let mut brace_state = StringTrackState::default();
    let mut paren_state = StringTrackState::default();
    let mut return_paren_depth = 0;

    // Collect all lines for look-ahead
    let lines: Vec<&str> = template_code.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("import ") {
            imports.push_str(line);
            imports.push('\n');
        } else if trimmed.starts_with("const _hoisted_") {
            // Hoisted template variables
            hoisted.push_str(line);
            hoisted.push('\n');
        } else if trimmed.starts_with("export function render(")
            || trimmed.starts_with("function render(")
        {
            in_render = true;
            brace_depth = 0;
            brace_state = StringTrackState::default();
            paren_state = StringTrackState::default();
            brace_depth += count_braces_with_state(line, &mut brace_state);
        } else if in_render {
            brace_depth += count_braces_with_state(line, &mut brace_state);

            // Extract the return statement inside the render function (may span multiple lines)
            if in_return {
                // Continue collecting return body
                render_body.push('\n');
                render_body.push_str(line);
                return_paren_depth += count_parens_with_state(line, &mut paren_state);

                // Check if return statement is complete:
                // - Parentheses must be balanced (return_paren_depth <= 0)
                // - Next non-empty line must NOT be a ternary continuation (? or :)
                if return_paren_depth <= 0 {
                    // Look ahead to check for ternary continuation
                    let next_continues_ternary = lines
                        .iter()
                        .skip(i + 1)
                        .map(|l| l.trim())
                        .find(|l| !l.is_empty())
                        .map(|l| l.starts_with('?') || l.starts_with(':'))
                        .unwrap_or(false);

                    if !next_continues_ternary {
                        in_return = false;
                        // Remove trailing semicolon if present
                        let trimmed_body = render_body.trim_end();
                        if let Some(stripped) = trimmed_body.strip_suffix(';') {
                            render_body = stripped.to_compact_string();
                        }
                    }
                }
            } else if let Some(stripped) = trimmed.strip_prefix("return ") {
                render_body = stripped.to_compact_string();
                // Count parentheses to handle multi-line return (string-aware)
                paren_state = StringTrackState::default();
                return_paren_depth = count_parens_with_state(stripped, &mut paren_state);
                if return_paren_depth > 0 {
                    in_return = true;
                } else {
                    // Check if next non-empty line is a ternary continuation
                    let next_continues_ternary = lines
                        .iter()
                        .skip(i + 1)
                        .map(|l| l.trim())
                        .find(|l| !l.is_empty())
                        .map(|l| l.starts_with('?') || l.starts_with(':'))
                        .unwrap_or(false);

                    if next_continues_ternary {
                        in_return = true;
                    } else {
                        // Single line return - remove trailing semicolon if present
                        if render_body.ends_with(';') {
                            render_body.pop();
                        }
                    }
                }
            } else if trimmed.starts_with("const _component_")
                || trimmed.starts_with("const _directive_")
            {
                // Component/directive resolution statements go in preamble
                preamble.push_str(trimmed);
                preamble.push('\n');
            }

            if brace_depth == 0 {
                in_render = false;
            }
        }
    }

    // Compact the render body to remove unnecessary line breaks inside function calls
    let compacted = compact_render_body(&render_body);

    (imports, hoisted, preamble, compacted)
}

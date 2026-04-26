//! Extraction of imports, hoisted consts, and render functions from compiled template code.

use vize_carton::{String, ToCompactString};

use super::string_tracking::{
    compact_render_body, count_braces_with_state, count_parens_with_state, StringTrackState,
};

fn is_vapor_template_declaration(line: &str) -> bool {
    line.starts_with("const t") && line.contains("_template(")
}

fn detect_render_export_name(trimmed: &str) -> Option<&'static str> {
    if trimmed.starts_with("export function render(") || trimmed.starts_with("function render(") {
        Some("render")
    } else if trimmed.starts_with("export function ssrRender(")
        || trimmed.starts_with("function ssrRender(")
    {
        Some("ssrRender")
    } else {
        None
    }
}

fn finalize_render_body(render_body: &mut String) {
    while render_body.ends_with([' ', '\t', '\n', '\r']) {
        render_body.pop();
    }

    if render_body.ends_with(';') {
        render_body.pop();
    }
}

/// Extract imports, hoisted consts, and render function from compiled template code
/// Returns (imports, hoisted, render_function, render_function_name)
/// where render_function is the full function definition.
pub(crate) fn extract_template_parts_full(
    template_code: &str,
) -> (String, String, String, &'static str) {
    let mut imports = String::default();
    let mut hoisted = String::default();
    let mut render_fn = String::default();
    let mut render_fn_name = "";
    let mut in_render = false;
    let mut brace_depth = 0;
    let mut brace_state = StringTrackState::default();

    for line in template_code.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("import ") {
            imports.push_str(line);
            imports.push('\n');
        } else if let Some(name) = detect_render_export_name(trimmed) {
            in_render = true;
            render_fn_name = name;
            brace_depth = 0;
            brace_state = StringTrackState::default();
            brace_depth += count_braces_with_state(line, &mut brace_state);
            render_fn.push_str(line);
            render_fn.push('\n');
        } else if trimmed.starts_with("const _hoisted_")
            || is_vapor_template_declaration(trimmed)
            || (!trimmed.is_empty() && !in_render)
        {
            hoisted.push_str(line);
            hoisted.push('\n');
        } else if in_render {
            brace_depth += count_braces_with_state(line, &mut brace_state);
            render_fn.push_str(line);
            render_fn.push('\n');

            if brace_depth == 0 {
                in_render = false;
            }
        }
    }

    (imports, hoisted, render_fn, render_fn_name)
}

/// Extract imports, hoisted consts, preamble (component/directive resolution), and render body
/// from compiled template code.
/// Returns (imports, hoisted, preamble, render_body, render_function_name)
#[allow(dead_code)]
pub(crate) fn extract_template_parts(
    template_code: &str,
) -> (String, String, String, String, &'static str) {
    let mut imports = String::default();
    let mut hoisted = String::default();
    let mut preamble = String::default(); // Component/directive resolution statements
    let mut render_body = String::default();
    let mut render_fn_name = "";
    let mut in_render = false;
    let mut in_block_render = false;
    let mut saw_block_render = false;
    let mut in_return = false;
    let mut brace_depth = 0;
    let mut brace_state = StringTrackState::default();
    let mut paren_state = StringTrackState::default();
    let mut return_paren_depth = 0;
    let mut pending_ternary_continuation = false;

    for line in template_code.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("import ") {
            imports.push_str(line);
            imports.push('\n');
        } else if trimmed.starts_with("const _hoisted_") || is_vapor_template_declaration(trimmed) {
            // Hoisted template variables
            hoisted.push_str(line);
            hoisted.push('\n');
        } else if let Some(name) = detect_render_export_name(trimmed) {
            in_render = true;
            render_fn_name = name;
            in_block_render = trimmed.starts_with("function render(") && trimmed.contains("$props");
            saw_block_render = saw_block_render || in_block_render;
            brace_depth = 0;
            brace_state = StringTrackState::default();
            paren_state = StringTrackState::default();
            brace_depth += count_braces_with_state(line, &mut brace_state);
        } else if in_render {
            let brace_delta = count_braces_with_state(line, &mut brace_state);
            let next_brace_depth = brace_depth + brace_delta;

            if in_block_render {
                if !(next_brace_depth == 0 && trimmed == "}") {
                    if !render_body.is_empty() {
                        render_body.push('\n');
                    }
                    render_body.push_str(line);
                }

                brace_depth = next_brace_depth;
                if brace_depth == 0 {
                    in_render = false;
                    in_block_render = false;
                }
                continue;
            }

            brace_depth = next_brace_depth;

            if pending_ternary_continuation && !trimmed.is_empty() {
                if trimmed.starts_with('?') || trimmed.starts_with(':') {
                    pending_ternary_continuation = false;
                } else {
                    pending_ternary_continuation = false;
                    in_return = false;
                    finalize_render_body(&mut render_body);
                }
            }

            // Extract the return statement inside the render function (may span multiple lines)
            if in_return {
                // Continue collecting return body
                render_body.push('\n');
                render_body.push_str(line);
                return_paren_depth += count_parens_with_state(line, &mut paren_state);

                if return_paren_depth <= 0 {
                    pending_ternary_continuation = true;
                }
            } else if let Some(stripped) = trimmed.strip_prefix("return ") {
                render_body = stripped.to_compact_string();
                // Count parentheses to handle multi-line return (string-aware)
                paren_state = StringTrackState::default();
                return_paren_depth = count_parens_with_state(stripped, &mut paren_state);
                if return_paren_depth > 0 {
                    in_return = true;
                } else {
                    in_return = true;
                    pending_ternary_continuation = true;
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

    if in_return {
        finalize_render_body(&mut render_body);
    }

    // Compact VDOM-style return expressions, but keep Vapor statement blocks intact.
    let compacted = if saw_block_render {
        render_body
    } else {
        compact_render_body(&render_body)
    };

    (imports, hoisted, preamble, compacted, render_fn_name)
}

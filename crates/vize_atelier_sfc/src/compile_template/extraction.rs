//! Extraction of imports, hoisted consts, and render functions from compiled template code.

use vize_carton::{String, ToCompactString};

use super::TemplateCodeSections;
use super::string_tracking::{
    StringTrackState, count_braces_with_state, count_delims_with_state, count_parens_with_state,
};

/// Slice the structural sections out of compiled template code using
/// emission-recorded byte offsets.
///
/// Produces the same `(imports, hoisted, preamble, render_body, name)` tuple
/// as [`extract_template_parts`], but without re-scanning the whole module
/// line by line: the codegen pipeline already knows where each section
/// starts and ends, so this is slicing plus a trim pass over the (tiny)
/// asset-resolution region.
pub(crate) fn slice_template_parts(
    template_code: &str,
    sections: &TemplateCodeSections,
) -> (String, String, String, String, &'static str) {
    let slice = |(start, end): (usize, usize)| template_code.get(start..end).unwrap_or_default();

    let imports = String::new(slice(sections.imports));
    let hoisted = String::new(slice(sections.hoisted));

    // Asset-resolution statements carry the render function's indentation;
    // the inline assembly expects them trimmed, one per line. The region also
    // ends with the blank separator line codegen emits before `return`.
    let assets_raw = slice(sections.assets);
    let mut preamble = String::with_capacity(assets_raw.len());
    for line in assets_raw.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            preamble.push_str(trimmed);
            preamble.push('\n');
        }
    }

    // Same trailing cleanup as `finalize_render_body`: drop trailing
    // whitespace, then at most one `;`.
    let mut body = slice(sections.return_expr);
    body = body.trim_end_matches([' ', '\t', '\n', '\r']);
    if let Some(stripped) = body.strip_suffix(';') {
        body = stripped;
    }
    let render_body = String::new(body);

    (imports, hoisted, preamble, render_body, "render")
}

fn is_vapor_template_declaration(line: &str) -> bool {
    line.starts_with("const t") && line.contains("_template(")
}

/// A hoisted const declaration may span multiple lines when its value is a multi-line
/// object/array/paren literal, e.g.
///
/// ```text
/// const _hoisted_1 = { style: {
///   position: 'absolute',
/// } }
/// ```
///
/// Returns the net delimiter depth opened by `line` (over all of `{} [] ()`), so the caller
/// can keep appending continuation lines to `hoisted` until the depth returns to zero. Without
/// this the continuation lines were dropped, truncating the declaration into invalid JS.
fn hoisted_line_open_depth(line: &str, state: &mut StringTrackState) -> i32 {
    count_delims_with_state(line, state)
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

/// Extract imports, hoisted consts, and render function from compiled template code.
///
/// This is a line scanner over the compiler output rather than a second JS parse
/// or regex pipeline. `StringTrackState` carries string/comment/template-literal
/// state across lines so brace depth stays correct while still keeping the
/// profiled build path allocation-light.
///
/// Returns (imports, hoisted, render_function, render_function_name) where
/// render_function is the full function definition.
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
    // Depth/state for a hoisted declaration whose value spans multiple lines.
    let mut hoisted_depth = 0;
    let mut hoisted_state = StringTrackState::default();

    for line in template_code.lines() {
        let trimmed = line.trim();

        // Continuation lines of a multi-line hoisted declaration: keep collecting until the
        // value's delimiters are balanced so the declaration is emitted intact.
        if hoisted_depth > 0 {
            hoisted_depth += hoisted_line_open_depth(line, &mut hoisted_state);
            hoisted.push_str(line);
            hoisted.push('\n');
            continue;
        }

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
            hoisted_state = StringTrackState::default();
            hoisted_depth = hoisted_line_open_depth(line, &mut hoisted_state);
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
    let mut in_return = false;
    let mut brace_depth = 0;
    let mut brace_state = StringTrackState::default();
    let mut paren_state = StringTrackState::default();
    let mut return_paren_depth = 0;
    let mut pending_ternary_continuation = false;
    // Depth/state for a hoisted declaration whose value spans multiple lines.
    let mut hoisted_depth = 0;
    let mut hoisted_state = StringTrackState::default();

    for line in template_code.lines() {
        let trimmed = line.trim();

        // Continuation lines of a multi-line hoisted declaration: keep collecting until the
        // value's delimiters are balanced so the declaration is emitted intact. Previously
        // these lines fell through and were dropped, truncating the const into invalid JS.
        if hoisted_depth > 0 {
            hoisted_depth += hoisted_line_open_depth(line, &mut hoisted_state);
            hoisted.push_str(line);
            hoisted.push('\n');
            continue;
        }

        if trimmed.starts_with("import ") {
            imports.push_str(line);
            imports.push('\n');
        } else if trimmed.starts_with("const _hoisted_") || is_vapor_template_declaration(trimmed) {
            // Hoisted template variables (value may span multiple lines).
            hoisted_state = StringTrackState::default();
            hoisted_depth = hoisted_line_open_depth(line, &mut hoisted_state);
            hoisted.push_str(line);
            hoisted.push('\n');
        } else if let Some(name) = detect_render_export_name(trimmed) {
            in_render = true;
            render_fn_name = name;
            in_block_render = trimmed.starts_with("function render(") && trimmed.contains("$props");
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

    (imports, hoisted, preamble, render_body, render_fn_name)
}

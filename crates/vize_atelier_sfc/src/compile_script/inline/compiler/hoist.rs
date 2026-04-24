use vize_carton::{String, ToCompactString};

use crate::script::ScriptCompileContext;

use super::super::helpers::extract_const_name;

/// Separate hoisted consts (literal consts that can be module-level) from setup code.
///
/// Returns (hoisted_lines, setup_body_lines).
pub(super) fn separate_hoisted_consts(
    transformed_setup: &str,
    ctx: &ScriptCompileContext,
) -> (Vec<String>, Vec<String>) {
    let mut hoisted_lines: Vec<String> = Vec::new();
    let mut setup_body_lines: Vec<String> = Vec::new();
    let mut in_multiline_value = false;

    for line in transformed_setup.lines() {
        let trimmed = line.trim();
        // Track multi-line template literals / strings - don't hoist individual lines
        if in_multiline_value {
            setup_body_lines.push(line.to_compact_string());
            // Count unescaped backticks to detect end of template literal
            let backticks = trimmed
                .chars()
                .fold((0usize, false), |(count, escaped), c| {
                    if escaped {
                        (count, false)
                    } else if c == '\\' {
                        (count, true)
                    } else if c == '`' {
                        (count + 1, false)
                    } else {
                        (count, false)
                    }
                })
                .0;
            if backticks % 2 == 1 {
                in_multiline_value = false;
            }
            continue;
        }
        // Check if this is a literal const that should be hoisted
        if trimmed.starts_with("const ") && !trimmed.starts_with("const {") {
            // Check for multi-line const where value is on the next line (e.g., `const x =\n  'value'`)
            if let Some(eq_pos) = trimmed.find('=') {
                let value_part = trimmed[eq_pos + 1..].trim();
                // If value part is empty, the value is on the next line - don't hoist
                if value_part.is_empty() {
                    setup_body_lines.push(line.to_compact_string());
                    continue;
                }
                // Check for multi-line template literal (unclosed backtick)
                let backticks = value_part
                    .chars()
                    .fold((0usize, false), |(count, escaped), c| {
                        if escaped {
                            (count, false)
                        } else if c == '\\' {
                            (count, true)
                        } else if c == '`' {
                            (count + 1, false)
                        } else {
                            (count, false)
                        }
                    })
                    .0;
                if backticks % 2 == 1 {
                    // Unclosed template literal - don't hoist, mark as multi-line
                    in_multiline_value = true;
                    setup_body_lines.push(line.to_compact_string());
                    continue;
                }
            }
            // Extract variable name and check if it's LiteralConst
            if let Some(name) = extract_const_name(trimmed) {
                if matches!(
                    ctx.bindings.bindings.get(name.as_str()),
                    Some(crate::types::BindingType::LiteralConst)
                ) {
                    hoisted_lines.push(line.to_compact_string());
                    continue;
                }
            }
        }
        setup_body_lines.push(line.to_compact_string());
    }

    (hoisted_lines, setup_body_lines)
}

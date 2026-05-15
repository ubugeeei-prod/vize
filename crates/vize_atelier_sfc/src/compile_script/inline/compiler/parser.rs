use vize_carton::{String, ToCompactString};

use super::super::super::{
    macros::{
        is_macro_call_line, is_multiline_macro_start, is_paren_macro_start,
        is_props_destructure_line,
    },
    statement_sections::extract_script_sections,
};
use super::super::helpers::strip_comments_for_counting;

/// Parse script content to extract imports, setup lines, and TypeScript declarations.
///
/// Returns a tuple of (user_imports, setup_lines, ts_declarations).
pub(super) fn parse_script_content(
    content: &str,
    is_ts: bool,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    if let Some(sections) = extract_script_sections(content, is_ts) {
        return sections;
    }

    let mut user_imports = Vec::new();
    let mut setup_lines = Vec::new();
    // Collect TypeScript interfaces/types to preserve at module level (before export default)
    let mut ts_declarations: Vec<String> = Vec::new();

    // Parse script content - extract imports and setup code
    let mut in_import = false;
    let mut import_buffer = String::default();
    let mut in_destructure = false;
    let mut destructure_buffer = String::default();
    let mut brace_depth: i32 = 0;
    let mut in_macro_call = false;
    let mut macro_angle_depth: i32 = 0;
    let mut in_paren_macro_call = false;
    let mut paren_macro_depth: i32 = 0;
    let mut waiting_for_macro_close = false;
    // Track remaining parentheses after destructure's function call: `const { x } = func(\n...\n)`
    let mut in_destructure_call = false;
    let mut destructure_call_paren_depth: i32 = 0;
    let mut destructure_call_keep_lines = false; // true for regular function calls (keep args in output)
                                                 // Track multiline object literals: const xxx = { ... }
    let mut in_object_literal = false;
    let mut object_literal_buffer = String::default();
    let mut object_literal_brace_depth: i32 = 0;
    // Track TypeScript-only declarations (interface, type) to skip them
    let mut in_ts_interface = false;
    let mut ts_interface_brace_depth: i32 = 0;
    let mut in_ts_type = false;
    let mut ts_type_depth: i32 = 0; // Track angle brackets and parens for complex types
    let mut ts_type_pending_end = false; // True when type may have ended on `}` but need to check next line
                                         // Track template literals (backtick strings) to skip content inside them
    let mut in_template_literal = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Handle multi-line macro calls
        if in_macro_call {
            // Count angle brackets but ignore => (arrow functions) and comments
            let cleaned = strip_comments_for_counting(trimmed);
            let line_no_arrow = cleaned.replace("=>", "");
            macro_angle_depth += line_no_arrow.matches('<').count() as i32;
            macro_angle_depth -= line_no_arrow.matches('>').count() as i32;
            let trimmed_no_semi_m = trimmed.trim_end_matches(';');
            if macro_angle_depth <= 0
                && (trimmed_no_semi_m.contains("()") || trimmed_no_semi_m.ends_with(')'))
            {
                in_macro_call = false;
            }
            continue;
        }

        // Handle remaining parentheses from destructure's function call
        // e.g., `const { x } = someFunc(\n  arg1,\n  arg2\n)`
        if in_destructure_call {
            let cleaned = strip_comments_for_counting(trimmed);
            destructure_call_paren_depth += cleaned.matches('(').count() as i32;
            destructure_call_paren_depth -= cleaned.matches(')').count() as i32;
            // For regular (non-macro) function calls, keep argument lines in setup output
            if destructure_call_keep_lines {
                setup_lines.push(line.to_compact_string());
            }
            if destructure_call_paren_depth <= 0 {
                in_destructure_call = false;
            }
            continue;
        }

        if in_paren_macro_call {
            let cleaned = strip_comments_for_counting(trimmed);
            paren_macro_depth += cleaned.matches('(').count() as i32;
            paren_macro_depth -= cleaned.matches(')').count() as i32;
            if paren_macro_depth <= 0 {
                in_paren_macro_call = false;
            }
            continue;
        }

        if waiting_for_macro_close {
            destructure_buffer.push_str(line);
            destructure_buffer.push('\n');
            // Track angle brackets for type args (ignore => arrow functions and comments)
            let cleaned = strip_comments_for_counting(trimmed);
            let line_no_arrow = cleaned.replace("=>", "");
            macro_angle_depth += line_no_arrow.matches('<').count() as i32;
            macro_angle_depth -= line_no_arrow.matches('>').count() as i32;
            let trimmed_no_semi_w = trimmed.trim_end_matches(';');
            if macro_angle_depth <= 0
                && (trimmed_no_semi_w.ends_with("()") || trimmed_no_semi_w.ends_with(')'))
            {
                waiting_for_macro_close = false;
                destructure_buffer.clear();
            }
            continue;
        }

        if in_destructure {
            destructure_buffer.push_str(line);
            destructure_buffer.push('\n');
            // Track both braces and angle brackets for type args (ignore => arrow functions and comments)
            let cleaned = strip_comments_for_counting(trimmed);
            let line_no_arrow = cleaned.replace("=>", "");
            brace_depth += cleaned.matches('{').count() as i32;
            brace_depth -= cleaned.matches('}').count() as i32;
            macro_angle_depth += line_no_arrow.matches('<').count() as i32;
            macro_angle_depth -= line_no_arrow.matches('>').count() as i32;
            // Only consider closed when BOTH braces and angle brackets are balanced
            // and we have the closing parentheses
            if brace_depth <= 0 && macro_angle_depth <= 0 {
                let is_props_macro = destructure_buffer.contains("defineProps")
                    || destructure_buffer.contains("withDefaults");
                let trimmed_no_semi = trimmed.trim_end_matches(';');
                if is_props_macro
                    && !trimmed_no_semi.ends_with("()")
                    && !trimmed_no_semi.ends_with(')')
                {
                    waiting_for_macro_close = true;
                    continue;
                }
                in_destructure = false;
                if !is_props_macro {
                    // Not a props destructure - add to setup lines
                    for buf_line in destructure_buffer.lines() {
                        setup_lines.push(buf_line.to_compact_string());
                    }
                }
                // Check if the destructure's RHS has an unclosed function call:
                // `} = someFunc(\n  arg1,\n)` -- paren opens on this line, closes later
                let paren_balance = destructure_buffer.matches('(').count() as i32
                    - destructure_buffer.matches(')').count() as i32;
                if paren_balance > 0 {
                    in_destructure_call = true;
                    destructure_call_paren_depth = paren_balance;
                    destructure_call_keep_lines = !is_props_macro;
                }
                destructure_buffer.clear();
            }
            continue;
        }

        // Detect macro call starts
        if is_paren_macro_start(trimmed)
            && !trimmed.starts_with("const {")
            && !trimmed.starts_with("let {")
        {
            in_paren_macro_call = true;
            paren_macro_depth =
                trimmed.matches('(').count() as i32 - trimmed.matches(')').count() as i32;
            continue;
        }

        if is_multiline_macro_start(trimmed)
            && !trimmed.starts_with("const {")
            && !trimmed.starts_with("let {")
        {
            in_macro_call = true;
            macro_angle_depth =
                trimmed.matches('<').count() as i32 - trimmed.matches('>').count() as i32;
            continue;
        }

        // Detect destructure start with type args: const { x } = defineProps<{...}>()
        // This pattern has both the destructure closing brace AND type arg opening angle bracket
        if (trimmed.starts_with("const {")
            || trimmed.starts_with("let {")
            || trimmed.starts_with("var {"))
            && (trimmed.contains("defineProps<") || trimmed.contains("withDefaults("))
        {
            // Check if it's complete on a single line (strip trailing semicolons)
            let trimmed_no_semi_d = trimmed.trim_end_matches(';');
            if !trimmed_no_semi_d.ends_with("()") && !trimmed_no_semi_d.ends_with(')') {
                // Multi-line: wait for completion
                in_destructure = true;
                destructure_buffer = line.to_compact_string() + "\n";
                brace_depth =
                    trimmed.matches('{').count() as i32 - trimmed.matches('}').count() as i32;
                macro_angle_depth =
                    trimmed.matches('<').count() as i32 - trimmed.matches('>').count() as i32;
                continue;
            } else {
                // Single line, complete - skip it
                continue;
            }
        }

        // Detect destructure where value starts on the next line:
        //   const { x, y } =
        //     defineProps<...>()
        // Braces are balanced on this line but the RHS is on the next line.
        if (trimmed.starts_with("const {")
            || trimmed.starts_with("let {")
            || trimmed.starts_with("var {"))
            && trimmed.contains('}')
            && trimmed.ends_with('=')
        {
            in_destructure = true;
            destructure_buffer = line.to_compact_string() + "\n";
            brace_depth = 0; // braces are balanced on this line
            macro_angle_depth = 0;
            continue;
        }

        // Detect destructure start (without type args)
        if (trimmed.starts_with("const {")
            || trimmed.starts_with("let {")
            || trimmed.starts_with("var {"))
            && !trimmed.contains('}')
        {
            in_destructure = true;
            destructure_buffer = line.to_compact_string() + "\n";
            brace_depth = trimmed.matches('{').count() as i32 - trimmed.matches('}').count() as i32;
            macro_angle_depth = 0;
            continue;
        }

        // Skip single-line props destructure
        if is_props_destructure_line(trimmed) {
            continue;
        }

        // Handle multiline object literals: const xxx = { ... }
        if in_object_literal {
            object_literal_buffer.push_str(line);
            object_literal_buffer.push('\n');
            object_literal_brace_depth += trimmed.matches('{').count() as i32;
            object_literal_brace_depth -= trimmed.matches('}').count() as i32;
            if object_literal_brace_depth <= 0 {
                // Object literal is complete, add to setup_lines
                for buf_line in object_literal_buffer.lines() {
                    setup_lines.push(buf_line.to_compact_string());
                }
                in_object_literal = false;
                object_literal_buffer.clear();
            }
            continue;
        }

        // Detect multiline object literal start: const xxx = { or const xxx: Type = {
        if (trimmed.starts_with("const ")
            || trimmed.starts_with("let ")
            || trimmed.starts_with("var "))
            && trimmed.contains('=')
            && trimmed.ends_with('{')
            && !trimmed.contains("defineProps")
            && !trimmed.contains("defineEmits")
            && !trimmed.contains("defineModel")
        {
            in_object_literal = true;
            object_literal_buffer = line.to_compact_string() + "\n";
            object_literal_brace_depth =
                trimmed.matches('{').count() as i32 - trimmed.matches('}').count() as i32;
            continue;
        }

        // Track template literals (backtick strings) - count unescaped backticks
        // We need to track this to avoid treating code inside template literals as real imports
        let backtick_count = line
            .chars()
            .fold((0, false), |(count, escaped), c| {
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

        // Track if we were in template literal before this line
        let was_in_template_literal = in_template_literal;

        // Toggle template literal state for each unescaped backtick
        if backtick_count % 2 == 1 {
            in_template_literal = !in_template_literal;
        }

        // Skip import/macro detection for content inside template literals
        // but still add the content to setup_lines
        if was_in_template_literal {
            // This line is inside (or closes) a template literal
            if !trimmed.is_empty() && !is_macro_call_line(trimmed) {
                setup_lines.push(line.to_compact_string());
            }
            continue;
        }

        // Handle imports (only when NOT inside template literal)
        if trimmed.starts_with("import ") {
            // Handle side-effect imports without semicolons (e.g., import '@/css/reset.scss')
            // These have no 'from' clause and are always single-line
            if !trimmed.contains(" from ") && (trimmed.contains('\'') || trimmed.contains('"')) {
                let mut imp = String::with_capacity(line.len() + 1);
                imp.push_str(line);
                imp.push('\n');
                user_imports.push(imp);
                continue;
            }
            in_import = true;
            import_buffer.clear();
        }

        if in_import {
            import_buffer.push_str(line);
            import_buffer.push('\n');
            if trimmed.ends_with(';') || (trimmed.contains(" from ") && !trimmed.ends_with(',')) {
                user_imports.push(import_buffer.clone());
                in_import = false;
            }
            continue;
        }

        // Handle TypeScript interface declarations (collect for TS output, skip for JS)
        if in_ts_interface {
            if is_ts {
                if let Some(last) = ts_declarations.last_mut() {
                    last.push('\n');
                    last.push_str(line);
                }
            }
            ts_interface_brace_depth += trimmed.matches('{').count() as i32;
            ts_interface_brace_depth -= trimmed.matches('}').count() as i32;
            if ts_interface_brace_depth <= 0 {
                in_ts_interface = false;
            }
            continue;
        }

        // Detect TypeScript interface start
        if trimmed.starts_with("interface ") || trimmed.starts_with("export interface ") {
            in_ts_interface = true;
            ts_interface_brace_depth =
                trimmed.matches('{').count() as i32 - trimmed.matches('}').count() as i32;
            if is_ts {
                ts_declarations.push(line.to_compact_string());
            }
            if ts_interface_brace_depth <= 0 {
                in_ts_interface = false;
            }
            continue;
        }

        // Detect TypeScript `declare` statements (e.g., `declare global { }`, `declare module '...' { }`)
        if trimmed.starts_with("declare ") {
            let has_brace = trimmed.contains('{');
            if has_brace {
                let depth =
                    trimmed.matches('{').count() as i32 - trimmed.matches('}').count() as i32;
                if depth > 0 {
                    // Multi-line declare block: reuse the interface brace tracking
                    in_ts_interface = true;
                    ts_interface_brace_depth = depth;
                }
                if is_ts {
                    ts_declarations.push(line.to_compact_string());
                }
            } else {
                // Single-line declare (e.g., `declare const x: number`)
                if is_ts {
                    ts_declarations.push(line.to_compact_string());
                }
            }
            continue;
        }

        // Handle TypeScript type declarations (collect for TS output, skip for JS)
        if in_ts_type {
            let is_type_continuation = trimmed.starts_with('|')
                || trimmed.starts_with('&')
                || trimmed.starts_with('?')
                || trimmed.starts_with(':');

            // Handle pending end from previous line's closing `}`
            if ts_type_pending_end {
                ts_type_pending_end = false;
                if is_type_continuation {
                    // Continue the type - next union/intersection variant
                    if is_ts {
                        if let Some(last) = ts_declarations.last_mut() {
                            last.push('\n');
                            last.push_str(line);
                        }
                    }
                    let cleaned = strip_comments_for_counting(trimmed);
                    let line_no_arrow = cleaned.replace("=>", "__");
                    ts_type_depth += cleaned.matches('{').count() as i32;
                    ts_type_depth -= cleaned.matches('}').count() as i32;
                    ts_type_depth += line_no_arrow.matches('<').count() as i32;
                    ts_type_depth -= line_no_arrow.matches('>').count() as i32;
                    ts_type_depth += cleaned.matches('(').count() as i32;
                    ts_type_depth -= cleaned.matches(')').count() as i32;
                    continue;
                } else {
                    // NOT a continuation - type truly ended on the previous line
                    in_ts_type = false;
                    // Fall through to normal line processing below
                }
            }

            if in_ts_type {
                if is_ts {
                    if let Some(last) = ts_declarations.last_mut() {
                        last.push('\n');
                        last.push_str(line);
                    }
                }
                // Track balanced brackets for complex types like: type X = { a: string } | { b: number }
                // Strip `=>` before counting angle brackets to avoid misinterpreting arrow functions
                let cleaned = strip_comments_for_counting(trimmed);
                let line_no_arrow = cleaned.replace("=>", "__");
                ts_type_depth += cleaned.matches('{').count() as i32;
                ts_type_depth -= cleaned.matches('}').count() as i32;
                ts_type_depth += line_no_arrow.matches('<').count() as i32;
                ts_type_depth -= line_no_arrow.matches('>').count() as i32;
                ts_type_depth += cleaned.matches('(').count() as i32;
                ts_type_depth -= cleaned.matches(')').count() as i32;
                // Type declaration ends when balanced and NOT a continuation line
                // A line that starts with | or & is a union/intersection continuation
                // Type declaration ends when:
                // - brackets/parens are balanced (depth <= 0)
                // - line is NOT a continuation (doesn't start with | or &)
                // - line ends with semicolon, OR ends without continuation chars
                if ts_type_depth <= 0
                    && (trimmed.ends_with(';')
                        || (!is_type_continuation
                            && !trimmed.ends_with('|')
                            && !trimmed.ends_with('&')
                            && !trimmed.ends_with(',')
                            && !trimmed.ends_with('{')))
                {
                    // If the line ends with `}` (without `;`), the next line might be a union continuation
                    if trimmed.ends_with('}') && !trimmed.ends_with("};") {
                        ts_type_pending_end = true;
                    } else {
                        in_ts_type = false;
                    }
                }
                continue;
            }
        }

        // Detect TypeScript type alias start
        // Guard: ensure the word after `type ` is a valid identifier start (letter, _, {),
        // not an operator like `===`. This avoids misdetecting `type === 'foo'` as a TS type.
        // `{` is also valid: `export type { Foo }` (re-export syntax).
        if (trimmed.starts_with("type ")
            && trimmed[5..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '{'))
            || (trimmed.starts_with("export type ")
                && trimmed[12..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '{'))
        {
            // Check if it's a single-line type
            let has_equals = trimmed.contains('=');
            if has_equals {
                // Strip `=>` before counting angle brackets (arrow functions are not type delimiters)
                let line_no_arrow = trimmed.replace("=>", "__");
                ts_type_depth = trimmed.matches('{').count() as i32
                    - trimmed.matches('}').count() as i32
                    + line_no_arrow.matches('<').count() as i32
                    - line_no_arrow.matches('>').count() as i32
                    + trimmed.matches('(').count() as i32
                    - trimmed.matches(')').count() as i32;
                // Check if complete on one line
                // A type is NOT complete if:
                // - brackets/parens aren't balanced (depth > 0)
                // - line ends with continuation characters (|, &, ,, {, =)
                if ts_type_depth <= 0 {
                    if trimmed.ends_with(';') {
                        // Definitely complete single-line type
                        if is_ts {
                            ts_declarations.push(line.to_compact_string());
                        }
                        continue;
                    }
                    if !trimmed.ends_with('|')
                        && !trimmed.ends_with('&')
                        && !trimmed.ends_with(',')
                        && !trimmed.ends_with('{')
                        && !trimmed.ends_with('=')
                    {
                        // Possibly complete, but next line may be conditional type continuation (? / :)
                        if is_ts {
                            ts_declarations.push(line.to_compact_string());
                        }
                        in_ts_type = true;
                        ts_type_pending_end = true;
                        continue;
                    }
                }
                if is_ts {
                    ts_declarations.push(line.to_compact_string());
                }
                in_ts_type = true;
            } else {
                // type without equals (e.g., `type X` on its own line) - rare but handle
                if is_ts {
                    ts_declarations.push(line.to_compact_string());
                }
            }
            continue;
        }

        if !trimmed.is_empty() && !is_macro_call_line(trimmed) {
            // All user code goes to setup_lines
            // Hoisting user-defined consts is problematic without proper AST-based scope tracking
            // Template-generated _hoisted_X consts are handled separately by template.hoisted
            setup_lines.push(line.to_compact_string());
        }
    }

    (user_imports, setup_lines, ts_declarations)
}

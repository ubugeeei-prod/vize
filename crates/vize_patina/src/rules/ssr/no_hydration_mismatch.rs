//! Rule: no-hydration-mismatch
//!
//! Warns when non-deterministic values are used in templates that could cause
//! hydration mismatches in SSR.
//!
//! ## Why is this bad?
//! In SSR, the server renders HTML first, then the client hydrates it.
//! If the values differ between server and client, Vue will show a warning
//! and potentially re-render the component, negating SSR benefits.
//!
//! Common causes of hydration mismatch:
//! - Random values (Math.random(), crypto.randomUUID())
//! - Current time (Date.now(), new Date())
//! - Environment-specific values
//!
//! ## How to fix?
//! - Use `useId()` instead of random IDs
//! - Move non-deterministic logic to `onMounted` or use `<ClientOnly>`
//! - Use `useDateFormat` with a fixed date on server
//!
//! ## Example
//!
//! Bad:
//! ```vue
//! <template>
//!   <div :id="`item-${Math.random()}`">
//!     Current time: {{ Date.now() }}
//!   </div>
//! </template>
//! ```
//!
//! Good:
//! ```vue
//! <script setup>
//! import { useId } from 'vue';
//! const id = useId();
//! </script>
//!
//! <template>
//!   <div :id="`item-${id}`">
//!     <!-- Time shown only on client -->
//!     <ClientOnly>
//!       Current time: {{ Date.now() }}
//!     </ClientOnly>
//!   </div>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{ElementNode, ExpressionNode, InterpolationNode};

/// Non-deterministic function/value patterns that cause hydration mismatch
const HYDRATION_MISMATCH_PATTERNS: &[(&str, &str)] = &[
    // Random values
    (
        "Math.random",
        "Random values differ between server and client. Use `useId()` for unique IDs",
    ),
    (
        "crypto.randomUUID",
        "Random UUIDs differ between server and client. Use `useId()` for unique IDs",
    ),
    (
        "crypto.getRandomValues",
        "Random values differ between server and client",
    ),
    (
        "Math.floor(Math.random",
        "Random values differ between server and client. Use `useId()` for unique IDs",
    ),
    (
        "uuid()",
        "Random UUIDs differ between server and client. Use `useId()` for unique IDs",
    ),
    (
        "nanoid()",
        "Random IDs differ between server and client. Use `useId()` for unique IDs",
    ),
    // Date/Time
    (
        "Date.now",
        "Current time differs between server and client. Consider using `<ClientOnly>` or a fixed timestamp",
    ),
    (
        "new Date()",
        "Current time differs between server and client. Consider using `<ClientOnly>` or a fixed timestamp",
    ),
    (
        ".getTime()",
        "Time values may differ between server and client",
    ),
    (
        ".toLocaleString()",
        "Locale formatting may differ between server and client environments",
    ),
    (
        ".toLocaleDateString()",
        "Locale formatting may differ between server and client environments",
    ),
    (
        ".toLocaleTimeString()",
        "Locale formatting may differ between server and client environments",
    ),
    // Performance timing
    (
        "performance.now",
        "Performance timing differs between server and client",
    ),
    // Environment-specific
    (
        "process.env",
        "Environment variables may differ between server and client. Ensure they are consistent or use runtime config",
    ),
    (
        "import.meta.env",
        "Environment variables may differ between server and client. Ensure they are consistent or use runtime config",
    ),
];

static META: RuleMeta = RuleMeta {
    name: "ssr/no-hydration-mismatch",
    description: "Disallow non-deterministic values that cause hydration mismatch",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

pub struct NoHydrationMismatch;

impl NoHydrationMismatch {
    /// Check if expression contains any mismatch-prone patterns
    fn check_expression(content: &str) -> Option<(&'static str, &'static str)> {
        scan_expression_code(content)
    }
}

fn pattern_match(pattern: &'static str) -> Option<(&'static str, &'static str)> {
    HYDRATION_MISMATCH_PATTERNS
        .iter()
        .find(|(candidate, _)| *candidate == pattern)
        .copied()
}

fn scan_expression_code(content: &str) -> Option<(&'static str, &'static str)> {
    let bytes = content.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'\'' | b'"' => index = skip_quoted_string(bytes, index),
            b'`' => {
                let (next, found) = scan_template_literal(content, index);
                if found.is_some() {
                    return found;
                }
                index = next;
            }
            _ => {
                if let Some(found) = match_pattern_at(content, index) {
                    return Some(found);
                }
                index += 1;
            }
        }
    }

    None
}

fn match_pattern_at(content: &str, index: usize) -> Option<(&'static str, &'static str)> {
    // Match on bytes: `index` advances byte-wise, so it may sit inside a
    // multibyte character where `content[index..]` would panic. All patterns
    // are ASCII, so byte comparison is equivalent.
    let bytes = content.as_bytes();

    for pattern in [
        "Math.random",
        "crypto.randomUUID",
        "crypto.getRandomValues",
        "Date.now",
        "performance.now",
        "process.env",
        "import.meta.env",
    ] {
        if bytes[index..].starts_with(pattern.as_bytes())
            && has_member_left_boundary(bytes, index)
            && has_member_right_boundary(bytes, index + pattern.len())
        {
            return pattern_match(pattern);
        }
    }

    for pattern in ["uuid()", "nanoid()", "new Date()"] {
        if bytes[index..].starts_with(pattern.as_bytes())
            && has_identifier_left_boundary(bytes, index)
            && has_identifier_right_boundary(bytes, index + pattern.len())
        {
            return pattern_match(pattern);
        }
    }

    for pattern in [
        ".getTime()",
        ".toLocaleString()",
        ".toLocaleDateString()",
        ".toLocaleTimeString()",
    ] {
        if bytes[index..].starts_with(pattern.as_bytes())
            && has_identifier_right_boundary(bytes, index + pattern.len())
        {
            return pattern_match(pattern);
        }
    }

    None
}

fn skip_quoted_string(bytes: &[u8], start: usize) -> usize {
    let quote = bytes[start];
    let mut escaped = false;
    let mut index = start + 1;

    while index < bytes.len() {
        let byte = bytes[index];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == quote {
            return index + 1;
        }
        index += 1;
    }

    bytes.len()
}

fn scan_template_literal(
    content: &str,
    start: usize,
) -> (usize, Option<(&'static str, &'static str)>) {
    let bytes = content.as_bytes();
    let mut escaped = false;
    let mut index = start + 1;

    while index < bytes.len() {
        let byte = bytes[index];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'`' {
            return (index + 1, None);
        } else if byte == b'$' && bytes.get(index + 1) == Some(&b'{') {
            let expression_start = index + 2;
            let expression_end = find_template_expression_end(bytes, expression_start);
            if let Some(found) = scan_expression_code(&content[expression_start..expression_end]) {
                return (expression_end.saturating_add(1), Some(found));
            }
            index = expression_end;
        }
        index += 1;
    }

    (bytes.len(), None)
}

fn find_template_expression_end(bytes: &[u8], start: usize) -> usize {
    let mut depth = 1;
    let mut index = start;

    while index < bytes.len() {
        match bytes[index] {
            b'\'' | b'"' => index = skip_quoted_string(bytes, index),
            b'`' => {
                let (next, _) = scan_template_literal_bytes(bytes, index);
                index = next;
            }
            b'{' => {
                depth += 1;
                index += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return index;
                }
                index += 1;
            }
            _ => index += 1,
        }
    }

    bytes.len()
}

fn scan_template_literal_bytes(bytes: &[u8], start: usize) -> (usize, ()) {
    let mut escaped = false;
    let mut index = start + 1;

    while index < bytes.len() {
        let byte = bytes[index];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'`' {
            return (index + 1, ());
        }
        index += 1;
    }

    (bytes.len(), ())
}

fn has_member_left_boundary(bytes: &[u8], index: usize) -> bool {
    index == 0 || (!is_identifier_byte(bytes[index - 1]) && bytes[index - 1] != b'.')
}

fn has_member_right_boundary(bytes: &[u8], index: usize) -> bool {
    bytes
        .get(index)
        .is_none_or(|byte| !is_identifier_byte(*byte))
}

fn has_identifier_left_boundary(bytes: &[u8], index: usize) -> bool {
    index == 0 || !is_identifier_byte(bytes[index - 1])
}

fn has_identifier_right_boundary(bytes: &[u8], index: usize) -> bool {
    bytes
        .get(index)
        .is_none_or(|byte| !is_identifier_byte(*byte))
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
}

impl Rule for NoHydrationMismatch {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
        // Only run if SSR mode is enabled
        if !ctx.is_ssr_enabled() {
            return;
        }

        let content = match &interpolation.content {
            ExpressionNode::Simple(s) => s.content.as_str(),
            ExpressionNode::Compound(_) => return, // Skip compound expressions
        };

        if let Some((pattern, _help)) = Self::check_expression(content) {
            ctx.warn_with_help(
                ctx.t_fmt("ssr/no-hydration-mismatch.message", &[("pattern", pattern)]),
                &interpolation.loc,
                ctx.t_fmt("ssr/no-hydration-mismatch.help", &[("pattern", pattern)]),
            );
        }
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &vize_relief::ast::DirectiveNode<'a>,
    ) {
        // Only run if SSR mode is enabled
        if !ctx.is_ssr_enabled() {
            return;
        }

        // Check directive expressions
        if let Some(exp) = &directive.exp {
            let content = match exp {
                ExpressionNode::Simple(s) => s.content.as_str(),
                ExpressionNode::Compound(_) => return, // Skip compound expressions
            };

            if let Some((pattern, _help)) = Self::check_expression(content) {
                ctx.warn_with_help(
                    ctx.t_fmt(
                        "ssr/no-hydration-mismatch.message-attr",
                        &[("pattern", pattern)],
                    ),
                    &directive.loc,
                    ctx.t_fmt("ssr/no-hydration-mismatch.help", &[("pattern", pattern)]),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoHydrationMismatch;

    #[test]
    fn test_detects_math_random() {
        let content = "items.map(() => Math.random())";
        assert!(NoHydrationMismatch::check_expression(content).is_some());
    }

    #[test]
    fn test_detects_date_now() {
        let content = "Date.now()";
        assert!(NoHydrationMismatch::check_expression(content).is_some());
    }

    #[test]
    fn test_detects_new_date() {
        let content = "new Date()";
        assert!(NoHydrationMismatch::check_expression(content).is_some());
    }

    #[test]
    fn test_detects_crypto_random() {
        let content = "crypto.randomUUID()";
        assert!(NoHydrationMismatch::check_expression(content).is_some());
    }

    #[test]
    fn test_allows_safe_code() {
        let content = "items.map(item => item.name)";
        assert!(NoHydrationMismatch::check_expression(content).is_none());
    }

    #[test]
    fn test_multibyte_content_does_not_panic() {
        // Regression: byte-wise scanning used to slice mid-character and panic
        // on non-ASCII expressions (e.g. Japanese comments in misskey).
        let content =
            "[\n\t// 行が選択されているときは範囲選択色の適用を行側に任せる\n\tcell.row,\n]";
        assert!(NoHydrationMismatch::check_expression(content).is_none());
    }

    #[test]
    fn test_detects_pattern_after_multibyte_text() {
        let content = "// 乱数を使う\nMath.random()";
        assert!(NoHydrationMismatch::check_expression(content).is_some());
    }

    #[test]
    fn test_ignores_string_literal_false_positive() {
        let content = r#"'Date.now and Math.random are just text'"#;
        assert!(NoHydrationMismatch::check_expression(content).is_none());
    }

    #[test]
    fn test_ignores_double_quoted_raw_interpolation_text() {
        let content = r#""see the Date.now() docs for details""#;
        assert!(NoHydrationMismatch::check_expression(content).is_none());
    }

    #[test]
    fn test_ignores_template_literal_raw_text_false_positive() {
        let content = r#"`Date.now and import.meta.env are just text`"#;
        assert!(NoHydrationMismatch::check_expression(content).is_none());
    }

    #[test]
    fn test_detects_template_literal_embedded_expression() {
        let content = r#"`item-${Date.now()}`"#;
        assert_eq!(
            NoHydrationMismatch::check_expression(content).map(|(pattern, _)| pattern),
            Some("Date.now")
        );
    }

    #[test]
    fn test_ignores_unrelated_member_chain() {
        let content = "globals.Date.now";
        assert!(NoHydrationMismatch::check_expression(content).is_none());
    }

    #[test]
    fn test_ignores_process_env_substring_in_member_chain() {
        let content = "config.process.envName";
        assert!(NoHydrationMismatch::check_expression(content).is_none());
    }

    #[test]
    fn test_ignores_method_call_substring_in_longer_member_name() {
        let content = "clock.getTime()zone";
        assert!(NoHydrationMismatch::check_expression(content).is_none());
    }

    #[test]
    fn test_detects_member_method_call() {
        let content = "createdAt.getTime()";
        assert_eq!(
            NoHydrationMismatch::check_expression(content).map(|(pattern, _)| pattern),
            Some(".getTime()")
        );
    }

    #[test]
    fn test_detects_import_meta_env() {
        let content = "import.meta.env.MODE";
        assert_eq!(
            NoHydrationMismatch::check_expression(content).map(|(pattern, _)| pattern),
            Some("import.meta.env")
        );
    }
}

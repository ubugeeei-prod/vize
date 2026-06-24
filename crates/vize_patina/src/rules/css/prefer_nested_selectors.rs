//! css/prefer-nested-selectors
//!
//! Recommend using CSS nesting for descendant selectors.
//!
//! CSS nesting allows writing more maintainable and readable styles
//! by nesting child selectors inside parent selectors.
//!
//! ## Examples
//!
//! Before:
//! ```css
//! .parent .child { color: red; }
//! ```
//!
//! After:
//! ```css
//! .parent {
//!   .child { color: red; }
//! }
//! ```

use lightningcss::stylesheet::StyleSheet;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{CssLintResult, CssRule, CssRuleMeta};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/prefer-nested-selectors",
    description: "Recommend using CSS nesting for descendant selectors",
    default_severity: Severity::Warning,
};

/// Prefer nested selectors rule
pub struct PreferNestedSelectors;

impl CssRule for PreferNestedSelectors {
    fn meta(&self) -> &'static CssRuleMeta {
        &META
    }

    fn check<'i>(
        &self,
        source: &'i str,
        _stylesheet: &StyleSheet<'i, 'i>,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        // Use pattern matching to find descendant selectors
        // Pattern: ".class .child" or "element child" with space separator
        let bytes = source.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            // Skip CSS at-rules entirely. At-rules like `@import` and
            // `@keyframes` are not selectors, and their preludes/bodies can
            // contain spaces or identifiers that would otherwise be
            // misinterpreted as descendant selectors.
            if let Some(next) = skip_at_rule(bytes, i) {
                i = next;
                continue;
            }

            // Find a selector start (., #, or letter for element)
            if let Some(selector_start) = find_selector_start(bytes, i) {
                // Find the selector end (before {)
                if let Some(brace_pos) = find_next_brace(bytes, selector_start) {
                    let selector = &source[selector_start..brace_pos];
                    let trimmed = selector.trim();

                    // Skip selectors that already use CSS nesting syntax.
                    // The presence of `&` anywhere in the selector list means
                    // the rule is written inside a nesting context; warning
                    // about "could be nested" is a false positive and would
                    // conflict with the formatter's nested output.
                    if !is_already_nested(trimmed)
                        && is_descendant_selector(trimmed)
                        && let Some((_parent, _child)) = split_descendant_selector(trimmed)
                    {
                        let start = (offset + selector_start) as u32;
                        let end = (offset + brace_pos) as u32;

                        result.add_diagnostic(
                            LintDiagnostic::warn(
                                META.name,
                                "Consider using CSS nesting for descendant selectors",
                                start,
                                end,
                            )
                            .with_help(
                                "Use CSS nesting syntax to nest child selectors inside parent selectors",
                            ),
                        );
                    }
                    i = brace_pos + 1;
                } else {
                    i += 1;
                }
            } else {
                break;
            }
        }
    }
}

/// At-rules whose body does not contain ordinary style rules and so
/// should be skipped entirely (prelude + block).
const NON_NESTED_BLOCK_AT_RULES: &[&str] = &[
    "keyframes",
    "-webkit-keyframes",
    "-moz-keyframes",
    "font-face",
    "page",
    "counter-style",
    "property",
    "font-feature-values",
    "color-profile",
    "viewport",
];

/// At-rules that end with `;` rather than a block.
const STATEMENT_AT_RULES: &[&str] = &["import", "charset", "namespace", "use", "forward"];

/// If `bytes[start..]` begins (after leading whitespace) with a CSS at-rule,
/// return the index just past the end of that at-rule. Otherwise, return
/// `None`.
///
/// For statement at-rules (e.g. `@import "x.css";`) the entire statement up
/// to and including the terminating `;` is consumed.
///
/// For block at-rules whose body should not be inspected (e.g.
/// `@keyframes`, `@font-face`), the entire `@kw ... { ... }` is consumed.
///
/// For other block at-rules (`@media`, `@supports`, `@container`, `@scope`,
/// `@layer { ... }`, `@document`) the prelude up to and including the
/// opening `{` is consumed so the scanner descends into the body and keeps
/// flagging descendant selectors inside conditional groups.
#[inline]
fn skip_at_rule(bytes: &[u8], start: usize) -> Option<usize> {
    // Skip leading whitespace and closing braces from prior rules.
    let mut p = start;
    while p < bytes.len() {
        match bytes[p] {
            b' ' | b'\t' | b'\n' | b'\r' | b'}' => p += 1,
            _ => break,
        }
    }
    if p >= bytes.len() || bytes[p] != b'@' {
        return None;
    }

    // Read the at-rule keyword.
    let kw_start = p + 1;
    let mut kw_end = kw_start;
    while kw_end < bytes.len() {
        let b = bytes[kw_end];
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' {
            kw_end += 1;
        } else {
            break;
        }
    }
    if kw_end == kw_start {
        return None;
    }
    let kw = &bytes[kw_start..kw_end];

    // Helper: case-insensitive match against an ASCII keyword.
    let eq_ignore_ascii_case = |needle: &str| -> bool {
        if kw.len() != needle.len() {
            return false;
        }
        kw.iter()
            .zip(needle.bytes())
            .all(|(a, b)| a.eq_ignore_ascii_case(&b))
    };

    let is_statement = STATEMENT_AT_RULES.iter().any(|s| eq_ignore_ascii_case(s));
    let is_non_nested_block = NON_NESTED_BLOCK_AT_RULES
        .iter()
        .any(|s| eq_ignore_ascii_case(s));

    if is_statement {
        // Consume up to and including the `;` (or end of input).
        let mut q = kw_end;
        while q < bytes.len() && bytes[q] != b';' {
            q += 1;
        }
        return Some((q + 1).min(bytes.len()));
    }

    // Find the next `;` or `{` to determine whether this at-rule has a body.
    let mut q = kw_end;
    while q < bytes.len() && bytes[q] != b'{' && bytes[q] != b';' {
        q += 1;
    }
    if q >= bytes.len() {
        return Some(bytes.len());
    }
    if bytes[q] == b';' {
        // e.g. `@layer foo, bar;` — statement form, no body.
        return Some(q + 1);
    }

    // bytes[q] == b'{'
    if is_non_nested_block {
        // Skip the entire block by matching braces.
        return Some(skip_balanced_block(bytes, q));
    }

    // Conditional group rule (`@media`, `@supports`, ...). Step past the
    // opening brace so the main loop continues scanning inside.
    Some(q + 1)
}

/// Given `open_pos` pointing at `{`, return the index just past the
/// matching `}` (or end of input if unbalanced).
#[inline]
fn skip_balanced_block(bytes: &[u8], open_pos: usize) -> usize {
    let mut depth: i32 = 0;
    let mut i = open_pos;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

/// Return `true` if the selector list already uses CSS nesting syntax
/// (i.e. contains the `&` parent selector outside of brackets, parens, or
/// strings). Such selectors must be inside a nesting context already, so
/// suggesting nesting is a false positive.
#[inline]
fn is_already_nested(selector: &str) -> bool {
    let bytes = selector.as_bytes();
    let mut bracket_depth: usize = 0;
    let mut paren_depth: usize = 0;
    let mut in_quote = false;
    let mut quote_char: u8 = 0;

    for &b in bytes {
        if !in_quote && (b == b'"' || b == b'\'') {
            in_quote = true;
            quote_char = b;
            continue;
        }
        if in_quote {
            if b == quote_char {
                in_quote = false;
            }
            continue;
        }
        match b {
            b'[' => bracket_depth += 1,
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b'(' => paren_depth += 1,
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'&' if bracket_depth == 0 && paren_depth == 0 => return true,
            _ => {}
        }
    }
    false
}

/// Find the start of a selector
#[inline]
fn find_selector_start(bytes: &[u8], start: usize) -> Option<usize> {
    for (offset, &byte) in bytes[start..].iter().enumerate() {
        match byte {
            b'.' | b'#' => return Some(start + offset),
            b'a'..=b'z' | b'A'..=b'Z' => {
                // Check it's not inside a comment or string
                return Some(start + offset);
            }
            b' ' | b'\n' | b'\r' | b'\t' | b'}' => continue,
            _ => continue,
        }
    }
    None
}

/// Find the next opening brace
#[inline]
fn find_next_brace(bytes: &[u8], start: usize) -> Option<usize> {
    for (offset, &byte) in bytes[start..].iter().enumerate() {
        if byte == b'{' {
            return Some(start + offset);
        }
        // Stop at @ rules or }
        if byte == b'@' || byte == b'}' {
            return None;
        }
    }
    None
}

/// Find the closing brace for a rule (reserved for future use)
#[inline]
#[allow(dead_code)]
fn find_closing_brace(bytes: &[u8], open_pos: usize) -> usize {
    let mut depth = 1;
    for (offset, &byte) in bytes[open_pos + 1..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return open_pos + 1 + offset;
                }
            }
            _ => {}
        }
    }
    bytes.len()
}

/// Check if a selector is a descendant selector
#[inline]
fn is_descendant_selector(selector: &str) -> bool {
    let bytes = selector.as_bytes();
    let mut bracket_depth: usize = 0;
    let mut paren_depth: usize = 0;
    let mut in_quote = false;
    let mut quote_char: u8 = 0;

    for &b in bytes {
        // Handle quotes
        if !in_quote && (b == b'"' || b == b'\'') {
            in_quote = true;
            quote_char = b;
            continue;
        }
        if in_quote && b == quote_char {
            in_quote = false;
            continue;
        }
        if in_quote {
            continue;
        }

        match b {
            b'[' => bracket_depth += 1,
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b'(' => paren_depth += 1,
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b' ' if bracket_depth == 0 && paren_depth == 0 => {
                // Found a space outside brackets/parens - this is a descendant selector
                return true;
            }
            b'>' | b'+' | b'~' if bracket_depth == 0 && paren_depth == 0 => {
                // Also handle child, adjacent, and sibling combinators
                return true;
            }
            _ => {}
        }
    }
    false
}

/// Split a descendant selector into parent and child parts
#[inline]
fn split_descendant_selector(selector: &str) -> Option<(&str, &str)> {
    let bytes = selector.as_bytes();
    let mut bracket_depth: usize = 0;
    let mut paren_depth: usize = 0;

    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'[' => bracket_depth += 1,
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b'(' => paren_depth += 1,
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b' ' | b'>' | b'+' | b'~' if bracket_depth == 0 && paren_depth == 0 => {
                let parent = selector[..i].trim();
                let child = selector[i..]
                    .trim()
                    .trim_start_matches([' ', '>', '+', '~'])
                    .trim();
                if !parent.is_empty() && !child.is_empty() {
                    return Some((parent, child));
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::PreferNestedSelectors;
    use crate::rules::css::CssLinter;

    fn create_linter() -> CssLinter {
        let mut linter = CssLinter::new();
        linter.add_rule(Box::new(PreferNestedSelectors));
        linter
    }

    #[test]
    fn test_simple_selector() {
        let linter = create_linter();
        let result = linter.lint(".button { color: red; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_descendant_selector() {
        let linter = create_linter();
        let result = linter.lint(".parent .child { color: red; }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_child_selector() {
        let linter = create_linter();
        let result = linter.lint(".parent > .child { color: red; }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_element_descendant() {
        let linter = create_linter();
        let result = linter.lint("div span { color: red; }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_complex_selector() {
        let linter = create_linter();
        let result = linter.lint(".parent .child { color: red; }", 0);
        assert_eq!(result.warning_count, 1);
        // Fix is not yet implemented for this rule
        // assert!(result.diagnostics[0].fix.is_some());
    }

    #[test]
    fn test_attribute_selector() {
        let linter = create_linter();
        // Space inside attribute selector should not trigger
        let result = linter.lint("[data-foo=\"bar baz\"] { color: red; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_nested_selector_list_does_not_warn() {
        // CSS nesting syntax: the `&` parent selector means the rule is
        // already nested. Suggesting nesting would conflict with the
        // formatter's output. See https://github.com/ubugeeei-prod/vize/issues/2246.
        let linter = create_linter();
        let result = linter.lint(".rendered-content { & h1, & h2 { font-weight: 600; } }", 0);
        assert_eq!(
            result.warning_count, 0,
            "& h1, & h2 should not warn; diagnostics: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_nested_selector_single_does_not_warn() {
        let linter = create_linter();
        let result = linter.lint(".parent { & .child { color: red; } }", 0);
        assert_eq!(
            result.warning_count, 0,
            "& .child should not warn; diagnostics: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_keyframes_does_not_warn() {
        let linter = create_linter();
        let source = "@keyframes loading { 0% { opacity: 0; } 100% { opacity: 1; } }";
        let result = linter.lint(source, 0);
        assert_eq!(
            result.warning_count, 0,
            "@keyframes prelude/body should not warn; diagnostics: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_import_does_not_warn() {
        let linter = create_linter();
        let source = "@import \"x.css\";\n.foo { color: red; }";
        let result = linter.lint(source, 0);
        assert_eq!(
            result.warning_count, 0,
            "@import should not warn; diagnostics: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_font_face_does_not_warn() {
        let linter = create_linter();
        let source = "@font-face { font-family: \"X\"; src: url(x.woff2); }";
        let result = linter.lint(source, 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_media_query_still_warns_on_descendants() {
        // Conditional group rules should still descend into their bodies so
        // genuine descendant selectors are still caught.
        let linter = create_linter();
        let result = linter.lint(
            "@media (min-width: 600px) { .parent .child { color: red; } }",
            0,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_descendant_after_keyframes_still_warns() {
        let linter = create_linter();
        let source = "@keyframes loading { 0% { opacity: 0; } } .parent .child { color: red; }";
        let result = linter.lint(source, 0);
        assert_eq!(result.warning_count, 1);
    }
}

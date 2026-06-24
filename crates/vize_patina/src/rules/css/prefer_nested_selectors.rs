//! css/prefer-nested-selectors
//!
//! Recommend using CSS nesting for descendant selectors.

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
        let bytes = source.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if let Some(next) = skip_at_rule(bytes, i) {
                i = next;
                continue;
            }
            let Some(selector_start) = find_selector_start(bytes, i) else {
                break;
            };
            let Some(brace_pos) = find_next_brace(bytes, selector_start) else {
                i += 1;
                continue;
            };
            let trimmed = source[selector_start..brace_pos].trim();
            if !is_already_nested(trimmed) && split_descendant_selector(trimmed).is_some() {
                result.add_diagnostic(
                    LintDiagnostic::warn(
                        META.name,
                        "Consider using CSS nesting for descendant selectors",
                        (offset + selector_start) as u32,
                        (offset + brace_pos) as u32,
                    )
                    .with_help(
                        "Use CSS nesting syntax to nest child selectors inside parent selectors",
                    ),
                );
            }
            i = brace_pos + 1;
        }
    }
}

/// At-rules whose body does not contain ordinary style rules; the entire block is skipped.
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

fn skip_at_rule(bytes: &[u8], start: usize) -> Option<usize> {
    let mut p = start;
    while p < bytes.len() && matches!(bytes[p], b' ' | b'\t' | b'\n' | b'\r' | b'}') {
        p += 1;
    }
    if p >= bytes.len() || bytes[p] != b'@' {
        return None;
    }
    let kw_start = p + 1;
    let mut kw_end = kw_start;
    while kw_end < bytes.len()
        && (bytes[kw_end].is_ascii_alphanumeric() || matches!(bytes[kw_end], b'-' | b'_'))
    {
        kw_end += 1;
    }
    if kw_end == kw_start {
        return None;
    }
    let kw = &bytes[kw_start..kw_end];
    let eq_kw = |s: &str| -> bool {
        kw.len() == s.len()
            && kw
                .iter()
                .zip(s.bytes())
                .all(|(a, b)| a.eq_ignore_ascii_case(&b))
    };

    if STATEMENT_AT_RULES.iter().any(|&s| eq_kw(s)) {
        let end = bytes[kw_end..]
            .iter()
            .position(|&b| b == b';')
            .map_or(bytes.len(), |pos| kw_end + pos + 1);
        return Some(end);
    }

    let mut q = kw_end;
    while q < bytes.len() && bytes[q] != b'{' && bytes[q] != b';' {
        q += 1;
    }
    if q >= bytes.len() {
        return Some(bytes.len());
    }
    if bytes[q] == b';' {
        return Some(q + 1);
    }
    // bytes[q] == b'{'
    if NON_NESTED_BLOCK_AT_RULES.iter().any(|&s| eq_kw(s)) {
        return Some(skip_balanced_block(bytes, q));
    }
    Some(q + 1)
}

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

fn is_already_nested(selector: &str) -> bool {
    let bytes = selector.as_bytes();
    let (mut bracket, mut paren) = (0usize, 0usize);
    let (mut in_q, mut qc) = (false, 0u8);
    for &b in bytes {
        if !in_q && (b == b'"' || b == b'\'') {
            in_q = true;
            qc = b;
            continue;
        }
        if in_q {
            if b == qc {
                in_q = false;
            }
            continue;
        }
        match b {
            b'[' => bracket += 1,
            b']' => bracket = bracket.saturating_sub(1),
            b'(' => paren += 1,
            b')' => paren = paren.saturating_sub(1),
            b'&' if bracket == 0 && paren == 0 => return true,
            _ => {}
        }
    }
    false
}

fn find_selector_start(bytes: &[u8], start: usize) -> Option<usize> {
    bytes[start..]
        .iter()
        .position(|&b| matches!(b, b'.' | b'#' | b'[' | b':' | b'*' | b'a'..=b'z' | b'A'..=b'Z'))
        .map(|pos| start + pos)
}

fn find_next_brace(bytes: &[u8], start: usize) -> Option<usize> {
    for (offset, &byte) in bytes[start..].iter().enumerate() {
        if byte == b'{' {
            return Some(start + offset);
        }
        if byte == b'@' || byte == b'}' {
            return None;
        }
    }
    None
}

fn split_descendant_selector(selector: &str) -> Option<(&str, &str)> {
    let bytes = selector.as_bytes();
    let (mut bracket, mut paren) = (0usize, 0usize);
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'[' => bracket += 1,
            b']' => bracket = bracket.saturating_sub(1),
            b'(' => paren += 1,
            b')' => paren = paren.saturating_sub(1),
            b' ' | b'>' | b'+' | b'~' if bracket == 0 && paren == 0 => {
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
    fn test_attribute_selector() {
        let linter = create_linter();
        let result = linter.lint("[data-foo=\"bar baz\"] { color: red; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_nested_selector_list_does_not_warn() {
        // CSS nesting syntax: the `&` parent selector means the rule is already nested.
        // See https://github.com/ubugeeei-prod/vize/issues/2246.
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
            "@keyframes body should not warn; diagnostics: {:?}",
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
        let result = linter.lint("@font-face { font-family: \"X\"; src: url(x.woff2); }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_media_query_still_warns_on_descendants() {
        // Conditional group rules should still descend into their bodies.
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

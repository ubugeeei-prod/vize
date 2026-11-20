//! css/prefer-slotted
//!
//! Recommend using ::v-slotted() selector for styling slot content.
//!
//! When using scoped styles, slot content from parent components
//! is not affected by the child's scoped styles. Use ::v-slotted()
//! to explicitly target slot content.
//!
//! ## Vue 3.3+ Features
//!
//! - `::v-slotted(.class)` - Style slot content
//! - `::v-deep(.class)` - Style deep child components
//! - `::v-global(.class)` - Escape scoped styles
//!
//! ## Examples
//!
//! ### Using ::v-slotted()
//! ```css
//! <style scoped>
//! ::v-slotted(.content) {
//!   color: red;
//! }
//! </style>
//! ```

use cssparser::{Parser, ParserInput, Token};
use vize_l0::Span;
use vize_l1::css::StyleSheet;

use crate::diagnostic::Severity;

use super::style_rule::{all_rule_preludes, masked, skip_block};
use super::{CssLintResult, CssRuleMeta, StyleRule};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/prefer-slotted",
    description: "Recommend ::v-slotted() for styling slot content",
    default_severity: Severity::Warning,
};

const DEPRECATED_DEEP: [&str; 3] = [">>>", "/deep/", "::v-deep"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectorToken {
    Separator,
    Comma,
    Colon,
    Delim(char),
    Slot,
    Deep,
    VDeep,
    Other,
}

#[derive(Clone, Copy)]
struct SpannedToken {
    kind: SelectorToken,
    span: Span,
}

struct SelectorScan {
    tokens: Vec<SpannedToken>,
    slotted: bool,
}

fn scan_selector(sheet: &StyleSheet<'_>, prelude: Span) -> SelectorScan {
    // `masked` preserves terminated block comments as tokens. Ignoring those
    // tokens keeps `./**/slot` in one compound while line comments remain
    // byte-for-byte blanks and therefore preserve authored offsets.
    let text = masked(sheet, prelude);
    let mut input = ParserInput::new(&text);
    let mut parser = Parser::new(&mut input);
    let mut tokens = Vec::new();
    let mut slotted = false;
    while !parser.is_exhausted() {
        let start = parser.position().byte_index();
        let Ok(token) = parser.next_including_whitespace_and_comments() else {
            break;
        };
        let kind = match token {
            Token::WhiteSpace(_) => SelectorToken::Separator,
            Token::Comment(_) => continue,
            Token::Comma => SelectorToken::Comma,
            Token::Colon => SelectorToken::Colon,
            Token::Delim(value) => SelectorToken::Delim(*value),
            Token::Ident(value) if value.eq_ignore_ascii_case("slot") => SelectorToken::Slot,
            Token::Ident(value) if value.eq_ignore_ascii_case("deep") => SelectorToken::Deep,
            Token::Ident(value) if value.eq_ignore_ascii_case("v-deep") => SelectorToken::VDeep,
            Token::Function(value) => {
                if value.eq_ignore_ascii_case("slotted") || value.eq_ignore_ascii_case("v-slotted")
                {
                    slotted = true;
                }
                skip_block(&mut parser);
                SelectorToken::Other
            }
            Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock => {
                skip_block(&mut parser);
                SelectorToken::Other
            }
            _ => SelectorToken::Other,
        };
        let end = parser.position().byte_index();
        tokens.push(SpannedToken {
            kind,
            span: Span::new(prelude.start + start as u32, prelude.start + end as u32),
        });
    }
    SelectorScan { tokens, slotted }
}

fn warn_deprecated_deep(scan: &SelectorScan, result: &mut CssLintResult) {
    const PATTERNS: [[SelectorToken; 3]; 3] = [
        [
            SelectorToken::Delim('>'),
            SelectorToken::Delim('>'),
            SelectorToken::Delim('>'),
        ],
        [
            SelectorToken::Delim('/'),
            SelectorToken::Deep,
            SelectorToken::Delim('/'),
        ],
        [
            SelectorToken::Colon,
            SelectorToken::Colon,
            SelectorToken::VDeep,
        ],
    ];
    let mut index = 0;
    while index < scan.tokens.len() {
        let Some(span) = PATTERNS
            .iter()
            .find_map(|pattern| adjacent_pattern(&scan.tokens, index, pattern))
        else {
            index += 1;
            continue;
        };
        result.warn(
            META.name,
            "Deprecated deep selector syntax",
            "Use :deep(.class) or ::v-deep(.class) with parentheses in Vue 3",
            span.start,
            span.end,
        );
        index += 3;
    }
}

fn adjacent_pattern(
    tokens: &[SpannedToken],
    index: usize,
    pattern: &[SelectorToken; 3],
) -> Option<Span> {
    let window = tokens.get(index..index + pattern.len())?;
    let [first, second, third] = window else {
        return None;
    };
    (window
        .iter()
        .zip(pattern)
        .all(|(token, expected)| token.kind == *expected)
        && first.span.end == second.span.start
        && second.span.end == third.span.start)
        .then(|| Span::new(first.span.start, third.span.end))
}

fn slot_starts_type_selector(tokens: &[SpannedToken], index: usize) -> bool {
    index == 0
        || tokens.get(index - 1).is_some_and(|token| {
            matches!(
                token.kind,
                SelectorToken::Separator
                    | SelectorToken::Comma
                    | SelectorToken::Delim('>' | '+' | '~')
            )
        })
}

/// It reads every rule's selector in the L1 structure, in every style
/// language, with comments masked. The deprecated deep combinators are not
/// CSS, so the lightningcss version never saw a block that held one.
pub struct PreferSlotted;

impl StyleRule for PreferSlotted {
    fn meta(&self) -> &'static CssRuleMeta {
        &META
    }

    fn check(&self, sheet: &StyleSheet<'_>, result: &mut CssLintResult) {
        let candidate = |text: &str| {
            text.contains("slot") || DEPRECATED_DEEP.iter().any(|deep| text.contains(deep))
        };
        let mut slotted: Option<bool> = None;
        for prelude in all_rule_preludes(sheet) {
            if !candidate(sheet.text(prelude)) {
                continue;
            }
            let scan = scan_selector(sheet, prelude);
            warn_deprecated_deep(&scan, result);
            if !scan
                .tokens
                .iter()
                .any(|token| token.kind == SelectorToken::Slot)
            {
                continue;
            }
            let slotted = *slotted.get_or_insert_with(|| {
                all_rule_preludes(sheet).any(|prelude| {
                    candidate(sheet.text(prelude)) && scan_selector(sheet, prelude).slotted
                })
            });
            if slotted {
                continue;
            }
            for (index, token) in scan.tokens.iter().enumerate() {
                if token.kind != SelectorToken::Slot
                    || !slot_starts_type_selector(&scan.tokens, index)
                {
                    continue;
                }
                result.warn(
                    META.name,
                    "Consider using ::v-slotted() to style slot content in scoped styles",
                    "Use `::v-slotted(selector)` to explicitly target content passed to slots",
                    token.span.start,
                    token.span.end,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PreferSlotted;
    use crate::rules::css::CssLinter;

    fn create_linter() -> CssLinter {
        let mut linter = CssLinter::new();
        linter.add_style_rule(Box::new(PreferSlotted));
        linter
    }

    #[test]
    fn test_valid_v_slotted() {
        let linter = create_linter();
        let result = linter.lint("::v-slotted(.content) { color: red; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_normal_selector() {
        let linter = create_linter();
        let result = linter.lint(".button { color: red; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warns_deprecated_deep_combinators() {
        let linter = create_linter();
        let source = ".a >>> .b { color: red; }\n.c /deep/ .d { color: red; }\n.e ::v-deep .f { color: red; }\n.g ::v-deep(.h) { color: red; }";
        let reported: Vec<_> = linter
            .lint(source, 0)
            .diagnostics
            .iter()
            .map(|diagnostic| &source[diagnostic.start as usize..diagnostic.end as usize])
            .collect();
        assert_eq!(reported, [">>>", "/deep/", "::v-deep"]);
    }

    #[test]
    fn test_reads_preprocessor_selectors() {
        use vize_l1::css::lang::StyleLanguage;
        let linter = create_linter();
        let scss = ".a {\n  // slot is fine in a comment\n  slot > .b { color: red; }\n  .c >>> .d { color: red; }\n}";
        let result = linter.lint_block(scss, 0, StyleLanguage::Scss, true);
        let reported: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| &scss[diagnostic.start as usize..diagnostic.end as usize])
            .collect();
        assert_eq!(reported, ["slot", ">>>"]);
        let stylus = "slot\n  color red\n";
        assert_eq!(
            linter
                .lint_block(stylus, 0, StyleLanguage::Stylus, true)
                .warning_count,
            1
        );
    }

    #[test]
    fn test_ignores_slot_and_deep_selectors_inside_comments() {
        let linter = create_linter();
        let source = r#"
/*
 * Layout notes:
 *   ├ slot     40px
 *   └ details  14px
 *
 * The old `.legacy::v-deep .inner` rule was removed.
 * 旧 `.a .b >>> .c` は無効だった
 */
.field-box {
  display: block;
  & :deep(.child-control) {
    padding: 0 12px;
  }
}
"#;
        let result = linter.lint(source, 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warns_slot_selector() {
        let linter = create_linter();
        // Valid CSS that uses slot element directly
        let result = linter.lint("slot .content { color: red; }", 0);
        assert!(result.warning_count >= 1);
    }

    #[test]
    fn test_tokenizes_deep_and_slot_selectors() {
        let linter = create_linter();
        let source = r#"
[data-label="before >>> after"] { color: red; }
.a>>>.b { color: red; }
slot+.label { color: red; }
slot-label { color: red; }
./**/slot { color: red; }
"#;
        let reported: Vec<_> = linter
            .lint(source, 0)
            .diagnostics
            .iter()
            .map(|diagnostic| &source[diagnostic.start as usize..diagnostic.end as usize])
            .collect();
        assert_eq!(reported, [">>>", "slot"]);
    }

    #[test]
    fn test_only_actual_slotted_selector_suppresses_slot_warning() {
        use vize_l1::css::lang::StyleLanguage;

        let linter = create_linter();
        let declarations_only = "slot { color: red; } .demo::before { content: \":slotted\"; }";
        assert_eq!(
            linter
                .lint_block(declarations_only, 0, StyleLanguage::Scss, true)
                .warning_count,
            1
        );
        let actual = "slot { color: red; } ::v-slotted(.demo) { color: blue; }";
        assert_eq!(
            linter
                .lint_block(actual, 0, StyleLanguage::Scss, true)
                .warning_count,
            0
        );
    }
}

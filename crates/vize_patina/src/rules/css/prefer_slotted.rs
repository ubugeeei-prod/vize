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

use vize_l0::Span;
use vize_l1::css::StyleSheet;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::style_rule::{rule_preludes, without_comments};
use super::{CssLintResult, CssRuleMeta, StyleRule};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/prefer-slotted",
    description: "Recommend ::v-slotted() for styling slot content",
    default_severity: Severity::Warning,
};

/// Deprecated deep-selector spellings, each followed by whitespace or the
/// end of the selector.
const DEPRECATED_DEEP: [&str; 3] = [">>>", "/deep/", "::v-deep"];

/// It reads every rule's selector in the L1 structure, in every style
/// language, with comments masked. The deprecated deep combinators are not
/// CSS, so the lightningcss version never saw a block that held one.
pub struct PreferSlotted;

impl StyleRule for PreferSlotted {
    fn meta(&self) -> &'static CssRuleMeta {
        &META
    }

    fn check(&self, sheet: &StyleSheet<'_>, result: &mut CssLintResult) {
        let whole = Span::new(
            sheet.nodes().first().map_or(0, |node| node.span().start),
            sheet.nodes().last().map_or(0, |node| node.span().end),
        );
        let code = without_comments(sheet, whole);
        let slotted = code.contains("::v-slotted") || code.contains(":slotted");
        for prelude in rule_preludes(sheet) {
            let text = without_comments(sheet, prelude);
            let bytes = text.as_bytes();
            let at = |offset: usize, len: usize| {
                Span::new(
                    prelude.start + offset as u32,
                    prelude.start + (offset + len) as u32,
                )
            };
            for pattern in DEPRECATED_DEEP {
                for (offset, _) in text.match_indices(pattern) {
                    let next = bytes.get(offset + pattern.len());
                    if next.is_none_or(u8::is_ascii_whitespace) {
                        let span = at(offset, pattern.len());
                        result.add_diagnostic(
                            LintDiagnostic::warn(
                                META.name,
                                "Deprecated deep selector syntax",
                                span.start,
                                span.end,
                            )
                            .with_help(
                                "Use :deep(.class) or ::v-deep(.class) with parentheses in Vue 3",
                            ),
                        );
                    }
                }
            }
            if slotted {
                continue;
            }
            for (offset, _) in text.match_indices("slot") {
                let before = offset
                    .checked_sub(1)
                    .and_then(|previous| bytes.get(previous));
                let after = bytes.get(offset + 4);
                let starts =
                    before.is_none_or(|byte| matches!(byte, b' ' | b'\n' | b'\t' | b',' | b'>'));
                let ends = after.is_none_or(|byte| {
                    matches!(byte, b' ' | b'\n' | b'\t' | b'.' | b'[' | b'>' | b',')
                });
                if starts && ends {
                    let span = at(offset, 4);
                    result.add_diagnostic(
                        LintDiagnostic::warn(
                            META.name,
                            "Consider using ::v-slotted() to style slot content in scoped styles",
                            span.start,
                            span.end,
                        )
                        .with_help("Use `::v-slotted(selector)` to explicitly target content passed to slots"),
                    );
                }
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
}

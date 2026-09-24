//! Directive disable ranges and suppression pragma handling.

use vize_s0::{CompactString, directive::DirectiveSeverity};

use super::{DisabledRange, LintContext, eslint_directive::parse_eslint_disable_comment};

impl LintContext<'_> {
    /// Check if a rule is disabled at a specific line.
    #[inline]
    pub(super) fn is_disabled_at(&self, rule_name: &str, line: u32) -> bool {
        for range in &self.disabled_all {
            if line >= range.start_line {
                if let Some(end) = range.end_line {
                    if line <= end {
                        return true;
                    }
                } else {
                    return true;
                }
            }
        }

        if let Some(ranges) = self.disabled_rules.get(rule_name) {
            for range in ranges {
                if line >= range.start_line {
                    if let Some(end) = range.end_line {
                        if line <= end {
                            return true;
                        }
                    } else {
                        return true;
                    }
                }
            }
        }

        self.inline_suppressions.is_disabled_at(rule_name, line)
    }

    /// Returns true if `rule_name` is disabled at the line containing
    /// `offset`.
    ///
    /// A rule can anchor a suppression check to a stable offset (for example,
    /// an element's opening-tag start) so that an `eslint-disable-next-line`
    /// keeps applying even when the formatter moves the exact diagnostic span
    /// onto a different line. (#3252)
    #[inline]
    pub fn is_rule_disabled_at_offset(&self, rule_name: &str, offset: u32) -> bool {
        self.is_disabled_at(rule_name, self.offset_to_line(offset))
    }

    /// Disable all rules starting from a line.
    pub fn disable_all(&mut self, start_line: u32, end_line: Option<u32>) {
        self.disabled_all.push(DisabledRange {
            start_line,
            end_line,
        });
    }

    /// Disable specific rules starting from a line.
    pub fn disable_rules(&mut self, rules: &[&str], start_line: u32, end_line: Option<u32>) {
        for rule in rules {
            let range = DisabledRange {
                start_line,
                end_line,
            };
            self.disabled_rules
                .entry(CompactString::from(*rule))
                .or_default()
                .push(range);
        }
    }

    /// Register an actual template comment. The visitor supplies AST comment
    /// nodes, so text and attribute values cannot introduce lint directives.
    pub(crate) fn register_lint_comment(&mut self, content: &str, offset: u32) {
        let Some(directive) = parse_eslint_disable_comment(content) else {
            return;
        };
        let marker_offset = content
            .find("eslint-")
            .into_iter()
            .chain(content.find("oxlint-"))
            .min()
            .unwrap_or(0);
        let preceding_lines = content
            .as_bytes()
            .get(..marker_offset)
            .unwrap_or_default()
            .iter()
            .filter(|&&byte| byte == b'\n')
            .count() as u32;
        let line = self.offset_to_line(offset) + preceding_lines;
        self.inline_suppressions.record(directive, line);
    }

    /// JSX still uses the source scanner until its own comment AST is wired in.
    pub(super) fn prescan_eslint_disable_comments(&mut self) {
        if !self.source.contains("eslint-") && !self.source.contains("oxlint-") {
            return;
        }
        for (line_number, line) in (1u32..).zip(self.source.lines()) {
            if !line.contains("eslint-") && !line.contains("oxlint-") {
                continue;
            }
            if let Some(directive) = parse_eslint_disable_comment(line) {
                self.inline_suppressions.record(directive, line_number);
            }
        }
    }

    /// Begin a `@vize:ignore-start` region (disables all rules from this line).
    pub fn push_ignore_region(&mut self, line: u32) {
        self.disable_all(line, None);
    }

    /// End a `@vize:ignore-end` region (closes the most recent open ignore region).
    pub fn pop_ignore_region(&mut self, line: u32) {
        for range in self.disabled_all.iter_mut().rev() {
            if range.end_line.is_none() {
                range.end_line = Some(line);
                return;
            }
        }
    }

    /// Register that `@vize:expected` expects an error on the next line.
    pub fn expect_error_next_line(&mut self, current_line: u32) {
        self.expected_error_lines.insert(current_line + 1);
    }

    /// Set a severity override for diagnostics on the next line.
    pub fn set_severity_override_next_line(
        &mut self,
        current_line: u32,
        severity: DirectiveSeverity,
    ) {
        self.severity_overrides.insert(current_line + 1, severity);
    }
}

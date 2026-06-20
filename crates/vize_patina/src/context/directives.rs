//! Directive disable ranges and suppression pragma handling.

use vize_carton::{CompactString, String, directive::DirectiveSeverity};

use super::{
    DisabledRange, LintContext,
    eslint_directive::{EslintDisableKind, parse_eslint_disable_comment},
};

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

        false
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

    fn disable_rule_names<'r, I>(&mut self, rules: I, start_line: u32, end_line: Option<u32>)
    where
        I: IntoIterator<Item = &'r str>,
    {
        for rule in rules {
            let range = DisabledRange {
                start_line,
                end_line,
            };
            self.disabled_rules
                .entry(CompactString::from(rule))
                .or_default()
                .push(range);
        }
    }

    fn enable_all_rules(&mut self, line: u32) {
        for range in &mut self.disabled_all {
            if range.end_line.is_none() {
                range.end_line = Some(line);
            }
        }
        for ranges in self.disabled_rules.values_mut() {
            for range in ranges {
                if range.end_line.is_none() {
                    range.end_line = Some(line);
                }
            }
        }
    }

    fn enable_rule_names<'r, I>(&mut self, rules: I, line: u32)
    where
        I: IntoIterator<Item = &'r str>,
    {
        for rule in rules {
            if let Some(ranges) = self.disabled_rules.get_mut(rule) {
                for range in ranges {
                    if range.end_line.is_none() {
                        range.end_line = Some(line);
                    }
                }
            }
        }
    }

    pub(super) fn prescan_eslint_disable_comments(&mut self) {
        if !self.source.contains("eslint-") {
            return;
        }
        for (line_number, line) in (1u32..).zip(self.source.lines()) {
            if !line.contains("eslint-") {
                continue;
            }
            if let Some(directive) = parse_eslint_disable_comment(line) {
                match directive.kind {
                    EslintDisableKind::DisableNextLine => {
                        self.apply_eslint_disable(
                            line_number + 1,
                            Some(line_number + 1),
                            directive.rules,
                        );
                    }
                    EslintDisableKind::DisableLine => {
                        self.apply_eslint_disable(line_number, Some(line_number), directive.rules);
                    }
                    EslintDisableKind::Disable => {
                        self.apply_eslint_disable(line_number, None, directive.rules);
                    }
                    EslintDisableKind::Enable => {
                        if directive.rules.is_empty() {
                            self.enable_all_rules(line_number);
                        } else {
                            self.enable_rule_names(
                                directive.rules.iter().map(String::as_str),
                                line_number,
                            );
                        }
                    }
                }
            }
        }
    }

    fn apply_eslint_disable(&mut self, start_line: u32, end_line: Option<u32>, rules: Vec<String>) {
        if rules.is_empty() {
            self.disable_all(start_line, end_line);
        } else {
            self.disable_rule_names(rules.iter().map(String::as_str), start_line, end_line);
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

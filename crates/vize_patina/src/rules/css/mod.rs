//! CSS lint rules for Vue.js SFC style blocks.
//!
//! These rules check CSS/SCSS/Less code in `<style>` blocks using lightning-css
//! for high-performance parsing.
//!
//! ## Enabling CSS Rules
//!
//! CSS rules can be enabled in your configuration:
//!
//! ```toml
//! [rules.css]
//! "no-important" = "warn"
//! "no-id-selectors" = "warn"
//! ```
//!
//! ## Inline Disable Comments
//!
//! Rules can be disabled inline using CSS comments:
//!
//! ```css
//! /* vize-disable css/no-important */
//! .foo { color: red !important; }
//! /* vize-enable css/no-important */
//!
//! /* vize-disable-next-line css/no-important */
//! .bar { color: blue !important; }
//!
//! .baz { color: green !important; } /* vize-disable-line css/no-important */
//! ```

#![allow(clippy::disallowed_macros)]

mod no_display_none;
mod no_hardcoded_values;
mod no_id_selectors;
mod no_important;
mod no_utility_classes;
mod no_v_bind_performance;
mod prefer_logical_properties;
mod prefer_nested_selectors;
mod prefer_slotted;
mod require_font_display;

use vize_carton::FxHashSet;

use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use memchr::memmem;

use crate::diagnostic::{LintDiagnostic, Severity};
use vize_carton::String;
use vize_carton::ToCompactString;

pub use no_display_none::NoDisplayNone;
pub use no_hardcoded_values::NoHardcodedValues;
pub use no_id_selectors::NoIdSelectors;
pub use no_important::NoImportant;
pub use no_utility_classes::NoUtilityClasses;
pub use no_v_bind_performance::NoVBindPerformance;
pub use prefer_logical_properties::PreferLogicalProperties;
pub use prefer_nested_selectors::PreferNestedSelectors;
pub use prefer_slotted::PreferSlotted;
pub use require_font_display::RequireFontDisplay;

/// Metadata for a CSS rule
pub struct CssRuleMeta {
    /// Rule name (e.g., "css/no-important")
    pub name: &'static str,
    /// Rule description
    pub description: &'static str,
    /// Default severity
    pub default_severity: Severity,
}

/// Result of linting a style block
#[derive(Debug, Default)]
pub struct CssLintResult {
    pub diagnostics: Vec<LintDiagnostic>,
    pub error_count: usize,
    pub warning_count: usize,
}

impl CssLintResult {
    pub fn add_diagnostic(&mut self, diagnostic: LintDiagnostic) {
        match diagnostic.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
        }
        self.diagnostics.push(diagnostic);
    }
}

/// Trait for CSS lint rules
pub trait CssRule: Send + Sync {
    /// Get rule metadata
    fn meta(&self) -> &'static CssRuleMeta;

    /// Check the CSS content using the parsed stylesheet
    ///
    /// * `source` - The original CSS source
    /// * `stylesheet` - The parsed stylesheet from lightning-css
    /// * `offset` - The offset of the style block in the original file
    /// * `result` - Accumulator for diagnostics
    fn check<'i>(
        &self,
        source: &'i str,
        stylesheet: &StyleSheet<'i, 'i>,
        offset: usize,
        result: &mut CssLintResult,
    );
}

/// Tracks disabled rules from inline comments
#[derive(Debug, Default)]
pub struct DisabledRules {
    /// Rules disabled from a specific line onwards (until re-enabled)
    /// Maps (start_line, rule_name) to enabled status
    block_disabled: Vec<(usize, String, bool)>,
    /// Rules disabled for a specific line only
    /// Maps line_number to set of disabled rule names
    line_disabled: Vec<(usize, FxHashSet<String>)>,
    /// Rules disabled for the next line only
    next_line_disabled: Vec<(usize, FxHashSet<String>)>,
}

impl DisabledRules {
    /// Parse disable comments from CSS source
    pub fn parse(source: &str) -> Self {
        let mut result = Self::default();
        let bytes = source.as_bytes();

        // Pattern matchers
        let disable_finder = memmem::Finder::new(b"vize-disable ");
        let enable_finder = memmem::Finder::new(b"vize-enable ");
        let disable_line_finder = memmem::Finder::new(b"vize-disable-line ");
        let disable_next_line_finder = memmem::Finder::new(b"vize-disable-next-line ");

        // Track line numbers
        let mut line_starts: Vec<usize> = vec![0];
        for (i, &b) in bytes.iter().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }

        let get_line_number =
            |pos: usize| -> usize { line_starts.partition_point(|&start| start <= pos) };

        // Find block disable/enable comments
        let mut search_start = 0;
        while let Some(pos) = disable_finder.find(&bytes[search_start..]) {
            let abs_pos = search_start + pos;
            // Check if inside a CSS comment
            if Self::is_in_css_comment(bytes, abs_pos) {
                let line = get_line_number(abs_pos);
                let rule_name = Self::extract_rule_name(source, abs_pos + 13); // "vize-disable ".len()
                if !rule_name.is_empty() {
                    result.block_disabled.push((line, rule_name, true));
                }
            }
            search_start = abs_pos + 1;
        }

        search_start = 0;
        while let Some(pos) = enable_finder.find(&bytes[search_start..]) {
            let abs_pos = search_start + pos;
            if Self::is_in_css_comment(bytes, abs_pos) {
                let line = get_line_number(abs_pos);
                let rule_name = Self::extract_rule_name(source, abs_pos + 12); // "vize-enable ".len()
                if !rule_name.is_empty() {
                    result.block_disabled.push((line, rule_name, false));
                }
            }
            search_start = abs_pos + 1;
        }

        // Find line-specific disable comments
        search_start = 0;
        while let Some(pos) = disable_line_finder.find(&bytes[search_start..]) {
            let abs_pos = search_start + pos;
            if Self::is_in_css_comment(bytes, abs_pos) {
                let line = get_line_number(abs_pos);
                let rule_name = Self::extract_rule_name(source, abs_pos + 18); // "vize-disable-line ".len()
                if !rule_name.is_empty() {
                    if let Some((_, set)) =
                        result.line_disabled.iter_mut().find(|(l, _)| *l == line)
                    {
                        set.insert(rule_name);
                    } else {
                        let mut set = FxHashSet::default();
                        set.insert(rule_name);
                        result.line_disabled.push((line, set));
                    }
                }
            }
            search_start = abs_pos + 1;
        }

        // Find next-line disable comments
        search_start = 0;
        while let Some(pos) = disable_next_line_finder.find(&bytes[search_start..]) {
            let abs_pos = search_start + pos;
            if Self::is_in_css_comment(bytes, abs_pos) {
                let line = get_line_number(abs_pos);
                let rule_name = Self::extract_rule_name(source, abs_pos + 23); // "vize-disable-next-line ".len()
                if !rule_name.is_empty() {
                    let next_line = line + 1;
                    if let Some((_, set)) = result
                        .next_line_disabled
                        .iter_mut()
                        .find(|(l, _)| *l == next_line)
                    {
                        set.insert(rule_name);
                    } else {
                        let mut set = FxHashSet::default();
                        set.insert(rule_name);
                        result.next_line_disabled.push((next_line, set));
                    }
                }
            }
            search_start = abs_pos + 1;
        }

        // Sort block_disabled by line number for consistent processing
        result
            .block_disabled
            .sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.2.cmp(&b.2).reverse()));

        result
    }

    /// Check if a position is inside a CSS comment
    fn is_in_css_comment(bytes: &[u8], pos: usize) -> bool {
        if pos < 2 {
            return false;
        }
        // Look backwards for /* and make sure no */ before pos
        let mut i = pos.saturating_sub(1);
        loop {
            if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
                // Found comment start, now check there's no close before pos
                for j in (i + 2)..pos {
                    if j + 1 < bytes.len() && bytes[j] == b'*' && bytes[j + 1] == b'/' {
                        return false;
                    }
                }
                return true;
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
        false
    }

    /// Extract rule name from position
    fn extract_rule_name(source: &str, start: usize) -> String {
        let bytes = source.as_bytes();
        if start >= bytes.len() {
            return String::default();
        }

        let mut end = start;
        while end < bytes.len() {
            let b = bytes[end];
            if b.is_ascii_alphanumeric() || b == b'-' || b == b'/' || b == b'_' {
                end += 1;
            } else {
                break;
            }
        }

        (&source[start..end]).to_compact_string()
    }

    /// Check if a rule is disabled at a given line
    pub fn is_disabled(&self, rule_name: &str, line: usize) -> bool {
        // Check line-specific disables
        if let Some((_, set)) = self.line_disabled.iter().find(|(l, _)| *l == line) {
            if set.contains(rule_name) {
                return true;
            }
        }

        // Check next-line disables
        if let Some((_, set)) = self.next_line_disabled.iter().find(|(l, _)| *l == line) {
            if set.contains(rule_name) {
                return true;
            }
        }

        // Check block disables (need to track state across all lines up to this one)
        let mut disabled = false;
        for (block_line, name, is_disable) in &self.block_disabled {
            // Only consider entries at or before the current line
            if *block_line <= line && name == rule_name {
                disabled = *is_disable;
            }
        }

        disabled
    }
}

/// Strip vize disable comments from CSS source for compilation
pub fn strip_vize_comments(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let bytes = source.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Check for comment start
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            // Find comment end
            let comment_start = i;
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            if i + 1 < bytes.len() {
                i += 2; // Skip */
            }

            let comment = &source[comment_start..i];
            // Only strip vize-related comments
            if !comment.contains("vize-disable")
                && !comment.contains("vize-enable")
                && !comment.contains("vize-disable-line")
                && !comment.contains("vize-disable-next-line")
            {
                result.push_str(comment);
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

/// Linter for style blocks using lightning-css
pub struct CssLinter {
    rules: Vec<Box<dyn CssRule>>,
}

impl CssLinter {
    /// Create a new CSS linter with no rules
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Create a CSS linter with all available rules
    pub fn with_all_rules() -> Self {
        Self {
            rules: vec![
                Box::new(NoImportant),
                Box::new(NoIdSelectors),
                Box::new(PreferLogicalProperties),
                Box::new(RequireFontDisplay),
                Box::new(PreferNestedSelectors),
                Box::new(NoDisplayNone),
                Box::new(NoVBindPerformance),
                Box::new(NoHardcodedValues::default()),
                Box::new(NoUtilityClasses),
                Box::new(PreferSlotted),
            ],
        }
    }

    /// Add a rule to the linter
    pub fn add_rule(&mut self, rule: Box<dyn CssRule>) {
        self.rules.push(rule);
    }

    /// Lint a style block
    pub fn lint(&self, source: &str, offset: usize) -> CssLintResult {
        self.lint_with_options(source, offset, true)
    }

    /// Lint a style block with options
    pub fn lint_with_options(
        &self,
        source: &str,
        offset: usize,
        respect_disable_comments: bool,
    ) -> CssLintResult {
        let mut result = CssLintResult::default();

        // Parse disable comments
        let disabled_rules = if respect_disable_comments {
            Some(DisabledRules::parse(source))
        } else {
            None
        };

        // Parse CSS with lightning-css
        let stylesheet = match StyleSheet::parse(source, ParserOptions::default()) {
            Ok(ss) => ss,
            Err(_) => {
                // If parsing fails, skip CSS linting
                return result;
            }
        };

        for rule in &self.rules {
            rule.check(source, &stylesheet, offset, &mut result);
        }

        // Filter out disabled diagnostics
        if let Some(disabled) = disabled_rules {
            let line_starts: Vec<usize> = std::iter::once(0)
                .chain(source.bytes().enumerate().filter_map(|(i, b)| {
                    if b == b'\n' {
                        Some(i + 1)
                    } else {
                        None
                    }
                }))
                .collect();

            let get_line =
                |pos: u32| -> usize { line_starts.partition_point(|&start| start <= pos as usize) };

            result.diagnostics.retain(|d| {
                let line = get_line(d.start);
                !disabled.is_disabled(d.rule_name, line)
            });

            // Recalculate counts
            result.error_count = result
                .diagnostics
                .iter()
                .filter(|d| matches!(d.severity, Severity::Error))
                .count();
            result.warning_count = result
                .diagnostics
                .iter()
                .filter(|d| matches!(d.severity, Severity::Warning))
                .count();
        }

        result
    }
}

impl Default for CssLinter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod disable_tests {
    use super::{strip_vize_comments, CssLinter, DisabledRules};

    #[test]
    fn test_parse_disable_comments() {
        let source = "/* vize-disable css/no-important */\n.foo { color: red !important; }";
        let disabled = DisabledRules::parse(source);
        println!("block_disabled: {:?}", disabled.block_disabled);
        assert!(
            !disabled.block_disabled.is_empty(),
            "block_disabled should not be empty"
        );
        assert!(
            disabled.is_disabled("css/no-important", 2),
            "css/no-important should be disabled on line 2"
        );
    }

    #[test]
    fn test_parse_next_line_comments() {
        let source =
            "/* vize-disable-next-line css/no-important */\n.foo { color: red !important; }";
        let disabled = DisabledRules::parse(source);
        println!("next_line_disabled: {:?}", disabled.next_line_disabled);
        println!("block_disabled: {:?}", disabled.block_disabled);
        println!("line_disabled: {:?}", disabled.line_disabled);
        assert!(
            !disabled.next_line_disabled.is_empty(),
            "next_line_disabled should not be empty"
        );
        assert!(
            disabled.is_disabled("css/no-important", 2),
            "css/no-important should be disabled on line 2"
        );
    }

    #[test]
    fn test_disable_line() {
        let linter = CssLinter::with_all_rules();
        let source = ".foo { color: red !important; } /* vize-disable-line css/no-important */";
        let result = linter.lint(source, 0);
        // Should not have warnings for !important on this line
        assert!(!result
            .diagnostics
            .iter()
            .any(|d| d.rule_name == "css/no-important"));
    }

    #[test]
    fn test_disable_next_line() {
        let source =
            "/* vize-disable-next-line css/no-important */\n.foo { color: red !important; }";

        // First verify parsing works
        let disabled = DisabledRules::parse(source);
        println!("disabled: {:?}", disabled);
        assert!(
            !disabled.next_line_disabled.is_empty(),
            "next_line_disabled should not be empty: {:?}",
            disabled
        );

        // Calculate line starts
        let line_starts: Vec<usize> = std::iter::once(0)
            .chain(source.bytes().enumerate().filter_map(|(i, b)| {
                if b == b'\n' {
                    Some(i + 1)
                } else {
                    None
                }
            }))
            .collect();
        println!("line_starts: {:?}", line_starts);

        let get_line =
            |pos: u32| -> usize { line_starts.partition_point(|&start| start <= pos as usize) };

        let linter = CssLinter::with_all_rules();
        let result = linter.lint(source, 0);

        // Debug: print all diagnostics with calculated line numbers
        for d in &result.diagnostics {
            let line = get_line(d.start);
            println!(
                "Diagnostic: {} at byte {} (line {}), disabled={}",
                d.rule_name,
                d.start,
                line,
                disabled.is_disabled(d.rule_name, line)
            );
        }

        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.rule_name == "css/no-important"),
            "Should not have css/no-important warning, got: {:?}",
            result
                .diagnostics
                .iter()
                .map(|d| format!(
                    "{} at byte {} (line {})",
                    d.rule_name,
                    d.start,
                    get_line(d.start)
                ))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_disable_block() {
        let source = r#"/* vize-disable css/no-important */
.foo { color: red !important; }
.bar { color: blue !important; }
/* vize-enable css/no-important */
.baz { color: green !important; }"#;

        // First verify parsing works
        let disabled = DisabledRules::parse(source);
        assert!(
            disabled.block_disabled.len() >= 2,
            "block_disabled should have at least 2 entries (disable and enable): {:?}",
            disabled.block_disabled
        );

        let linter = CssLinter::with_all_rules();
        let result = linter.lint(source, 0);
        // Only .baz should have a warning
        let important_warnings: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|d| d.rule_name == "css/no-important")
            .collect();
        assert_eq!(
            important_warnings.len(),
            1,
            "Expected 1 warning, got {}: {:?}",
            important_warnings.len(),
            important_warnings
                .iter()
                .map(|d| format!("{} at {}", d.rule_name, d.start))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_strip_vize_comments() {
        let source = r#".foo { color: red; } /* vize-disable css/no-important */
.bar { color: blue !important; }
/* regular comment */
        .baz { color: green; }"#;
        let stripped = strip_vize_comments(source);
        insta::assert_snapshot!(stripped.as_str());
    }
}

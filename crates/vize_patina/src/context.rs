//! Lint context for rule execution.
//!
//! Uses arena allocation for high-performance memory management.
//! The context tracks element traversal state, scope variables,
//! disabled rule ranges, and collects diagnostics.

mod helpers;
mod state;

pub use state::{DisabledRange, ElementContext, SsrMode};

use crate::diagnostic::{HelpLevel, LintDiagnostic, Severity};
use std::borrow::Cow;
use vize_carton::String;
use vize_carton::{
    Allocator, CompactString, FxHashMap, FxHashSet,
    directive::DirectiveSeverity,
    i18n::{Locale, t, t_fmt},
};
use vize_croquis::Croquis;

/// Lint context provides utilities for rules during execution.
///
/// Uses arena allocation for efficient memory management during lint traversal.
pub struct LintContext<'a> {
    /// Arena allocator for this lint session.
    allocator: &'a Allocator,
    /// Source code being linted.
    pub source: &'a str,
    /// Filename for diagnostics.
    pub filename: &'a str,
    /// Locale for i18n (default: English).
    locale: Locale,
    /// Collected diagnostics (pre-allocated capacity).
    pub(crate) diagnostics: Vec<LintDiagnostic>,
    /// Current rule name (set by visitor before calling rule methods).
    pub current_rule: &'static str,
    /// Parent element stack for context (pre-allocated capacity).
    pub(crate) element_stack: Vec<ElementContext>,
    /// Variables in current scope (from v-for).
    pub(crate) scope_variables: FxHashSet<CompactString>,
    /// Cached error count for fast access.
    pub(crate) error_count: usize,
    /// Cached warning count for fast access.
    pub(crate) warning_count: usize,
    /// Disabled ranges for all rules.
    disabled_all: Vec<DisabledRange>,
    /// Disabled ranges per rule name.
    disabled_rules: FxHashMap<CompactString, Vec<DisabledRange>>,
    /// Line offsets for fast line number lookup.
    line_offsets: Vec<u32>,
    /// Optional set of enabled rule names (if None, all rules are enabled).
    enabled_rules: Option<FxHashSet<String>>,
    /// Optional semantic analysis from croquis.
    pub(crate) analysis: Option<&'a Croquis>,
    /// Rules that should ignore semantic analysis for this lint pass.
    analysis_excluded_rules: Option<&'static [&'static str]>,
    /// SSR mode for linting.
    ssr_mode: SsrMode,
    /// Help display level.
    pub(crate) help_level: HelpLevel,
    /// Lines where `@vize:expected` expects an error on the next line.
    expected_error_lines: FxHashSet<u32>,
    /// Severity overrides from `@vize:level(...)` keyed by next-line number.
    severity_overrides: FxHashMap<u32, DirectiveSeverity>,
}

impl<'a> LintContext<'a> {
    /// Initial capacity for diagnostics vector.
    const INITIAL_DIAGNOSTICS_CAPACITY: usize = 16;
    /// Initial capacity for element stack.
    const INITIAL_STACK_CAPACITY: usize = 32;

    /// Create a new lint context with arena allocator.
    #[inline]
    pub fn new(allocator: &'a Allocator, source: &'a str, filename: &'a str) -> Self {
        Self::with_locale(allocator, source, filename, Locale::default())
    }

    /// Create a new lint context with specified locale.
    #[inline]
    pub fn with_locale(
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        locale: Locale,
    ) -> Self {
        Self {
            allocator,
            source,
            filename,
            locale,
            diagnostics: Vec::with_capacity(Self::INITIAL_DIAGNOSTICS_CAPACITY),
            current_rule: "",
            element_stack: Vec::with_capacity(Self::INITIAL_STACK_CAPACITY),
            scope_variables: FxHashSet::default(),
            error_count: 0,
            warning_count: 0,
            disabled_all: Vec::new(),
            disabled_rules: FxHashMap::default(),
            line_offsets: Self::compute_line_offsets(source),
            enabled_rules: None,
            analysis: None,
            analysis_excluded_rules: None,
            ssr_mode: SsrMode::default(),
            help_level: HelpLevel::default(),
            expected_error_lines: FxHashSet::default(),
            severity_overrides: FxHashMap::default(),
        }
    }

    /// Create a new lint context with semantic analysis.
    #[inline]
    pub fn with_analysis(
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        analysis: &'a Croquis,
    ) -> Self {
        Self {
            allocator,
            source,
            filename,
            locale: Locale::default(),
            diagnostics: Vec::with_capacity(Self::INITIAL_DIAGNOSTICS_CAPACITY),
            current_rule: "",
            element_stack: Vec::with_capacity(Self::INITIAL_STACK_CAPACITY),
            scope_variables: FxHashSet::default(),
            error_count: 0,
            warning_count: 0,
            disabled_all: Vec::new(),
            disabled_rules: FxHashMap::default(),
            line_offsets: Self::compute_line_offsets(source),
            enabled_rules: None,
            analysis: Some(analysis),
            analysis_excluded_rules: None,
            ssr_mode: SsrMode::default(),
            help_level: HelpLevel::default(),
            expected_error_lines: FxHashSet::default(),
            severity_overrides: FxHashMap::default(),
        }
    }

    /// Set semantic analysis.
    #[inline]
    pub fn set_analysis(&mut self, analysis: &'a Croquis) {
        self.analysis = Some(analysis);
    }

    /// Exclude selected rules from seeing semantic analysis in this pass.
    #[inline]
    pub fn set_analysis_excluded_rules(&mut self, rules: &'static [&'static str]) {
        self.analysis_excluded_rules = Some(rules);
    }

    /// Get semantic analysis (if available).
    #[inline]
    pub fn analysis(&self) -> Option<&Croquis> {
        if self
            .analysis_excluded_rules
            .is_some_and(|rules| rules.contains(&self.current_rule))
        {
            return None;
        }
        self.analysis
    }

    /// Check if semantic analysis is available.
    #[inline]
    pub fn has_analysis(&self) -> bool {
        self.analysis().is_some()
    }

    /// Set SSR mode.
    #[inline]
    pub fn set_ssr_mode(&mut self, mode: SsrMode) {
        self.ssr_mode = mode;
    }

    /// Get SSR mode.
    #[inline]
    pub fn ssr_mode(&self) -> SsrMode {
        self.ssr_mode
    }

    /// Check if SSR mode is enabled.
    #[inline]
    pub fn is_ssr_enabled(&self) -> bool {
        self.ssr_mode == SsrMode::Enabled
    }

    /// Set help display level.
    #[inline]
    pub fn set_help_level(&mut self, level: HelpLevel) {
        self.help_level = level;
    }

    /// Get help display level.
    #[inline]
    pub fn help_level(&self) -> HelpLevel {
        self.help_level
    }

    /// Set enabled rules filter.
    ///
    /// If set to Some, only rules in the set will report diagnostics.
    /// If set to None (default), all rules are enabled.
    #[inline]
    pub fn set_enabled_rules(&mut self, enabled: Option<FxHashSet<String>>) {
        self.enabled_rules = enabled;
    }

    /// Check if a rule is enabled.
    #[inline]
    pub fn is_rule_enabled(&self, rule_name: &str) -> bool {
        match &self.enabled_rules {
            Some(set) => set.contains(rule_name),
            None => true,
        }
    }

    /// Get the current locale.
    #[inline]
    pub fn locale(&self) -> Locale {
        self.locale
    }

    /// Translate a message key.
    #[inline]
    pub fn t(&self, key: &str) -> Cow<'static, str> {
        t(self.locale, key)
    }

    /// Translate a message key with variable substitution.
    #[inline]
    pub fn t_fmt(&self, key: &str, vars: &[(&str, &str)]) -> String {
        t_fmt(self.locale, key, vars).into()
    }

    /// Compute line offsets for fast line number lookup.
    fn compute_line_offsets(source: &str) -> Vec<u32> {
        let mut offsets = vec![0];
        for (i, c) in source.char_indices() {
            if c == '\n' {
                offsets.push((i + 1) as u32);
            }
        }
        offsets
    }

    /// Get line number (1-indexed) from byte offset.
    #[inline]
    pub fn offset_to_line(&self, offset: u32) -> u32 {
        match self.line_offsets.binary_search(&offset) {
            Ok(line) => (line + 1) as u32,
            Err(line) => line as u32,
        }
    }

    /// Get the allocator.
    #[inline]
    pub fn allocator(&self) -> &'a Allocator {
        self.allocator
    }

    /// Allocate a string in the arena.
    #[inline]
    pub fn alloc_str(&self, s: &str) -> &'a str {
        self.allocator.alloc_str(s)
    }

    /// Report a lint diagnostic.
    #[inline]
    pub fn report(&mut self, mut diagnostic: LintDiagnostic) {
        // Check if this rule is enabled
        if !self.is_rule_enabled(diagnostic.rule_name) {
            return;
        }

        // Check if this diagnostic is disabled via comments
        let line = self.offset_to_line(diagnostic.start);
        if self.is_disabled_at(diagnostic.rule_name, line) {
            return;
        }

        // Check if this line has an @vize:expected directive
        if self.expected_error_lines.remove(&line) {
            // Error was expected -- suppress it
            return;
        }

        // Apply @vize:level severity override
        if let Some(override_severity) = self.severity_overrides.remove(&line) {
            match override_severity {
                DirectiveSeverity::Off => return,
                DirectiveSeverity::Warn => diagnostic.severity = Severity::Warning,
                DirectiveSeverity::Error => diagnostic.severity = Severity::Error,
            }
        }

        match diagnostic.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
        }
        self.diagnostics.push(diagnostic);
    }

    /// Check if a rule is disabled at a specific line.
    #[inline]
    fn is_disabled_at(&self, rule_name: &str, line: u32) -> bool {
        // Check global disables
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

        // Check rule-specific disables
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

    /// Begin a `@vize:ignore-start` region (disables all rules from this line).
    pub fn push_ignore_region(&mut self, line: u32) {
        self.disable_all(line, None);
    }

    /// End a `@vize:ignore-end` region (closes the most recent open ignore region).
    pub fn pop_ignore_region(&mut self, line: u32) {
        // Find the last disabled_all range with end_line = None and close it
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

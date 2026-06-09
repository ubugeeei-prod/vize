//! Engine wiring for the built-in `css/*` lint rules.
//!
//! The `css/*` rules implement the [`CssRule`](crate::rules::css::CssRule)
//! trait and are assembled by [`CssLinter`](crate::rules::css::CssLinter).
//! This module drives that linter from the SFC `<style>` pipeline: for every
//! enabled `css/*` rule it parses each `<style>` block once (via lightning-css,
//! inside [`CssLinter::lint`]) and merges the reported diagnostics into the
//! file-level [`LintResult`].
//!
//! Like the built-in script rules, `css/*` rules are **opt-in**: they only run
//! when the active preset or the host configuration enables them by name.

use super::{LintResult, Linter};
use crate::rules::css::{
    CssLinter, CssRule, NoDisplayNone, NoHardcodedValues, NoIdSelectors, NoImportant,
    NoUtilityClasses, NoVBindPerformance, PreferLogicalProperties, PreferNestedSelectors,
    PreferSlotted, RequireFontDisplay,
};
use vize_atelier_sfc::SfcDescriptor;
use vize_carton::profile;

pub(crate) const RULE_NO_IMPORTANT: &str = "css/no-important";
pub(crate) const RULE_NO_ID_SELECTORS: &str = "css/no-id-selectors";
pub(crate) const RULE_PREFER_LOGICAL_PROPERTIES: &str = "css/prefer-logical-properties";
pub(crate) const RULE_REQUIRE_FONT_DISPLAY: &str = "css/require-font-display";
pub(crate) const RULE_PREFER_NESTED_SELECTORS: &str = "css/prefer-nested-selectors";
pub(crate) const RULE_NO_DISPLAY_NONE: &str = "css/no-display-none";
pub(crate) const RULE_NO_V_BIND_PERFORMANCE: &str = "css/no-v-bind-performance";
pub(crate) const RULE_NO_HARDCODED_VALUES: &str = "css/no-hardcoded-values";
pub(crate) const RULE_NO_UTILITY_CLASSES: &str = "css/no-utility-classes";
pub(crate) const RULE_PREFER_SLOTTED: &str = "css/prefer-slotted";

/// The full ordered set of built-in `css/*` rule names.
///
/// Order matches [`CssLinter::with_all_rules`] so that, when every rule is
/// enabled, diagnostics keep the same ordering as the standalone linter.
const ALL_BUILTIN_CSS_RULE_NAMES: &[&str] = &[
    RULE_NO_IMPORTANT,
    RULE_NO_ID_SELECTORS,
    RULE_PREFER_LOGICAL_PROPERTIES,
    RULE_REQUIRE_FONT_DISPLAY,
    RULE_PREFER_NESTED_SELECTORS,
    RULE_NO_DISPLAY_NONE,
    RULE_NO_V_BIND_PERFORMANCE,
    RULE_NO_HARDCODED_VALUES,
    RULE_NO_UTILITY_CLASSES,
    RULE_PREFER_SLOTTED,
];

#[inline]
pub(crate) const fn all_builtin_css_rule_names() -> &'static [&'static str] {
    ALL_BUILTIN_CSS_RULE_NAMES
}

/// Whether any built-in `css/*` rule is both configured for this linter
/// (`css_rules`) and enabled (`is_rule_enabled`).
#[inline]
pub(crate) fn has_active_builtin_css_rules(linter: &Linter) -> bool {
    linter
        .css_rules
        .iter()
        .copied()
        .any(|rule_name| linter.is_rule_enabled(rule_name))
}

/// Construct the boxed [`CssRule`] for a built-in `css/*` rule name.
fn css_rule_for_name(rule_name: &str) -> Option<Box<dyn CssRule>> {
    let rule: Box<dyn CssRule> = match rule_name {
        RULE_NO_IMPORTANT => Box::new(NoImportant),
        RULE_NO_ID_SELECTORS => Box::new(NoIdSelectors),
        RULE_PREFER_LOGICAL_PROPERTIES => Box::new(PreferLogicalProperties),
        RULE_REQUIRE_FONT_DISPLAY => Box::new(RequireFontDisplay),
        RULE_PREFER_NESTED_SELECTORS => Box::new(PreferNestedSelectors),
        RULE_NO_DISPLAY_NONE => Box::new(NoDisplayNone),
        RULE_NO_V_BIND_PERFORMANCE => Box::new(NoVBindPerformance),
        RULE_NO_HARDCODED_VALUES => Box::new(NoHardcodedValues::default()),
        RULE_NO_UTILITY_CLASSES => Box::new(NoUtilityClasses),
        RULE_PREFER_SLOTTED => Box::new(PreferSlotted),
        _ => return None,
    };
    Some(rule)
}

/// Run every active built-in `css/*` rule against each `<style>` block of the
/// SFC and append the diagnostics to `result`.
pub(crate) fn append_builtin_css_diagnostics(
    linter: &Linter,
    descriptor: &SfcDescriptor<'_>,
    result: &mut LintResult,
) {
    if descriptor.styles.is_empty() {
        return;
    }

    // Build a `CssLinter` carrying only the active rules, preserving the order
    // declared in `ALL_BUILTIN_CSS_RULE_NAMES`.
    let mut css_linter = CssLinter::new();
    for rule_name in ALL_BUILTIN_CSS_RULE_NAMES {
        if linter.css_rules.contains(rule_name)
            && linter.is_rule_enabled(rule_name)
            && let Some(rule) = css_rule_for_name(rule_name)
        {
            css_linter.add_rule(rule);
        }
    }

    for style in &descriptor.styles {
        // External styles (`<style src="...">`) have no inline content to lint.
        if style.src.is_some() {
            continue;
        }
        let source = style.content.as_ref();
        if source.trim().is_empty() {
            continue;
        }
        let css_result = profile!(
            "patina.css_rule.lint_style_block",
            css_linter.lint(source, style.loc.start)
        );
        result.error_count += css_result.error_count;
        result.warning_count += css_result.warning_count;
        result.diagnostics.extend(css_result.diagnostics);
    }
}

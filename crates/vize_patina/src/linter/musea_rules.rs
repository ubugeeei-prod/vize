//! Engine wiring for the built-in `musea/*` lint rules.
//!
//! Musea rules validate Art files (`*.art.vue`). They are intentionally
//! opt-in: the default Vue SFC presets must not start linting Art metadata
//! unless the host enables a `musea/*` rule by name.

use super::{LintResult, Linter};
use crate::rules::musea::MuseaLinter;
use vize_s0::profile;

pub(crate) const RULE_REQUIRE_TITLE: &str = "musea/require-title";
pub(crate) const RULE_REQUIRE_COMPONENT: &str = "musea/require-component";
pub(crate) const RULE_VALID_VARIANT: &str = "musea/valid-variant";
pub(crate) const RULE_UNIQUE_VARIANT_NAMES: &str = "musea/unique-variant-names";
pub(crate) const RULE_NO_EMPTY_VARIANT: &str = "musea/no-empty-variant";
pub(crate) const RULE_PREFER_DESIGN_TOKENS: &str = "musea/prefer-design-tokens";

const ALL_BUILTIN_MUSEA_RULE_NAMES: &[&str] = &[
    RULE_REQUIRE_TITLE,
    RULE_REQUIRE_COMPONENT,
    RULE_VALID_VARIANT,
    RULE_UNIQUE_VARIANT_NAMES,
    RULE_NO_EMPTY_VARIANT,
    RULE_PREFER_DESIGN_TOKENS,
];

#[inline]
pub(crate) const fn all_builtin_musea_rule_names() -> &'static [&'static str] {
    ALL_BUILTIN_MUSEA_RULE_NAMES
}

#[inline]
pub(crate) fn has_active_builtin_musea_rules(linter: &Linter) -> bool {
    linter
        .musea_rules
        .iter()
        .copied()
        .any(|rule_name| linter.is_rule_enabled(rule_name))
}

pub(crate) fn append_builtin_musea_diagnostics(
    linter: &Linter,
    source: &str,
    filename: &str,
    result: &mut LintResult,
) {
    if !filename.ends_with(".art.vue") || !has_active_builtin_musea_rules(linter) {
        return;
    }

    let mut musea_linter = MuseaLinter::new();
    musea_linter.check_require_title = rule_is_active(linter, RULE_REQUIRE_TITLE);
    musea_linter.check_require_component = rule_is_active(linter, RULE_REQUIRE_COMPONENT);
    musea_linter.check_valid_variant = rule_is_active(linter, RULE_VALID_VARIANT);
    musea_linter.check_unique_variant_names = rule_is_active(linter, RULE_UNIQUE_VARIANT_NAMES);
    musea_linter.check_no_empty_variant = rule_is_active(linter, RULE_NO_EMPTY_VARIANT);
    if rule_is_active(linter, RULE_PREFER_DESIGN_TOKENS)
        && let Some(config) = linter.musea_design_tokens.clone()
    {
        musea_linter = musea_linter.with_design_tokens(config);
    }

    let musea_result = profile!("patina.musea_rule.lint_art_file", musea_linter.lint(source));
    let diagnostics = musea_result
        .diagnostics
        .into_iter()
        .filter(|diagnostic| rule_is_active(linter, diagnostic.rule_name))
        .collect();
    super::severity::append_with_rule_overrides(result, diagnostics, &linter.severity_overrides);
    result
        .diagnostics
        .sort_unstable_by_key(|diagnostic| (diagnostic.start, diagnostic.end));
}

#[inline]
fn rule_is_active(linter: &Linter, rule_name: &'static str) -> bool {
    linter.musea_rules.contains(&rule_name) && linter.is_rule_enabled(rule_name)
}

#[cfg(test)]
mod tests {
    use super::ALL_BUILTIN_MUSEA_RULE_NAMES;
    use std::collections::BTreeSet;

    #[test]
    fn builtin_musea_rule_names_match_rule_metadata() {
        let routed: BTreeSet<_> = ALL_BUILTIN_MUSEA_RULE_NAMES.iter().copied().collect();
        let metadata: BTreeSet<_> = crate::rules::musea::builtin_musea_rules()
            .iter()
            .map(|meta| meta.name)
            .collect();

        assert_eq!(routed, metadata);
    }
}

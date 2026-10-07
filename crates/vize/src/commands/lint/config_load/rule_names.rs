//! Validate public configured IDs against the rules the lint engine actually supplies.

use crate::config::{LinterConfig, LinterConfigPlan};
use vize_l0::{FxHashSet, String, cstr};
use vize_patina::{RuleRegistry, builtin_css_rules, builtin_musea_rules, builtin_script_rules};

pub(super) fn validate(plan: &LinterConfigPlan) -> Result<(), String> {
    let root = configured_names(&plan.base);
    let entries = plan
        .entries
        .iter()
        .flat_map(|entry| {
            configured_names(&LinterConfig {
                rules: entry.rules.clone(),
                ..LinterConfig::default()
            })
        })
        .collect::<Vec<_>>();
    if root.is_empty() && entries.is_empty() {
        return Ok(());
    }

    let mut registry = RuleRegistry::with_all();
    registry.register_opt_in_rules();
    let mut nuxt = RuleRegistry::with_nuxt();
    nuxt.register_opt_in_rules();
    let mut known = registry
        .rule_names()
        .iter()
        .chain(nuxt.rule_names())
        .copied()
        .collect::<FxHashSet<_>>();
    known.extend(builtin_script_rules().into_iter().map(|rule| rule.name));
    known.extend(builtin_css_rules().into_iter().map(|rule| rule.name));
    known.extend(builtin_musea_rules().into_iter().map(|rule| rule.name));
    use vize_croquis_cf::{CrossFileDiagnostic, providers::vue_router::typing};
    known.extend([
        "cross-file",
        super::super::cross_file::CROSS_COMPONENT_RULE,
        super::super::cross_file::FALLTHROUGH_RULE,
        typing::UNKNOWN_ROUTE,
        typing::EXTRA_PARAM,
        typing::PARAM_TYPE,
        typing::MISSING_PARAM,
    ]);
    known.extend(
        CrossFileDiagnostic::CODES
            .into_iter()
            .map(|code| code.strip_prefix("vize:").unwrap_or(code)),
    );

    let mut problems = Vec::new();
    for (scope, names) in [("linter.rules", root), ("entries[].linter.rules", entries)] {
        let mut unknown = names
            .into_iter()
            .filter(|name| !known.contains(name.as_str()))
            .collect::<Vec<_>>();
        unknown.sort();
        unknown.dedup();
        if !unknown.is_empty() {
            problems.push(cstr!("  {scope}: {}", unknown.join(", ")));
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(cstr!(
            "Unknown lint rule IDs in configuration:\n{}",
            problems.join("\n")
        ))
    }
}

fn configured_names(config: &LinterConfig) -> Vec<String> {
    // These accessors exclude the config model's generated category/type-aware
    // markers while retaining every public ID, including disabled rules.
    let mut names = config.enabled_rules();
    names.extend(config.disabled_rules());
    names
}

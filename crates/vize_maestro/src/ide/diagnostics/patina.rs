//! Shared project-configured Patina constructor for diagnostics and actions.

use super::linter_options;
use tower_lsp::lsp_types::Url;
use vize_patina::LintPreset;

pub(in crate::ide) fn linter_for_uri(
    state: &crate::server::ServerState,
    uri: &Url,
    ecosystem_enabled: bool,
) -> Option<vize_patina::Linter> {
    let (linter_config, rule_options, features) = state.linter_settings_for_uri(uri)?;
    if !linter_config.enabled {
        return None;
    }
    let preset = linter_config.preset.as_deref();
    let preset = preset.and_then(LintPreset::parse).unwrap_or_default();
    let lint_options = linter_options::resolve_patina_options(uri, &linter_config, &rule_options);
    let mut linter = if ecosystem_enabled && linter_config.preset.is_none() {
        vize_patina::Linter::with_ecosystem()
    } else {
        vize_patina::Linter::with_preset(preset)
    }
    .with_additional_rules(lint_options.additional_rules)
    .with_disabled_rules(lint_options.disabled_rules)
    .with_category_severity_overrides(lint_options.category_severity_overrides)
    .with_rule_severity_overrides(lint_options.rule_severity_overrides)
    .with_restricted_globals(lint_options.restricted_globals)
    .with_restricted_members(lint_options.restricted_members)
    .with_musea_design_tokens(lint_options.musea_design_tokens);
    linter = linter.with_vue_version(features.vue_version);
    linter = linter_options::apply_rule_options(linter, &rule_options);

    #[cfg(not(target_arch = "wasm32"))]
    let linter = if linter_config.strict_reactivity_enabled() {
        linter.with_rule(Box::new(
            vize_patina::rules::type_aware::NoReactivityLoss::new(),
        ))
    } else {
        linter
    };
    Some(linter)
}

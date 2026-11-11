use tower_lsp::lsp_types::Url;
use vize_l0::{
    String,
    config::{ConfigLintRuleOptions, LintRuleSeverity, LinterConfig},
};
use vize_patina::{
    Severity,
    rules::musea::{MuseaLintResult, MuseaLinter, PreferDesignTokensConfig},
};

const MUSEA_PREFER_DESIGN_TOKENS: &str = "musea/prefer-design-tokens";
const SCRIPT_NO_RESTRICTED_MEMBERS: &str = "script/no-restricted-members";
const PINIA_PREFER_STORE_TO_REFS: &str = "ecosystem/pinia-prefer-store-to-refs";

pub(super) struct PatinaLintOptions {
    pub(super) additional_rules: Vec<String>,
    pub(super) disabled_rules: Vec<String>,
    pub(super) restricted_globals: Vec<(String, Option<String>)>,
    pub(super) restricted_members: Vec<(String, String, Option<String>)>,
    pub(super) musea_design_tokens: Vec<(String, String, String)>,
    pub(super) category_severity_overrides: Vec<(String, Severity)>,
    pub(super) rule_severity_overrides: Vec<(String, Severity)>,
}

pub(super) struct MuseaLintOptions {
    linter: MuseaLinter,
    rule_severity_overrides: Vec<(String, Severity)>,
}

impl MuseaLintOptions {
    pub(super) fn lint(&self, source: &str) -> MuseaLintResult {
        let mut result = self.linter.lint(source);
        if self.rule_severity_overrides.is_empty() {
            return result;
        }

        for diagnostic in &mut result.diagnostics {
            if let Some((_, severity)) = self
                .rule_severity_overrides
                .iter()
                .find(|(rule, _)| rule.as_str() == diagnostic.rule_name)
            {
                diagnostic.severity = *severity;
            }
        }
        result.error_count = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .count();
        result.warning_count = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Warning)
            .count();
        result
    }
}

pub(super) fn resolve_patina_options(
    uri: &Url,
    linter_config: &LinterConfig,
    rule_options: &ConfigLintRuleOptions,
) -> PatinaLintOptions {
    let mut additional_rules = linter_config.enabled_rules();
    let mut disabled_rules = linter_config.disabled_rules();
    if !pinia_is_available(uri) {
        disabled_rules.push(PINIA_PREFER_STORE_TO_REFS.into());
    }
    let restricted_members = rule_options.restricted_members();
    let musea_design_tokens = rule_options.musea_design_tokens();

    add_project_local_rule(
        &mut additional_rules,
        &disabled_rules,
        SCRIPT_NO_RESTRICTED_MEMBERS,
        !restricted_members.is_empty(),
    );
    add_project_local_rule(
        &mut additional_rules,
        &disabled_rules,
        MUSEA_PREFER_DESIGN_TOKENS,
        !musea_design_tokens.is_empty(),
    );

    PatinaLintOptions {
        additional_rules,
        disabled_rules,
        restricted_globals: rule_options.restricted_globals(),
        restricted_members,
        musea_design_tokens,
        category_severity_overrides: severity_overrides(
            linter_config.category_severity_overrides(),
        ),
        rule_severity_overrides: severity_overrides(linter_config.rule_severity_overrides()),
    }
}

// Match CLI package availability without changing the rule's store-name policy.
fn pinia_is_available(uri: &Url) -> bool {
    uri.to_file_path().is_err() || package_is_installed(uri, "pinia")
}

/// Whether `package` resolves from the document, as the CLI checks it.
pub(super) fn package_is_installed(uri: &Url, package: &str) -> bool {
    let Ok(file) = uri.to_file_path() else {
        return false;
    };
    file.parent().is_some_and(|directory| {
        directory.ancestors().any(|parent| {
            parent
                .join("node_modules")
                .join(package)
                .join("package.json")
                .is_file()
        })
    })
}

pub(super) fn musea_linter_for_uri(
    state: &crate::server::ServerState,
    uri: &Url,
) -> Option<MuseaLintOptions> {
    let (linter_config, rule_options, _) = state.linter_settings_for_uri(uri)?;
    if !linter_config.enabled {
        return None;
    }

    let rule_severity_overrides = severity_overrides(linter_config.rule_severity_overrides());
    let tokens = rule_options.musea_design_tokens();
    if tokens.is_empty()
        || linter_config
            .disabled_rules()
            .iter()
            .any(|rule| rule.as_str() == MUSEA_PREFER_DESIGN_TOKENS)
    {
        return Some(MuseaLintOptions {
            linter: MuseaLinter::new(),
            rule_severity_overrides,
        });
    }

    let mut config = PreferDesignTokensConfig::default();
    for (value, path, tier) in tokens {
        config.add_token(&value, &path, &tier);
    }
    Some(MuseaLintOptions {
        linter: MuseaLinter::new().with_design_tokens(config),
        rule_severity_overrides,
    })
}

/// The typed per-rule options of the CLI and native binding, plus the
/// components an installed vue-router or Nuxt registers globally.
pub(super) fn apply_rule_options(
    linter: vize_patina::Linter,
    options: &vize_l0::config::ConfigLintRuleOptions,
    vue_router_installed: bool,
    nuxt_installed: bool,
) -> vize_patina::Linter {
    let linter = linter.with_config_rule_options(options);
    if !vue_router_installed && !nuxt_installed {
        return linter;
    }
    let mut globals = options
        .component_registration_globals()
        .map(<[String]>::to_vec)
        .unwrap_or_default();
    if vue_router_installed {
        globals.extend(
            vize_patina::VUE_ROUTER_COMPONENTS
                .iter()
                .map(|name| String::from(*name)),
        );
    }
    if nuxt_installed {
        globals.extend(
            vize_patina::NUXT_BUILTIN_COMPONENTS
                .iter()
                .map(|name| String::from(*name)),
        );
    }
    linter.with_component_registration_options(
        globals,
        options.component_registration_ignore_patterns(),
    )
}

fn add_project_local_rule(
    additional_rules: &mut Vec<String>,
    disabled_rules: &[String],
    rule_name: &str,
    configured: bool,
) {
    if configured
        && !disabled_rules.iter().any(|rule| rule.as_str() == rule_name)
        && !additional_rules
            .iter()
            .any(|rule| rule.as_str() == rule_name)
    {
        additional_rules.push(rule_name.into());
    }
}

fn severity_overrides(entries: Vec<(String, LintRuleSeverity)>) -> Vec<(String, Severity)> {
    entries
        .into_iter()
        .filter_map(|(name, severity)| match severity {
            LintRuleSeverity::Off => None,
            LintRuleSeverity::Warn => Some((name, Severity::Warning)),
            LintRuleSeverity::Error => Some((name, Severity::Error)),
        })
        .collect()
}

pub(super) fn rule_docs_url(rule_name: &str) -> Option<Url> {
    if rule_name == "type/strict-boolean-expressions" {
        return Url::parse(
            "https://github.com/ubugeeei-prod/vize/blob/main/docs/content/rules/type-and-script.md",
        )
        .ok();
    }
    Url::parse(
        vize_l0::cstr!(
            "https://eslint.vuejs.org/rules/{}.html",
            rule_name.strip_prefix("vue/").unwrap_or(rule_name),
        )
        .as_str(),
    )
    .or_else(|_| Url::parse("https://eslint.vuejs.org/rules/"))
    .ok()
}

#[cfg(test)]
mod pinia_availability_tests;

#[cfg(test)]
mod rule_options_tests;

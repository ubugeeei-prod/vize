use tower_lsp::lsp_types::Url;
use vize_l0::{
    String,
    config::{ConfigLintRuleOptions, LintRuleSeverity, LinterConfig},
};
use vize_patina::{
    Severity,
    rules::musea::{MuseaLintResult, MuseaLinter, PreferDesignTokensConfig},
};

mod v_slot_style;
use self::v_slot_style::v_slot_style;

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

pub(super) fn apply_rule_options(
    mut linter: vize_patina::Linter,
    options: &vize_l0::config::ConfigLintRuleOptions,
    vue_router_installed: bool,
    nuxt_installed: bool,
) -> vize_patina::Linter {
    let configured = options.component_registration_globals();
    if configured.is_some() || vue_router_installed || nuxt_installed {
        let mut globals = configured.map(<[String]>::to_vec).unwrap_or_default();
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
        linter = linter.with_component_registration_globals(globals);
    }
    if let Some(names) = options.palpable_content_directives() {
        linter = linter.with_palpable_content_directives(names.to_vec());
    }
    if let Some(options) = options.strict_boolean_expressions() {
        linter = linter.with_strict_boolean_expressions_options(options);
    }
    if let Some(mode) = options.define_props_destructuring() {
        linter = linter.with_define_props_destructuring(mode);
    }
    if let Some(options) = options.define_macros_order() {
        linter = linter.with_define_macros_order(options);
    }
    if let Some(options) = options.component_name_in_template_casing_options() {
        linter = linter.with_component_name_in_template_casing_policy(
            component_casing(options.casing),
            options.registered_components_only,
            options.globals.clone(),
        );
    }
    if let Some(casing) = options.custom_event_name_casing() {
        linter = linter.with_custom_event_name_casing(event_name_casing(casing));
    }
    if let Some(options) = options.no_mutating_props() {
        linter = linter.with_no_mutating_props_options(no_mutating_props_options(options));
    }
    if let Some(options) = options.sfc_element_order() {
        linter = linter.with_sfc_element_order_options(sfc_element_order_options(options));
    }
    if let Some(options) = options.html_self_closing() {
        linter = linter.with_html_self_closing_options(html_self_closing_options(options));
    }
    if let Some(options) = options.html_quotes() {
        linter = linter.with_html_quotes(html_quotes(options));
    }
    if let Some(style) = options.mustache_interpolation_spacing() {
        linter = linter.with_mustache_interpolation_spacing(mustache_interpolation_spacing(style));
    }
    if let Some(style) = options.v_on_event_hyphenation() {
        linter = linter.with_v_on_event_hyphenation(v_on_event_hyphenation_style(style));
    }
    if let Some(style) = options.v_on_style() {
        linter = linter.with_v_on_style(v_on_style(style));
    }
    if let Some(style) = options.v_slot_style() {
        linter = linter.with_v_slot_style(v_slot_style(style));
    }
    if let Some(style) = options.v_bind_style() {
        linter = linter.with_v_bind_style(v_bind_style(style));
    }
    if let Some(style) = options.attribute_hyphenation() {
        linter = linter.with_attribute_hyphenation(attribute_hyphenation_style(style));
    }
    if let Some(modifiers) = options.valid_v_on_modifiers() {
        linter = linter.with_valid_v_on_modifiers(modifiers.to_vec());
    }
    if let Some(allow) = options.valid_v_slot_allow_modifiers() {
        linter = linter.with_valid_v_slot_allow_modifiers(allow);
    }
    linter
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

fn component_casing(
    casing: vize_l0::config::TemplateComponentNameCasing,
) -> vize_patina::rules::ComponentCasing {
    match casing {
        vize_l0::config::TemplateComponentNameCasing::PascalCase => {
            vize_patina::rules::ComponentCasing::PascalCase
        }
        vize_l0::config::TemplateComponentNameCasing::KebabCase => {
            vize_patina::rules::ComponentCasing::KebabCase
        }
    }
}

fn event_name_casing(
    casing: vize_l0::config::CustomEventNameCasing,
) -> vize_patina::rules::script::EventNameCasing {
    match casing {
        vize_l0::config::CustomEventNameCasing::CamelCase => {
            vize_patina::rules::script::EventNameCasing::CamelCase
        }
        vize_l0::config::CustomEventNameCasing::KebabCase => {
            vize_patina::rules::script::EventNameCasing::KebabCase
        }
    }
}

fn html_quotes(options: vize_l0::config::HtmlQuotesConfig) -> vize_patina::rules::vue::HtmlQuotes {
    vize_patina::rules::vue::HtmlQuotes {
        style: match options.style() {
            vize_l0::config::HtmlQuoteStyle::Double => {
                vize_patina::rules::vue::HtmlQuotesOption::Double
            }
            vize_l0::config::HtmlQuoteStyle::Single => {
                vize_patina::rules::vue::HtmlQuotesOption::Single
            }
        },
        avoid_escape: options.avoid_escape(),
    }
}

fn mustache_interpolation_spacing(
    style: vize_l0::config::MustacheInterpolationSpacingStyle,
) -> vize_patina::rules::vue::MustacheInterpolationSpacingStyle {
    match style {
        vize_l0::config::MustacheInterpolationSpacingStyle::Always => {
            vize_patina::rules::vue::MustacheInterpolationSpacingStyle::Always
        }
        vize_l0::config::MustacheInterpolationSpacingStyle::Never => {
            vize_patina::rules::vue::MustacheInterpolationSpacingStyle::Never
        }
    }
}

fn no_mutating_props_options(
    options: vize_l0::config::NoMutatingPropsOptions,
) -> vize_patina::rules::NoMutatingPropsOptions {
    vize_patina::rules::NoMutatingPropsOptions {
        shallow_only: options.shallow_only,
    }
}

fn sfc_element_order_options(
    options: vize_l0::config::SfcElementOrderOptions,
) -> vize_patina::rules::SfcElementOrderOptions {
    vize_patina::rules::SfcElementOrderOptions {
        order: options
            .order
            .into_iter()
            .map(|group| vize_patina::rules::SfcElementOrderGroup::new(group.selectors()))
            .collect(),
    }
}

fn html_self_closing_options(
    options: vize_l0::config::HtmlSelfClosingOptions,
) -> vize_patina::rules::HtmlSelfClosingOptions {
    vize_patina::rules::HtmlSelfClosingOptions {
        html: vize_patina::rules::HtmlSelfClosingHtmlOptions {
            void: html_self_closing_style(options.html.void_elements),
            normal: html_self_closing_style(options.html.normal),
            component: html_self_closing_style(options.html.component),
        },
        svg: html_self_closing_style(options.svg),
        math: html_self_closing_style(options.math),
    }
}

fn html_self_closing_style(
    style: vize_l0::config::HtmlSelfClosingStyle,
) -> vize_patina::rules::HtmlSelfClosingStyle {
    match style {
        vize_l0::config::HtmlSelfClosingStyle::Always => {
            vize_patina::rules::HtmlSelfClosingStyle::Always
        }
        vize_l0::config::HtmlSelfClosingStyle::Never => {
            vize_patina::rules::HtmlSelfClosingStyle::Never
        }
        vize_l0::config::HtmlSelfClosingStyle::Any => vize_patina::rules::HtmlSelfClosingStyle::Any,
    }
}

fn v_on_event_hyphenation_style(
    style: vize_l0::config::HyphenationStyle,
) -> vize_patina::rules::VOnEventHyphenationStyle {
    match style {
        vize_l0::config::HyphenationStyle::Always => {
            vize_patina::rules::VOnEventHyphenationStyle::Always
        }
        vize_l0::config::HyphenationStyle::Never => {
            vize_patina::rules::VOnEventHyphenationStyle::Never
        }
    }
}

fn v_on_style(style: vize_l0::config::VOnStyle) -> vize_patina::rules::vue::VOnStyleOption {
    match style {
        vize_l0::config::VOnStyle::Shorthand => vize_patina::rules::vue::VOnStyleOption::Shorthand,
        vize_l0::config::VOnStyle::Longform => vize_patina::rules::vue::VOnStyleOption::Longform,
    }
}

fn v_bind_style(style: vize_l0::config::VBindStyle) -> vize_patina::rules::vue::VBindStyleOption {
    match style {
        vize_l0::config::VBindStyle::Shorthand => {
            vize_patina::rules::vue::VBindStyleOption::Shorthand
        }
        vize_l0::config::VBindStyle::Longform => {
            vize_patina::rules::vue::VBindStyleOption::Longform
        }
    }
}

fn attribute_hyphenation_style(
    style: vize_l0::config::HyphenationStyle,
) -> vize_patina::rules::HyphenationStyle {
    match style {
        vize_l0::config::HyphenationStyle::Always => vize_patina::rules::HyphenationStyle::Always,
        vize_l0::config::HyphenationStyle::Never => vize_patina::rules::HyphenationStyle::Never,
    }
}

/// Point the new native type rule at its actual Vize reference.
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
mod valid_v_on_options_tests {
    use super::apply_rule_options;
    use vize_patina::{
        HelpLevel, LintDiagnostic, LintResult, Linter, Locale, RuleRegistry, rules::vue::ValidVOn,
    };

    #[test]
    fn lsp_custom_modifiers_reach_the_configured_rule() {
        let options =
            serde_json::from_str(r#"{"vue/valid-v-on":{"modifiers":["custom"]}}"#).unwrap();
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVOn));
        let linter = apply_rule_options(Linter::with_registry(registry), &options, false, false)
            .with_locale(Locale::En)
            .with_help_level(HelpLevel::Full);
        let result = linter.lint_template(
            r#"<div @click.custom="handler" @click.unknown="handler" />"#,
            "test.vue",
        );
        if let Some(directory) = std::env::var_os("VIZE_NATIVE_STRICT_CAPTURE_DIR") {
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(
                std::path::Path::new(&directory).join("lsp-custom-modifier.txt"),
                vize_l0::cstr!("{result:#?}").as_bytes(),
            )
            .unwrap();
        }
        let expected = LintResult {
            filename: "test.vue".into(),
            diagnostics: vec![LintDiagnostic::error(
                "vue/valid-v-on",
                "v-on does not support the modifier 'unknown'",
                36,
                43,
            )
            .with_help(
                "Use a built-in event modifier, keyboard alias, or a configured custom modifier.",
            )],
            error_count: 1,
            warning_count: 0,
        };
        assert_eq!(
            vize_l0::cstr!("{result:#?}"),
            vize_l0::cstr!("{expected:#?}"),
            "complete LSP-configured modifier diagnostic, including receiver and fix fields"
        );
        let empty = apply_rule_options(
            Linter::with_registry(RuleRegistry::new()),
            &options,
            false,
            false,
        );
        assert!(!empty.registry().has_rule("vue/valid-v-on"));
    }
}

#[cfg(test)]
mod define_macros_order_options_tests {
    use super::apply_rule_options;
    use vize_patina::Linter;

    #[test]
    fn lsp_custom_macro_order_reaches_the_configured_rule() {
        let options = serde_json::from_str(
            r#"{"script/define-macros-order":{"order":["defineOptions","defineProps","defineEmits"]}}"#,
        )
        .unwrap();
        let linter = apply_rule_options(
            Linter::new().with_enabled_rules(Some(vec!["script/define-macros-order".into()])),
            &options,
            false,
            false,
        );
        let result = linter.lint_sfc(
            r#"<script setup>
defineOptions({ inheritAttrs: false })
const props = defineProps()
const emit = defineEmits()
</script>"#,
            "App.vue",
        );
        assert!(result.diagnostics.is_empty(), "{result:#?}");
    }
}

#[cfg(test)]
mod mustache_interpolation_spacing_options_tests {
    use super::apply_rule_options;
    use vize_patina::{Linter, RuleRegistry, rules::vue::MustacheInterpolationSpacing};

    #[test]
    fn lsp_mustache_spacing_reaches_the_configured_rule() {
        let options =
            serde_json::from_str(r#"{"vue/mustache-interpolation-spacing":"never"}"#).unwrap();
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(MustacheInterpolationSpacing::default()));
        let result = apply_rule_options(Linter::with_registry(registry), &options, false, false)
            .lint_template("<div>{{ value }}</div>", "App.vue");
        assert_eq!(result.warning_count, 2, "{result:#?}");
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| { diagnostic.rule_name == "vue/mustache-interpolation-spacing" })
        );
        let empty = apply_rule_options(
            Linter::with_registry(RuleRegistry::new()),
            &options,
            false,
            false,
        );
        assert!(
            !empty
                .registry()
                .has_rule("vue/mustache-interpolation-spacing")
        );
    }
}

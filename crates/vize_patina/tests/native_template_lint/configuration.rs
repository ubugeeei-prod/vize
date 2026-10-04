use super::support::*;
use vize_l0::config::VueVersion;
use vize_patina::{
    HelpLevel, LintPreset, Linter, Locale, Rule, RuleCategory, RuleMeta, RuleRegistry, Severity,
    context::LintContext, native::template::NativeTemplateLintRefusal as Refusal,
};
use vize_relief::RootNode;

mod instances;

static NAMESAKE_META: RuleMeta = RuleMeta {
    name: RULE,
    description: "A custom namesake has no native capability",
    category: RuleCategory::StronglyRecommended,
    fixable: false,
    default_severity: Severity::Warning,
};
struct Namesake;
impl Rule for Namesake {
    fn meta(&self) -> &'static RuleMeta {
        &NAMESAKE_META
    }
    fn run_on_template<'a>(&self, context: &mut LintContext<'a>, root: &RootNode<'a>) {
        context.warn("actual custom namesake", &root.loc);
    }
}

#[test]
fn custom_same_name_rule_cannot_impersonate_a_native_registered_instance() {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(Namesake));
    let linter = Linter::with_registry(registry);
    assert_eq!(
        linter
            .lint_native_template(SOURCE, "myComponent.vue")
            .unwrap_err(),
        Refusal::UnprovidedRule { rule: RULE.into() }
    );
    assert_eq!(
        complete(&linter.lint_template(SOURCE, "myComponent.vue")),
        expected(
            "myComponent.vue",
            vec![serde_json::json!({
                "rule_name": RULE, "severity": "warning", "message": "actual custom namesake",
                "start": 0, "end": 0, "help": null, "labels": [], "fix": null,
            })]
        )
    );
}

#[test]
fn unknown_and_external_requested_names_are_refused_instead_of_omitted() {
    for name in [
        "unknown/missing",
        "script/no-next-tick",
        "css/no-important",
        "musea/no-duplicate-title",
        "type/strict-boolean-expressions",
    ] {
        let linter = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![RULE.into(), name.into()]));
        assert_eq!(
            linter
                .lint_native_template(SOURCE, "myComponent.vue")
                .unwrap_err(),
            Refusal::UnprovidedRule { rule: name.into() }
        );
    }
}

#[test]
fn disabled_precedence_retains_supported_callback_and_original_clean_output() {
    for locale in LOCALES {
        let linter = configured(locale, HelpLevel::Full)
            .with_enabled_rules(Some(vec![RULE.into(), "unknown/missing".into()]))
            .with_disabled_rules(vec!["unknown/missing".into()]);
        assert_pair(
            &linter,
            SOURCE,
            "myComponent.vue",
            expected(
                "myComponent.vue",
                vec![warning(
                    locale,
                    HelpLevel::Full,
                    "myComponent",
                    Severity::Warning,
                )],
            ),
        );
        let disabled = configured(locale, HelpLevel::Full).with_disabled_rules(vec![RULE.into()]);
        assert_pair(
            &disabled,
            SOURCE,
            "myComponent.vue",
            expected("myComponent.vue", vec![]),
        );
    }
}

#[test]
fn enabled_empty_and_incremental_empty_registry_have_complete_clean_results() {
    for linter in [
        Linter::with_preset(LintPreset::Incremental),
        Linter::new().with_enabled_rules(Some(vec![])),
    ] {
        assert_pair(
            &linter,
            SOURCE,
            "myComponent.vue",
            expected("myComponent.vue", vec![]),
        );
    }
}

#[test]
fn default_and_enabled_unprovided_existing_header_rules_refuse() {
    assert!(matches!(
        Linter::new().lint_native_template(SOURCE, "myComponent.vue"),
        Err(Refusal::UnprovidedRule { .. })
    ));
    for name in [
        "a11y/img-alt",
        "vue/no-v-html",
        "vue/no-textarea-mustache",
        "html/deprecated-attr",
    ] {
        let linter = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![name.into()]));
        assert_eq!(
            linter
                .lint_native_template(SOURCE, "myComponent.vue")
                .unwrap_err(),
            Refusal::UnprovidedRule { rule: name.into() }
        );
    }
}

#[test]
fn every_legacy_dialect_and_enabled_vapor_request_remains_typed_refused() {
    for requested in VueVersion::ALL
        .into_iter()
        .filter(|version| *version != VueVersion::V3)
    {
        assert_eq!(
            configured(Locale::En, HelpLevel::Full)
                .with_vue_version(Some(requested))
                .lint_native_template(SOURCE, "myComponent.vue")
                .unwrap_err(),
            Refusal::UnsupportedVueVersion { requested }
        );
    }
    assert_eq!(
        configured(Locale::En, HelpLevel::Full)
            .with_vapor_mode(Some(true))
            .lint_native_template(SOURCE, "myComponent.vue")
            .unwrap_err(),
        Refusal::UnsupportedVaporMode { requested: true }
    );
}

#[test]
fn all_locales_help_levels_and_configured_severities_keep_full_product_metadata() {
    for locale in LOCALES {
        for help in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
            for severity in [Severity::Warning, Severity::Error] {
                let linter = configured(locale, help)
                    .with_rule_severity_overrides(vec![(RULE.into(), severity)]);
                assert_pair(
                    &linter,
                    SOURCE,
                    "myComponent.vue",
                    expected(
                        "myComponent.vue",
                        vec![warning(locale, help, "myComponent", severity)],
                    ),
                );
            }
        }
    }
}

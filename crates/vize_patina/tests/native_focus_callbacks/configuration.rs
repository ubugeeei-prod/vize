use super::support::*;
use vize_patina::{
    HelpLevel, LintContext, Linter, Rule, RuleMeta, RuleRegistry, Severity,
    native::template::NativeTemplateLintRefusal as Refusal,
    rules::{
        a11y::{NoAccessKey, NoAutofocus},
        vue::ComponentDefinitionNameCasing,
    },
};
use vize_relief::ElementNode;

#[test]
fn disabled_precedence_and_per_rule_severity_override_keep_full_metadata() {
    let source = "<div :autofocus='opaque' :accesskey='opaque'></div>";
    for locale in LOCALES {
        for reverse in [false, true] {
            let configured = linter(reverse, locale, HelpLevel::Full)
                .with_enabled_rules(Some(vec![AUTO.into(), KEY.into()]))
                .with_disabled_rules(vec![AUTO.into()])
                .with_rule_severity_overrides(vec![
                    (AUTO.into(), Severity::Error),
                    (KEY.into(), Severity::Error),
                ]);
            pair(
                &configured,
                source,
                expected(vec![finding(
                    KEY,
                    locale,
                    HelpLevel::Full,
                    span(source, ":accesskey='opaque'"),
                    Severity::Error,
                )]),
            );
        }
        let configured = linter(false, locale, HelpLevel::None)
            .with_rule_severity_overrides(vec![(AUTO.into(), Severity::Error)]);
        pair(
            &configured,
            source,
            expected(vec![
                finding(
                    AUTO,
                    locale,
                    HelpLevel::None,
                    span(source, ":autofocus='opaque'"),
                    Severity::Error,
                ),
                finding(
                    KEY,
                    locale,
                    HelpLevel::None,
                    span(source, ":accesskey='opaque'"),
                    Severity::Warning,
                ),
            ]),
        );
    }
}

#[test]
fn empty_or_disabled_callback_sets_keep_static_only_not_inferred_binding_absence() {
    for configured in [
        linter(false, vize_patina::Locale::En, HelpLevel::Full).with_enabled_rules(Some(vec![])),
        linter(false, vize_patina::Locale::En, HelpLevel::Full)
            .with_disabled_rules(vec![AUTO.into(), KEY.into()]),
    ] {
        pair(
            &configured,
            "<div autofocus accesskey='h'/>",
            expected(vec![]),
        );
        let source = "<div :autofocus='opaque'></div>";
        original(&configured, source, expected(vec![]));
        assert_eq!(
            configured.lint_native_template(source, FILE).unwrap_err(),
            Refusal::UnsupportedAttribute {
                span: span(source, ":autofocus='opaque'")
            }
        );
    }
}

#[test]
fn actual_static_only_filename_instance_intersection_preserves_root_and_binding_refusal() {
    for reverse in [false, true] {
        let mut registry = RuleRegistry::new();
        if reverse {
            registry.register(Box::new(NoAutofocus));
            registry.register(Box::new(ComponentDefinitionNameCasing));
        } else {
            registry.register(Box::new(ComponentDefinitionNameCasing));
            registry.register(Box::new(NoAutofocus));
        }
        let configured = Linter::with_registry(registry);
        let source = "<div :autofocus='opaque'></div>";
        original(
            &configured,
            source,
            expected(vec![finding(
                AUTO,
                vize_patina::Locale::En,
                HelpLevel::Full,
                span(source, ":autofocus='opaque'"),
                Severity::Warning,
            )]),
        );
        assert_eq!(
            configured.lint_native_template(source, FILE).unwrap_err(),
            Refusal::UnsupportedAttribute {
                span: span(source, ":autofocus='opaque'")
            }
        );
        let source = "<div autofocus/>";
        pair(
            &configured,
            source,
            expected(vec![finding(
                AUTO,
                vize_patina::Locale::En,
                HelpLevel::Full,
                span(source, "autofocus"),
                Severity::Warning,
            )]),
        );
    }
}

struct Namesake(bool);
impl Rule for Namesake {
    fn meta(&self) -> &'static RuleMeta {
        if self.0 {
            Rule::meta(&NoAutofocus)
        } else {
            Rule::meta(&NoAccessKey)
        }
    }
    fn enter_element<'a>(&self, context: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if self.0 {
            Rule::enter_element(&NoAutofocus, context, element);
        } else {
            Rule::enter_element(&NoAccessKey, context, element);
        }
    }
}

#[test]
fn actual_namesake_overrides_cannot_borrow_builtin_capability_despite_full_original_warnings() {
    let source = "<div autofocus accesskey='h'/>";
    for (auto, rule, authored) in [(true, AUTO, "autofocus"), (false, KEY, "accesskey='h'")] {
        for locale in LOCALES {
            let mut registry = RuleRegistry::new();
            registry.register(Box::new(Namesake(auto)));
            let configured = Linter::with_registry(registry).with_locale(locale);
            original(
                &configured,
                source,
                expected(vec![finding(
                    rule,
                    locale,
                    HelpLevel::Full,
                    span(source, authored),
                    Severity::Warning,
                )]),
            );
            assert_eq!(
                configured.lint_native_template(source, FILE).unwrap_err(),
                Refusal::UnprovidedRule { rule: rule.into() }
            );
        }
    }
}

#[test]
fn unknown_enabled_names_are_refused_without_omitting_original_provided_diagnostics() {
    let source = "<div autofocus/>";
    for locale in LOCALES {
        let configured = linter(false, locale, HelpLevel::Full)
            .with_enabled_rules(Some(vec![AUTO.into(), "external/missing".into()]));
        original(
            &configured,
            source,
            expected(vec![finding(
                AUTO,
                locale,
                HelpLevel::Full,
                span(source, "autofocus"),
                Severity::Warning,
            )]),
        );
        assert_eq!(
            configured.lint_native_template(source, FILE).unwrap_err(),
            Refusal::UnprovidedRule {
                rule: "external/missing".into()
            }
        );
    }
}

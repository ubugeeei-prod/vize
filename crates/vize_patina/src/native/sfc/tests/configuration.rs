use super::super::NativeSfcLintRefusal as Refusal;
use super::support::*;
use crate::Locale;
use crate::{HelpLevel, Linter, RuleRegistry, Severity};
use vize_l0::config::VueVersion;

#[test]
fn actual_builtin_and_namesake_unprovided_instance_refuse_before_source_parsing() {
    let builtin = Linter::new().with_enabled_rules(Some(vec![RULE.into()]));
    assert_eq!(
        builtin.lint_native_sfc("<broken", FILE).unwrap_err(),
        Refusal::UnprovidedRule { rule: RULE.into() }
    );
    let events = log();
    let mut unprovided = audit(&FIRST, "namesake", Locale::En, HelpLevel::Full, &events);
    unprovided.provided = false;
    let configured = configured(vec![unprovided], Locale::En, HelpLevel::Full);
    assert_eq!(
        configured.lint_native_sfc(SOURCE, FILE).unwrap_err(),
        Refusal::UnprovidedRule { rule: RULE.into() }
    );
    assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
    assert_eq!(
        complete(&configured.lint_sfc(SOURCE, FILE)),
        expected(
            FILE,
            vec![diagnostic(
                RULE,
                Locale::En,
                HelpLevel::Full,
                "namesake",
                script_span(SOURCE).start,
                Severity::Warning
            )]
        )
    );
}

#[test]
fn disabled_precedence_and_unknown_requested_names_are_not_clean_rule_omission() {
    let configured = Linter::with_registry(RuleRegistry::new())
        .with_enabled_rules(Some(vec!["external/z".into(), "external/a".into()]));
    assert_eq!(
        configured.lint_native_sfc(SOURCE, FILE).unwrap_err(),
        Refusal::UnprovidedRule {
            rule: "external/a".into()
        }
    );
    let configured = configured.with_disabled_rules(vec!["external/a".into()]);
    assert_eq!(
        configured.lint_native_sfc(SOURCE, FILE).unwrap_err(),
        Refusal::UnprovidedRule {
            rule: "external/z".into()
        }
    );
    let configured = configured.with_disabled_rules(vec!["external/a".into(), "external/z".into()]);
    assert_eq!(
        complete(&configured.lint_native_sfc(SOURCE, FILE).unwrap()),
        empty(FILE)
    );
    assert_eq!(complete(&configured.lint_sfc(SOURCE, FILE)), empty(FILE));
}

#[test]
fn enabled_other_rule_families_refuse_without_native_callback_execution() {
    for name in [
        "vue/component-definition-name-casing",
        "a11y/img-alt",
        "css/no-important",
        "musea/require-title",
    ] {
        let events = log();
        let configured =
            single(&events, Locale::En).with_enabled_rules(Some(vec![RULE.into(), name.into()]));
        assert_eq!(
            configured.lint_native_sfc(SOURCE, FILE).unwrap_err(),
            Refusal::UnprovidedRule { rule: name.into() }
        );
        assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
    }
    let events = log();
    let configured = single(&events, Locale::En).with_type_aware_lint(true);
    assert_eq!(
        configured.lint_native_sfc(SOURCE, FILE).unwrap_err(),
        Refusal::UnsupportedTypeAwareMode
    );
    assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
}

#[test]
fn wrong_native_version_or_vapor_request_is_preserved_and_refused_before_parse() {
    let events = log();
    let configured = single(&events, Locale::En).with_vue_version(Some(VueVersion::V2));
    assert_eq!(
        configured.lint_native_sfc("<broken", FILE).unwrap_err(),
        Refusal::UnsupportedVueVersion {
            requested: VueVersion::V2
        }
    );
    let configured = single(&events, Locale::En).with_vapor_mode(Some(true));
    assert_eq!(
        configured.lint_native_sfc("<broken", FILE).unwrap_err(),
        Refusal::UnsupportedVaporMode { requested: true }
    );
    assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
}

#[test]
fn mismatched_override_metadata_and_setup_disabled_instance_cannot_borrow_catalog_capability() {
    let events = log();
    let mut configured = single(&events, Locale::En);
    configured.script_rule_overrides.insert(
        RULE,
        Box::new(audit(
            &SECOND,
            "foreign",
            Locale::En,
            HelpLevel::Full,
            &events,
        )),
    );
    assert_eq!(
        configured.lint_native_sfc(SOURCE, FILE).unwrap_err(),
        Refusal::UnprovidedRule { rule: RULE.into() }
    );
    let mut disabled = audit(
        &FIRST,
        "disabled-setup",
        Locale::En,
        HelpLevel::Full,
        &events,
    );
    disabled.runs_setup = false;
    let configured = configured_fn(disabled);
    assert_eq!(
        configured.lint_native_sfc(SOURCE, FILE).unwrap_err(),
        Refusal::UnprovidedRule { rule: RULE.into() }
    );
    assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
}
fn configured_fn(rule: Audit) -> Linter {
    configured(vec![rule], Locale::En, HelpLevel::Full)
}

#[test]
fn original_constructor_option_none_and_disabled_configured_instance_remain_exact() {
    for base in [
        Linter::new(),
        Linter::with_preset(crate::LintPreset::Incremental),
        Linter::with_registry(RuleRegistry::new()),
        Linter::with_ecosystem(),
    ] {
        let events = log();
        let mut configured = base.with_enabled_rules(Some(vec![RULE.into()]));
        configured.script_rule_overrides.insert(
            RULE,
            Box::new(audit(
                &FIRST,
                "actual",
                Locale::En,
                HelpLevel::Full,
                &events,
            )),
        );
        assert_eq!(
            complete(&configured.lint_native_sfc(SOURCE, FILE).unwrap()),
            expected(
                FILE,
                vec![diagnostic(
                    RULE,
                    Locale::En,
                    HelpLevel::Full,
                    "actual",
                    script_span(SOURCE).start,
                    Severity::Warning
                )]
            )
        );
        assert_eq!(
            *events.lock().unwrap(),
            [expected_event(
                SOURCE,
                FILE,
                &FIRST,
                "actual",
                Locale::En,
                HelpLevel::Full,
                None,
                None
            )]
        );
        let configured = configured.with_disabled_rules(vec![RULE.into()]);
        assert_eq!(
            complete(&configured.lint_native_sfc(SOURCE, FILE).unwrap()),
            empty(FILE)
        );
        assert_eq!(
            *events.lock().unwrap(),
            [expected_event(
                SOURCE,
                FILE,
                &FIRST,
                "actual",
                Locale::En,
                HelpLevel::Full,
                None,
                None
            )]
        );
    }
}

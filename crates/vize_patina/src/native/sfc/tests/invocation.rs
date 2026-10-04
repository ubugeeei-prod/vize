use super::super::NativeSfcLintRefusal as Refusal;
use super::support::*;
use crate::{HelpLevel, Linter, Severity, rules::script::ScriptRuleMeta};

static NUXT: ScriptRuleMeta = ScriptRuleMeta {
    name: "nuxt/nuxt-config-keys-order",
    description: "Actual catalog filename policy control",
    default_severity: Severity::Warning,
};
static GET: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-get-current-instance",
    description: "Actual catalog implicit activation control",
    default_severity: Severity::Warning,
};

#[test]
fn offered_nuxt_filename_policy_refuses_original_skipped_and_genuinely_selected_files() {
    for locale in LOCALES {
        for file in [
            FILE,
            "/apps/web/nuxt.config.ts",
            r"apps\web\.config\nuxt.mts",
        ] {
            let events = log();
            let configured = configured(
                vec![audit(&NUXT, "policy", locale, HelpLevel::Full, &events)],
                locale,
                HelpLevel::Full,
            );
            assert_eq!(
                configured.lint_native_sfc(SOURCE, file).unwrap_err(),
                Refusal::UnprovidedInvocationPolicy {
                    rule: NUXT.name.into()
                }
            );
            assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
            let full = if file == FILE {
                empty(file)
            } else {
                expected(
                    file,
                    vec![diagnostic(
                        NUXT.name,
                        locale,
                        HelpLevel::Full,
                        "policy",
                        script_span(SOURCE).start,
                        Severity::Warning,
                    )],
                )
            };
            assert_eq!(complete(&configured.lint_sfc(SOURCE, file)), full);
            // Refuse a provided policy family before source parse even at its
            // legitimate original filename; it is not silently skipped clean.
            assert_eq!(
                configured.lint_native_sfc("<broken", file).unwrap_err(),
                Refusal::UnprovidedInvocationPolicy {
                    rule: NUXT.name.into()
                }
            );
        }
    }
}

#[test]
fn offered_get_instance_refuses_both_implicit_skip_and_explicit_original_dispatch() {
    for locale in LOCALES {
        for explicit in [false, true] {
            let events = log();
            let configured = configured(
                vec![audit(&GET, "policy", locale, HelpLevel::Full, &events)],
                locale,
                HelpLevel::Full,
            );
            let configured = if explicit {
                configured
            } else {
                // Authentic public setters retain the installed actual catalog
                // but reset explicit-get. Disable all other actual families.
                let configured = configured.with_enabled_rules(None);
                let disabled = configured
                    .registry
                    .rule_names()
                    .iter()
                    .copied()
                    .chain(configured.script_rules.iter().copied())
                    .chain(configured.css_rules.iter().copied())
                    .chain(configured.musea_rules.iter().copied())
                    .filter(|&name| name != GET.name)
                    .map(Into::into)
                    .collect();
                configured.with_disabled_rules(disabled)
            };
            assert_eq!(configured.explicit_no_get_current_instance, explicit);
            assert_eq!(
                configured.lint_native_sfc(SOURCE, FILE).unwrap_err(),
                Refusal::UnprovidedInvocationPolicy {
                    rule: GET.name.into()
                }
            );
            assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
            let full = if explicit {
                expected(
                    FILE,
                    vec![diagnostic(
                        GET.name,
                        locale,
                        HelpLevel::Full,
                        "policy",
                        script_span(SOURCE).start,
                        Severity::Warning,
                    )],
                )
            } else {
                empty(FILE)
            };
            assert_eq!(complete(&configured.lint_sfc(SOURCE, FILE)), full);
        }
    }
}

#[test]
fn disabled_offered_policy_instances_do_not_block_a_valid_genuine_setup_callback() {
    for meta in [&GET, &NUXT] {
        for locale in LOCALES {
            let events = log();
            let configured = configured(
                vec![
                    audit(meta, "policy", locale, HelpLevel::Full, &events),
                    audit(&FIRST, "actual", locale, HelpLevel::Full, &events),
                ],
                locale,
                HelpLevel::Full,
            )
            .with_disabled_rules(vec![meta.name.into()]);
            let full = expected(
                FILE,
                vec![diagnostic(
                    RULE,
                    locale,
                    HelpLevel::Full,
                    "actual",
                    script_span(SOURCE).start,
                    Severity::Warning,
                )],
            );
            assert_eq!(
                complete(&configured.lint_native_sfc(SOURCE, FILE).unwrap()),
                full
            );
            assert_eq!(complete(&configured.lint_sfc(SOURCE, FILE)), full);
            assert_eq!(
                *events.lock().unwrap(),
                [expected_event(
                    SOURCE,
                    FILE,
                    &FIRST,
                    "actual",
                    locale,
                    HelpLevel::Full,
                    None,
                    None
                )]
            );
        }
    }
}

#[test]
fn unprovided_actual_policy_builtins_still_refuse_the_missing_instance_capability_first() {
    for meta in [&GET, &NUXT] {
        let configured = Linter::new().with_enabled_rules(Some(vec![meta.name.into()]));
        assert_eq!(
            configured.lint_native_sfc("<broken", FILE).unwrap_err(),
            Refusal::UnprovidedRule {
                rule: meta.name.into()
            }
        );
    }
}

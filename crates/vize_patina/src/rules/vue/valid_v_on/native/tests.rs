//! Real legacy/native source packets; old count-only inputs are not goldens.
mod ordering;
mod refusal;
mod support;

use crate::{HelpLevel, Locale, Severity};
use support::{
    ORIGINAL, Order, RULE, UNSUPPORTED_MODIFIER_ORIGINAL, case, configured, pair, pair_with,
};

#[test]
fn original_absent_handler_has_the_complete_whole_attribute_error_without_a_fix() {
    assert_eq!(ORIGINAL, "<div @click.native />");
    let observed = pair(&case(
        "old-original-native-modifier",
        ORIGINAL,
        Order::ValidOnly,
        1,
        0,
    ));
    let diagnostic = observed.initial.diagnostics.first().unwrap();
    assert_eq!(diagnostic.rule_name, RULE);
    assert_eq!((diagnostic.start, diagnostic.end), (5, 18));
    assert_eq!(diagnostic.severity, Severity::Error);
    assert!(diagnostic.fix.is_none());
    assert!(observed.applications.is_empty());
    assert_eq!(
        format!("{:#?}", observed.unchanged_requery.unwrap()),
        format!("{:#?}", observed.initial)
    );
}

#[test]
fn original_unsupported_modifier_precedes_the_original_missing_handler_error() {
    assert_eq!(UNSUPPORTED_MODIFIER_ORIGINAL, "<div @click.nativeish />");
    let observed = pair(&case(
        "old-original-two-errors",
        UNSUPPORTED_MODIFIER_ORIGINAL,
        Order::ValidOnly,
        2,
        0,
    ));
    let spans = observed
        .initial
        .diagnostics
        .iter()
        .map(|d| (d.start, d.end))
        .collect::<Vec<_>>();
    assert_eq!(spans, [(12, 21), (5, 21)]);
}

#[test]
fn original_absent_value_clean_and_error_inputs_keep_complete_results() {
    for (source, count) in [
        ("<div @click.once />", 1),
        ("<div @keyup.enter />", 1),
        ("<div @click.capture />", 1),
        ("<div @click.stop />", 0),
        ("<div @submit.prevent />", 0),
        ("<div @click.once.stop />", 0),
        ("<div @keyup.enter.prevent />", 0),
        ("<div @click></div>", 1),
        ("<div title=\"@click.nativeish\" />", 0),
    ] {
        pair(&case(
            "old-input-or-explicit-static-decoy",
            source,
            Order::ValidOnly,
            count,
            0,
        ));
    }
}

#[test]
fn actual_configured_instances_preserve_custom_modifiers_reset_and_nonverb_policy() {
    for (custom, count) in [(vec!["nativeish".into()], 1), (vec![], 2)] {
        let mut input = case(
            "configured-original",
            UNSUPPORTED_MODIFIER_ORIGINAL,
            Order::ValidOnly,
            count,
            0,
        );
        input.custom_modifiers = Some(custom);
        pair(&input);
    }
    let mut input = case(
        "custom-name-does-not-supply-action",
        "<div @click.custom />",
        Order::ValidOnly,
        1,
        0,
    );
    input.custom_modifiers = Some(vec!["custom".into()]);
    pair(&input);
    let mut reset = case(
        "actual-custom-reset",
        UNSUPPORTED_MODIFIER_ORIGINAL,
        Order::ValidOnly,
        2,
        0,
    );
    reset.custom_modifiers = Some(vec![]);
    let linter = configured(&reset)
        .with_valid_v_on_modifiers(vec!["nativeish".into()])
        .with_valid_v_on_modifiers(vec![]);
    pair_with(&linter, &reset);
}

#[test]
fn original_locale_help_and_severity_preserve_every_diagnostic_field() {
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        for help in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
            for severity in [Severity::Error, Severity::Warning] {
                let mut input = case(
                    "old-original-policy",
                    UNSUPPORTED_MODIFIER_ORIGINAL,
                    Order::ValidOnly,
                    2,
                    0,
                );
                input.locale = locale;
                input.help = help;
                input.severity = Some(severity);
                let observed = pair(&input);
                assert_eq!(
                    observed
                        .initial
                        .diagnostics
                        .iter()
                        .map(|d| d.severity)
                        .collect::<Vec<_>>(),
                    [severity, severity]
                );
            }
        }
    }
}

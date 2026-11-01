//! Bare source-unit qualification only; original component/history packets stay exact.
mod ordering;
mod refusal;
mod support;

use crate::{HelpLevel, Locale, Severity};
use support::{ORIGINAL, Order, RULE, case, pair, span};

#[test]
fn genuine_cross_rule_original_retains_whole_legacy_diagnostic_without_a_fix() {
    // This is the complete input already constructed in valid_v_on's
    // only_verb_modifiers_allow_missing_handler test, not a stripped component.
    assert_eq!(ORIGINAL, "<div @click.native />");
    let observed = pair(&case(
        "cross-rule-original",
        ORIGINAL,
        Order::NativeOnly,
        1,
        0,
    ));
    let diagnostic = observed.initial.diagnostics.first().unwrap();
    assert_eq!(diagnostic.rule_name.as_str(), RULE);
    assert_eq!(diagnostic.start, 12);
    assert_eq!(diagnostic.end, 18);
    assert_eq!(diagnostic.severity, Severity::Error);
    assert!(diagnostic.fix.is_none());
    assert!(observed.applications.is_empty());
    assert_eq!(
        format!("{:#?}", observed.unchanged_requery.unwrap()),
        format!("{:#?}", observed.initial)
    );
}

#[test]
fn exact_first_native_token_and_clean_controls_keep_whole_observations() {
    for (id, source, count) in [
        ("full-syntax", "<div v-on:click.native=\"handler\" />", 1),
        ("combined", "<div @click.native.stop=\"handler\" />", 1),
        (
            "duplicate-first-only",
            "<div @click.native.native=\"handler\" />",
            1,
        ),
        ("missing-handler", "<div @click.native />", 1),
        ("empty-handler", "<div @click.native=\"\" />", 1),
        ("plain-event", "<div @click=\"handler\" />", 0),
        ("substring", "<div @click.nativeish=\"handler\" />", 0),
        ("case-sensitive", "<div @click.Native=\"handler\" />", 0),
        ("static-decoy", "<div title=\".native\" />", 0),
        ("non-on-static", "<div native=\"native\" />", 0),
        (
            "two-elements",
            "<div @click.native=\"a\"/><div @keyup.native=\"b\"/>",
            2,
        ),
        (
            "unicode-crlf",
            "界🌸\r\n<div @click.stop.native=\"handler\" />",
            1,
        ),
    ] {
        let observed = pair(&case(id, source, Order::NativeOnly, count, 0));
        for diagnostic in &observed.initial.diagnostics {
            assert!(diagnostic.fix.is_none());
        }
        if count == 1 {
            let diagnostic = observed.initial.diagnostics.first().unwrap();
            let expected = span(source, "native");
            assert_eq!(
                (diagnostic.start, diagnostic.end),
                (expected.start, expected.end)
            );
        }
    }
}

#[test]
fn configured_locale_help_and_severity_preserve_complete_no_fix_packets() {
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        for help in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
            for severity in [Severity::Error, Severity::Warning] {
                let mut input = case("configured-original", ORIGINAL, Order::NativeOnly, 1, 0);
                input.locale = locale;
                input.help = help;
                input.severity = Some(severity);
                let observed = pair(&input);
                let diagnostic = observed.initial.diagnostics.first().unwrap();
                assert_eq!(diagnostic.severity, severity);
                assert!(diagnostic.fix.is_none());
            }
        }
    }
}

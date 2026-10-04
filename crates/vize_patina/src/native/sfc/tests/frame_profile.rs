use super::super::NativeSfcLintRefusal as Refusal;
use super::support::*;
use crate::{HelpLevel, Severity};
use vize_l0::Span;

#[test]
fn actual_form_feed_closing_frames_preserve_full_original_fatal_parser_vectors() {
    for (tag, closing) in [("script", "</script>"), ("template", "</template>")] {
        let source = SOURCE.replace(closing, &format!("</{tag}\u{c}>"));
        let separator = span(&source, "\u{c}");
        let start = source.find(&format!("<{tag}")).unwrap() as u32;
        for locale in LOCALES {
            let events = log();
            let configured = single(&events, locale);
            assert_eq!(
                configured.lint_native_sfc(&source, FILE).unwrap_err(),
                Refusal::UnsupportedEnvelope { span: separator }
            );
            assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
            // The unchanged original closing finder accepts only SP/TAB/CR/LF.
            // Its fatal block error owns this opener through the original EOF.
            assert_eq!(
                complete(&configured.lint_sfc(&source, FILE)),
                expected(
                    FILE,
                    vec![serde_json::json!({
                        "rule_name": "parser/sfc", "severity": "error",
                        "message": format!("Malformed <{tag}> block: the closing tag is missing."),
                        "start": start, "end": source.len(),
                        "help": null, "labels": [], "fix": null
                    })]
                )
            );
        }
    }
}

#[test]
fn retained_newline_equal_separators_refuse_before_callbacks_without_guessing_lang() {
    for authored in [
        "lang\n = \"ts\"",
        "lang= \n\"ts\"",
        "lang\r=\"ts\"",
        "lang=\u{c}\"ts\"",
    ] {
        let source = SOURCE.replace("lang=\"ts\"", authored);
        let prefix = Span::new(
            source.find("lang").unwrap() as u32 + 4,
            source.find("ts\"").unwrap() as u32,
        );
        for locale in LOCALES {
            let events = log();
            let configured = single(&events, locale);
            assert_eq!(
                configured.lint_native_sfc(&source, FILE).unwrap_err(),
                Refusal::UnsupportedEnvelope { span: prefix }
            );
            assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
            // The actual configured original byte rule still runs on the
            // genuine original script; this does not bless its different attrs.
            assert_eq!(
                complete(&configured.lint_sfc(&source, FILE)),
                expected(
                    FILE,
                    vec![diagnostic(
                        RULE,
                        locale,
                        HelpLevel::Full,
                        "actual",
                        script_span(&source).start,
                        Severity::Warning
                    )]
                )
            );
        }
    }
}

#[test]
fn retained_form_feed_opening_gaps_are_not_original_role_or_separator_authority() {
    for (old, new, gap) in [
        ("script setup", "script\u{c}setup", "\u{c}"),
        ("setup lang", "setup\u{c}lang", "\u{c}"),
        ("ts\">", "ts\"\u{c}>", "\u{c}"),
    ] {
        let source = SOURCE.replace(old, new);
        for locale in LOCALES {
            let events = log();
            let configured = single(&events, locale);
            assert_eq!(
                configured.lint_native_sfc(&source, FILE).unwrap_err(),
                Refusal::UnsupportedEnvelope {
                    span: span(&source, gap)
                }
            );
            assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
            assert_eq!(
                complete(&configured.lint_sfc(&source, FILE)),
                expected(
                    FILE,
                    vec![diagnostic(
                        RULE,
                        locale,
                        HelpLevel::Full,
                        "actual",
                        script_span(&source).start,
                        Severity::Warning
                    )]
                )
            );
        }
    }
}

#[test]
fn immediate_unquoted_carriage_return_preserves_original_value_boundary_refusal() {
    let source = SOURCE.replace("lang=\"ts\"", "lang=ts\r");
    for locale in LOCALES {
        let events = log();
        let configured = single(&events, locale);
        assert_eq!(
            configured.lint_native_sfc(&source, FILE).unwrap_err(),
            Refusal::UnsupportedEnvelope {
                span: span(&source, "\r")
            }
        );
        assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
        assert_eq!(
            complete(&configured.lint_sfc(&source, FILE)),
            expected(
                FILE,
                vec![diagnostic(
                    RULE,
                    locale,
                    HelpLevel::Full,
                    "actual",
                    script_span(&source).start,
                    Severity::Warning
                )]
            )
        );
    }
}

#[test]
fn genuine_sp_tab_equals_and_lf_unquoted_terminators_keep_complete_context_and_outputs() {
    for authored in [
        "lang\t= \"ts\"",
        "lang =\t'ts'",
        "lang=ts\n",
        "lang=ts \r\n",
    ] {
        let source = SOURCE.replace("lang=\"ts\"", authored);
        for locale in LOCALES {
            let events = log();
            let configured = single(&events, locale);
            let expected = expected(
                FILE,
                vec![diagnostic(
                    RULE,
                    locale,
                    HelpLevel::Full,
                    "actual",
                    script_span(&source).start,
                    Severity::Warning,
                )],
            );
            assert_eq!(
                complete(&configured.lint_native_sfc(&source, FILE).unwrap()),
                expected
            );
            assert_eq!(complete(&configured.lint_sfc(&source, FILE)), expected);
            assert_eq!(
                *events.lock().unwrap(),
                [expected_event(
                    &source,
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

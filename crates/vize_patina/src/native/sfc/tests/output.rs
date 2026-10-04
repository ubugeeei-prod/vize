use super::super::*;
use super::support::*;
use crate::{HelpLevel, Severity};
use vize_l0::{Allocator, config::VueVersion};

#[test]
fn complete_configured_output_matches_original_instance_in_all_locales_help_and_severities() {
    for locale in LOCALES {
        for level in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
            for severity in [Severity::Warning, Severity::Error] {
                let events = log();
                let configured = configured(
                    vec![audit(&FIRST, "actual", locale, level, &events)],
                    locale,
                    level,
                )
                .with_rule_severity_overrides(vec![(RULE.into(), severity)]);
                let point = script_span(SOURCE).start;
                let full = expected(
                    FILE,
                    vec![diagnostic(RULE, locale, level, "actual", point, severity)],
                );
                assert_eq!(complete(&configured.lint_sfc(SOURCE, FILE)), full);
                let native = configured.lint_native_sfc(SOURCE, FILE).unwrap();
                assert_eq!(complete(&native), full);
                assert_eq!(
                    complete(&configured.lint_native_sfc(SOURCE, FILE).unwrap()),
                    full
                );
                let event =
                    expected_event(SOURCE, FILE, &FIRST, "actual", locale, level, None, None);
                assert_eq!(*events.lock().unwrap(), [event.clone(), event]);
            }
        }
    }
}

#[test]
fn actual_registry_order_and_declared_default_severity_survive_requested_name_order() {
    let events = log();
    let configured = configured(
        vec![
            audit(&FIRST, "first", crate::Locale::En, HelpLevel::Full, &events),
            audit(
                &SECOND,
                "second",
                crate::Locale::En,
                HelpLevel::Full,
                &events,
            ),
        ],
        crate::Locale::En,
        HelpLevel::Full,
    );
    let point = script_span(SOURCE).start;
    // no-next-tick is earlier in the original catalog than prefer-use-template-ref.
    let full = expected(
        FILE,
        vec![
            diagnostic(
                SECOND_RULE,
                crate::Locale::En,
                HelpLevel::Full,
                "second",
                point,
                Severity::Error,
            ),
            diagnostic(
                RULE,
                crate::Locale::En,
                HelpLevel::Full,
                "first",
                point,
                Severity::Warning,
            ),
        ],
    );
    assert_eq!(complete(&configured.lint_sfc(SOURCE, FILE)), full);
    assert_eq!(
        complete(&configured.lint_native_sfc(SOURCE, FILE).unwrap()),
        full
    );
    assert_eq!(
        *events.lock().unwrap(),
        [
            expected_event(
                SOURCE,
                FILE,
                &SECOND,
                "second",
                crate::Locale::En,
                HelpLevel::Full,
                None,
                None
            ),
            expected_event(
                SOURCE,
                FILE,
                &FIRST,
                "first",
                crate::Locale::En,
                HelpLevel::Full,
                None,
                None
            ),
        ]
    );
}

#[test]
fn original_unicode_crlf_geometry_and_explicit_host_requests_are_retained() {
    let source = "\r\n<script setup lang=\"ts\">\r\nconst text = \"界 ref(null)\"\r\n</script>\r\n<template>界<div ref=\"text\" /></template>\r\n";
    for locale in LOCALES {
        let events = log();
        let configured = single(&events, locale)
            .with_vue_version(Some(VueVersion::V3))
            .with_vapor_mode(Some(false));
        let arena = Allocator::new();
        let native = configured
            .lint_native_sfc_with_allocator(&arena, source, FILE)
            .unwrap();
        let full = expected(
            FILE,
            vec![diagnostic(
                RULE,
                locale,
                HelpLevel::Full,
                "actual",
                script_span(source).start,
                Severity::Warning,
            )],
        );
        assert_eq!(complete(&native), full);
        assert_eq!(complete(&configured.lint_sfc(source, FILE)), full);
        assert_eq!(
            *events.lock().unwrap(),
            [expected_event(
                source,
                FILE,
                &FIRST,
                "actual",
                locale,
                HelpLevel::Full,
                Some(VueVersion::V3),
                Some(false)
            )]
        );
    }
}

#[test]
fn empty_callback_set_still_admits_the_original_whole_template_parser_profile() {
    let configured = crate::Linter::with_registry(crate::RuleRegistry::new());
    assert_eq!(
        complete(&configured.lint_native_sfc(SOURCE, FILE).unwrap()),
        empty(FILE)
    );
    assert_eq!(complete(&configured.lint_sfc(SOURCE, FILE)), empty(FILE));
    let unsupported = SOURCE.replace("ref=\"text\"", ":ref=\"text\"");
    assert_eq!(
        configured.lint_native_sfc(&unsupported, FILE).unwrap_err(),
        NativeSfcLintRefusal::Template(
            crate::native::template::NativeTemplateLintRefusal::UnsupportedAttribute {
                span: span(&unsupported, ":ref=\"text\"")
            }
        )
    );
}

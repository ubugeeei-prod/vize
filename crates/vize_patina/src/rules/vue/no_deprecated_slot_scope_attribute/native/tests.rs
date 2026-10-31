//! Genuine new producer/consumer pairs, not a fabricated lint-history entry.
//! Immutable corpus message/help/fix custody does not widen the native grammar.

mod support;

use crate::native::{
    sfc::NativeSfcLintRefusal as Refusal, template::NativeTemplateLintRefusal as Template,
};
use crate::{HelpLevel, Locale, Severity};
use support::*;
use vize_l0::{Span, config::VueVersion};

#[test]
fn admitted_html_preserves_the_complete_frozen_diagnostic_contract_and_requeries() {
    let frozen = corpus_case("slot-scope-bare");
    assert_eq!(frozen["expected"]["diagnostics"][0]["message"], MESSAGE);
    assert_eq!(frozen["expected"]["diagnostics"][0]["help"], HELP);
    assert_eq!(
        frozen["expected"]["diagnostics"][0]["labels"],
        serde_json::json!([])
    );
    assert_eq!(
        frozen["expected"]["diagnostics"][0]["fix"],
        serde_json::Value::Null
    );
    assert_pair(
        &configured(Locale::En, HelpLevel::Full),
        SOURCE,
        "NewHtmlPair.vue",
        &expected(
            "NewHtmlPair.vue",
            &[15],
            Severity::Error,
            Locale::En,
            HelpLevel::Full,
        ),
    );
}

#[test]
fn exact_frozen_component_input_keeps_legacy_output_and_native_context_refusal() {
    let frozen = corpus_case("slot-scope-bare");
    let source = frozen["source"].as_str().unwrap();
    assert_eq!(
        source,
        "<template><Widget><template slot-scope><span /></template></Widget></template>"
    );
    let linter = configured(Locale::En, HelpLevel::Full);
    for _ in 0..2 {
        assert_eq!(
            complete(&linter.lint_sfc(source, "slot-scope-bare.vue")),
            frozen["expected"]
        );
        assert_eq!(
            linter
                .lint_native_sfc(source, "slot-scope-bare.vue")
                .unwrap_err(),
            Refusal::Template(Template::UnsupportedContext {
                span: Span::new(10, 18)
            }),
        );
    }
}

#[test]
fn literal_values_bare_names_unicode_crlf_and_order_keep_original_source_ranges() {
    for source in [
        "<template><div slot-scope>bare</div></template>",
        "<template><span slot-scope='props'>single</span></template>",
        "<template><div slot-scope=props>unquoted</div></template>",
        "\r\n<template>日本語🍣<div data-note='slot-scope' slot-scope=\"{\r\n value\r\n}\">text</div></template>\r\n",
        "<template><div slot-scope='a'><span slot-scope='b'>nested</span></div><span slot-scope>last</span></template>",
    ] {
        let starts: Vec<_> = source
            .match_indices("slot-scope")
            .filter(|(at, _)| !source[..*at].ends_with("data-note='"))
            .map(|(at, _)| u32::try_from(at).unwrap())
            .collect();
        assert_pair(
            &configured(Locale::En, HelpLevel::Full),
            source,
            "Authored.vue",
            &expected(
                "Authored.vue",
                &starts,
                Severity::Error,
                Locale::En,
                HelpLevel::Full,
            ),
        );
    }
}

#[test]
fn lookalike_names_values_and_literal_text_do_not_become_deprecated_attributes() {
    for source in [
        "<template><div data-slot-scope='props'>slot-scope</div></template>",
        "<template><span title='slot-scope' slot-scope-extra='props'>text</span></template>",
        "<template><div slotScope='props' scope='props' SLOT-SCOPE='props'>text</div></template>",
    ] {
        assert_pair(
            &configured(Locale::En, HelpLevel::Full),
            source,
            "Clean.vue",
            &expected(
                "Clean.vue",
                &[],
                Severity::Error,
                Locale::En,
                HelpLevel::Full,
            ),
        );
    }
}

#[test]
fn configured_severity_locale_and_help_preserve_the_complete_original_vector() {
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        for help in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
            for severity in [Severity::Error, Severity::Warning] {
                let linter = configured(locale, help)
                    .with_rule_severity_overrides(vec![(RULE.into(), severity)]);
                assert_pair(
                    &linter,
                    SOURCE,
                    "Configured.vue",
                    &expected("Configured.vue", &[15], severity, locale, help),
                );
            }
        }
    }
}

#[test]
fn native_refusal_does_not_skip_late_comments_interpolation_or_bound_attributes() {
    for (suffix, text, kind) in [
        ("<!-- later -->", "<!-- later -->", "comment"),
        ("{{ later }}", "{{ later }}", "interpolation"),
        (
            "<span :title='later'></span>",
            ":title='later'",
            "attribute",
        ),
        (
            "<template slot-scope='later'></template>",
            "<template slot-scope='later'>",
            "context",
        ),
    ] {
        let source = SOURCE.replace("</template>", &format!("{suffix}</template>"));
        let at = u32::try_from(source.find(text).unwrap()).unwrap();
        let span = Span::new(at, at + u32::try_from(text.len()).unwrap());
        let expected = match kind {
            "comment" => Template::Comment { span },
            "interpolation" => Template::Interpolation { span },
            "attribute" => Template::UnsupportedAttribute { span },
            _ => Template::UnsupportedContext { span },
        };
        assert_eq!(
            configured(Locale::En, HelpLevel::Full)
                .lint_native_sfc(&source, "Later.vue")
                .unwrap_err(),
            Refusal::Template(expected)
        );
    }
}

#[test]
fn vue2_vapor_typeaware_and_unprovided_enabled_rule_controls_remain_explicit() {
    let linter = configured(Locale::En, HelpLevel::Full).with_vue_version(Some(VueVersion::V2));
    assert_eq!(
        complete(&linter.lint_sfc(SOURCE, "Legacy.vue")),
        expected(
            "Legacy.vue",
            &[],
            Severity::Error,
            Locale::En,
            HelpLevel::Full
        )
    );
    assert_eq!(
        linter.lint_native_sfc(SOURCE, "Legacy.vue").unwrap_err(),
        Refusal::UnsupportedVueVersion {
            requested: VueVersion::V2
        }
    );
    assert_eq!(
        configured(Locale::En, HelpLevel::Full)
            .with_vapor_mode(Some(true))
            .lint_native_sfc(SOURCE, "Vapor.vue")
            .unwrap_err(),
        Refusal::UnsupportedVaporMode { requested: true }
    );
    assert_eq!(
        configured(Locale::En, HelpLevel::Full)
            .with_type_aware_lint(true)
            .lint_native_sfc(SOURCE, "Typed.vue")
            .unwrap_err(),
        Refusal::UnsupportedTypeAwareMode
    );
    assert_eq!(
        configured(Locale::En, HelpLevel::Full)
            .with_enabled_rules(Some(vec![RULE.into(), "test/not-registered".into()]))
            .lint_native_sfc(SOURCE, "Unprovided.vue")
            .unwrap_err(),
        Refusal::UnprovidedRule {
            rule: "test/not-registered".into()
        }
    );
}

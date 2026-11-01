//! Actual bare source unit qualification; original 79/native17 capture is unchanged.
mod support;

use super::super::NoDeprecatedVOnNumberModifiers;
use crate::native::template::NativeTemplateLintRefusal as Refusal;
use crate::{HelpLevel, Linter, Locale, RuleMeta, RuleRegistry, Severity};
use support::*;
use vize_l0::{Span, config::VueVersion};

fn span(source: &str, needle: &str) -> Span {
    let start = source.find(needle).unwrap() as u32;
    Span::new(start, start + needle.len() as u32)
}

#[test]
fn original_bare_unit_keeps_the_independent_full_diagnostic_fix_and_requery() {
    assert_eq!(ORIGINAL, r#"<input @keyup.13="submit" />"#);
    let observation = pair(
        "original-shorthand",
        ORIGINAL,
        1,
        1,
        Locale::En,
        HelpLevel::Full,
        None,
    );
    let old = include_str!(
        "../../snapshots/vize_patina__rules__vue__no_deprecated_v_on_number_modifiers__tests__reports_number_modifier_shorthand.snap"
    );
    assert_eq!(
        format!("{:#?}\n", observation.initial.diagnostics),
        old.split_once("\n---\n").unwrap().1
    );
    assert_eq!(observation.applications.len(), 1);
    let applied = observation.applications.first().unwrap();
    assert_eq!(applied.diagnostic_index, 0);
    assert_eq!(
        applied.source.as_str(),
        r#"<input @keyup.enter="submit" />"#
    );
    assert_eq!(
        format!("{:#?}", applied.result),
        format!(
            "{:#?}",
            configured(Locale::En, HelpLevel::Full, None).lint_template(&applied.source, "App.vue")
        )
    );
    assert_eq!(applied.result.error_count, 0);
    assert!(observation.unchanged_requery.is_none());
}

#[test]
fn original_full_combined_first_numeric_and_clean_controls_keep_whole_observations() {
    for (id, source, count, fixes) in [
        ("original-full", r#"<input v-on:keyup.27="cancel" />"#, 1, 1),
        (
            "original-combined",
            r#"<input @keyup.13.stop="submit" />"#,
            1,
            1,
        ),
        (
            "original-first",
            r#"<input @keyup.13.27="handler" />"#,
            1,
            1,
        ),
        (
            "original-named",
            r#"<input @keyup.enter="submit" @keyup.esc="cancel" />"#,
            0,
            0,
        ),
        (
            "original-nonnumeric",
            r#"<input @keyup.f2="handler" />"#,
            0,
            0,
        ),
        ("first-zero", r#"<input @keyup.0.13="handler" />"#, 0, 0),
        ("first-nine", r#"<input @keyup.9.13="handler" />"#, 0, 0),
        ("prefix", r#"<input @keyup.13suffix="submit" />"#, 1, 1),
        (
            "signed-prefix",
            r#"<input @keyup.+13suffix="submit" />"#,
            1,
            1,
        ),
        ("negative", r#"<input @keyup.-13="submit" />"#, 1, 0),
        ("unmapped", r#"<input @keyup.999="submit" />"#, 1, 0),
        (
            "rounded",
            r#"<input @keyup.9007199254740993="submit" />"#,
            1,
            0,
        ),
        (
            "decimal-prefix-exempt",
            r#"<input @keyup.1e2.13="handler" />"#,
            0,
            0,
        ),
        (
            "hex-prefix-exempt",
            r#"<input @keyup.0x13.27="handler" />"#,
            0,
            0,
        ),
        (
            "two-attributes",
            r#"<input @keyup.13="submit" @keydown.27="cancel" />"#,
            2,
            2,
        ),
        (
            "static-decoys",
            r#"<input title=".27" data-key="13" @keyup.enter="13" />"#,
            0,
            0,
        ),
        (
            "unicode-crlf",
            "界🌸\r\n<input v-on:keyup.stop.13suffix.27=\"submit\" />",
            1,
            1,
        ),
    ] {
        pair(id, source, count, fixes, Locale::En, HelpLevel::Full, None);
    }
    let first = pair(
        "first-fix-requeries-next",
        r#"<input @keyup.13.27="handler" />"#,
        1,
        1,
        Locale::En,
        HelpLevel::Full,
        None,
    );
    let applied = first.applications.first().unwrap();
    assert_eq!(
        applied.source.as_str(),
        r#"<input @keyup.enter.27="handler" />"#
    );
    assert_eq!(applied.result.diagnostics.len(), 1);
    assert_eq!(applied.result.error_count, 1);
}

#[test]
fn help_locale_severity_and_disabled_instance_preserve_the_original_contract() {
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        for help in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
            for severity in [Severity::Error, Severity::Warning] {
                let observation = pair(
                    "configured-original",
                    ORIGINAL,
                    1,
                    1,
                    locale,
                    help,
                    Some(severity),
                );
                assert_eq!(
                    observation.initial.diagnostics.first().unwrap().severity,
                    severity
                );
                assert!(
                    observation
                        .initial
                        .diagnostics
                        .first()
                        .unwrap()
                        .fix
                        .is_some()
                );
            }
        }
    }
    let linter =
        configured(Locale::En, HelpLevel::Full, None).with_disabled_rules(vec![RULE.into()]);
    let source = "<input/>";
    assert_eq!(
        format!(
            "{:#?}",
            linter.lint_native_template(source, "Off.vue").unwrap()
        ),
        format!("{:#?}", linter.lint_template(source, "Off.vue"))
    );
    // Empty callback sets retain StaticOnly, even when a disabled rule knew Events.
    assert_eq!(
        linter
            .lint_native_template(ORIGINAL, "Off.vue")
            .unwrap_err(),
        Refusal::UnsupportedAttribute {
            span: span(ORIGINAL, r#"@keyup.13="submit""#)
        }
    );
}

#[test]
fn missing_dynamic_unknown_and_late_refusal_never_publish_a_partial_fix() {
    use crate::native::NativeLintRefusal;
    let linter = configured(Locale::En, HelpLevel::Full, None);
    let late_source = "<input @keyup.13=\"submit\" /><input v-unknown=\"x\"/>";
    let late_case = Case {
        id: "late-unknown-after-original-fix",
        history: SOURCE,
        source: late_source,
        filename: "Refused.vue",
        entry: "template",
        rule: RULE,
        vue_version: None,
        vapor: None,
        diagnostics: 1,
        fixes: 1,
    };
    let late_legacy = observe(&linter, &late_case, false);
    let snapshot = include_str!(
        "../../snapshots/vize_patina__rules__vue__no_deprecated_v_on_number_modifiers__tests__reports_number_modifier_shorthand.snap"
    );
    assert_eq!(
        format!("{:#?}\n", late_legacy.initial.diagnostics),
        snapshot.split_once("\n---\n").unwrap().1
    );
    println!("late_case:{late_case:#?}\nlegacy:{late_legacy:#?}");
    for (source, head) in [
        ("<input @[event].13=\"submit\" />", "@[event].13"),
        ("<input v-on.13=\"handlers\" />", "v-on.13"),
        ("<input @.13=\"submit\" />", "@.13"),
        (
            "<input @keyup.13=\"submit\" /><input v-unknown=\"x\"/>",
            "v-unknown",
        ),
    ] {
        let legacy = linter.lint_template(source, "Refused.vue");
        let actual = linter.lint_native_template(source, "Refused.vue");
        println!("source:{source:?}\nlegacy:{legacy:#?}\nnative:{actual:#?}");
        let expected = Refusal::Header(NativeLintRefusal::UnsupportedDirective {
            span: span(source, head),
        });
        assert_eq!(actual.unwrap_err(), expected);
    }
    for (source, opening) in [
        (
            "<MyInput @keyup.13=\"submit\" />",
            "<MyInput @keyup.13=\"submit\" />",
        ),
        (
            "<template><input @keyup.13=\"submit\" /></template>",
            "<template>",
        ),
        ("<input @keyup.13=\"submit\" /><MyInput/>", "<MyInput/>"),
    ] {
        let actual = linter.lint_native_template(source, "Owner.vue");
        println!(
            "source:{source:?}\nlegacy:{:#?}\nnative:{actual:#?}",
            linter.lint_template(source, "Owner.vue")
        );
        assert_eq!(
            actual.unwrap_err(),
            Refusal::UnsupportedContext {
                span: span(source, opening)
            }
        );
    }
    assert_eq!(
        linter
            .lint_native_sfc(
                "<template><input @keyup.13=\"submit\" /></template>",
                "Original.vue"
            )
            .unwrap_err(),
        crate::native::sfc::NativeSfcLintRefusal::UnprovidedRule { rule: RULE.into() }
    );
    assert_eq!(
        linter
            .with_vue_version(Some(VueVersion::V2))
            .lint_native_template(ORIGINAL, "Vue2.vue")
            .unwrap_err(),
        Refusal::UnsupportedVueVersion {
            requested: VueVersion::V2
        }
    );
}

#[test]
fn mixed_profiles_namesakes_and_unknown_requests_cannot_borrow_event_authority() {
    struct Namesake;
    impl crate::Rule for Namesake {
        fn meta(&self) -> &'static RuleMeta {
            crate::Rule::meta(&NoDeprecatedVOnNumberModifiers)
        }
    }
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(Namesake));
    assert_eq!(
        Linter::with_registry(registry)
            .lint_native_template(ORIGINAL, "Namesake.vue")
            .unwrap_err(),
        Refusal::UnprovidedRule { rule: RULE.into() }
    );
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoDeprecatedVOnNumberModifiers));
    registry.register(Box::new(crate::rules::vue::NoDeprecatedHtmlElementIs));
    let mixed = Linter::with_registry(registry);
    assert_eq!(
        mixed
            .lint_native_template(ORIGINAL, "Mixed.vue")
            .unwrap_err(),
        Refusal::UnsupportedAttribute {
            span: span(ORIGINAL, r#"@keyup.13="submit""#)
        }
    );
    let unknown = configured(Locale::En, HelpLevel::Full, None)
        .with_enabled_rules(Some(vec!["external/unknown".into()]));
    assert_eq!(
        unknown
            .lint_native_template(ORIGINAL, "Unknown.vue")
            .unwrap_err(),
        Refusal::UnprovidedRule {
            rule: "external/unknown".into()
        }
    );
}

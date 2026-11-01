//! Source-unit qualification only; the frozen 79/17 history capture is separate.
mod support;

use crate::native::template::NativeTemplateLintRefusal as Refusal;
use crate::{HelpLevel, Linter, Locale, Severity};
use support::*;
use vize_l0::{Span, config::VueVersion};

#[test]
fn original_bare_unit_retains_its_independent_whole_diagnostic_snapshot_and_observation() {
    let fixture = fixtures()
        .into_iter()
        .find(|c| c.id == "original-unit")
        .unwrap();
    assert_eq!(fixture.source.as_str(), ORIGINAL);
    assert_eq!(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/linter/native-html-element-is-6848/Original.html.txt"
        )),
        ORIGINAL
    );
    let observation = assert_pair(&fixture, Locale::En, HelpLevel::Full, None);
    let old = include_str!(
        "../../snapshots/vize_patina__rules__vue__no_deprecated_html_element_is__tests__reports_is_on_native_element.snap"
    );
    let body = old.split_once("\n---\n").unwrap().1;
    assert_eq!(format!("{:#?}\n", observation.initial.diagnostics), body);
}

#[test]
fn original_documented_sfc_retains_legacy_observation_and_explicit_native_entry_refusal() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/linter/native-html-element-is-6848/OriginalDocumented.vue.txt"
    ));
    assert_eq!(
        source,
        "<template>\n  <div is=\"MyComponent\" />\n</template>\n"
    );
    let case = Case {
        id: "original-module-document",
        history: PRODUCER,
        source,
        filename: "Documented.vue",
        entry: "sfc",
        rule: RULE,
        vue_version: None,
        vapor: None,
        diagnostics: 1,
        fixes: 0,
    };
    let linter = configured(Locale::En, HelpLevel::Full, None);
    let legacy = observe(&linter, &case, false);
    let expected = expected(
        case.filename,
        &[[18, 34]],
        Severity::Error,
        Locale::En,
        HelpLevel::Full,
    );
    assert_eq!(complete(&legacy.initial), expected);
    assert_eq!(
        complete(legacy.unchanged_requery.as_ref().unwrap()),
        expected
    );
    assert_eq!(legacy.applications.len(), 0);
    for _ in 0..2 {
        let refusal = linter.lint_native_sfc(source, case.filename).unwrap_err();
        assert_eq!(
            refusal,
            crate::native::sfc::NativeSfcLintRefusal::UnprovidedRule { rule: RULE.into() }
        );
        println!("{case:#?}\n{legacy:#?}\nnative:{refusal:?}");
    }
}

#[test]
fn checked_static_names_values_unicode_crlf_and_negative_prefixes_keep_complete_cases() {
    for fixture in fixtures() {
        assert_pair(&fixture, Locale::En, HelpLevel::Full, None);
    }
}

#[test]
fn explicit_severity_locale_help_and_disabled_instance_preserve_whole_results() {
    let fixture = fixtures()
        .into_iter()
        .find(|c| c.id == "original-unit")
        .unwrap();
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        for help in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
            for severity in [Severity::Warning, Severity::Error] {
                assert_pair(&fixture, locale, help, Some(severity));
            }
        }
    }
    let linter =
        configured(Locale::En, HelpLevel::Full, None).with_disabled_rules(vec![RULE.into()]);
    let expected = expected("Off.vue", &[], Severity::Error, Locale::En, HelpLevel::Full);
    for source in [ORIGINAL, "<div is=\"v&#117;e:Foo\"/>"] {
        assert_eq!(complete(&linter.lint_template(source, "Off.vue")), expected);
        assert_eq!(
            complete(&linter.lint_native_template(source, "Off.vue").unwrap()),
            expected
        );
    }
}

#[test]
fn entity_dependent_prefixes_and_late_refusals_cannot_publish_partial_findings() {
    for (source, spans) in [
        ("<div is=\"v&#117;e:Foo\"/>", vec![]),
        ("<div is=\"vue:Foo&amp;Bar\"/>", vec![]),
        ("<div is=\"Other&amp;Foo\"/>", vec![[5, 23]]),
        (
            "<div is=\"First\"/><span is=\"v&#117;e:Foo\"/>",
            vec![[5, 15]],
        ),
    ] {
        let at = source.rfind("is=").unwrap();
        let end = source.rfind("/>").unwrap();
        let refusal = Refusal::UnsupportedAttribute {
            span: Span::new(u32::try_from(at).unwrap(), u32::try_from(end).unwrap()),
        };
        assert_refused(source, "Entity.vue", &spans, refusal);
    }
    for (suffix, kind) in [
        ("<!-- late -->", "comment"),
        ("{{ late }}", "interpolation"),
        ("<span :is='late'/>", "attribute"),
        ("<Widget/>", "context"),
    ] {
        let source = format!("{ORIGINAL}{suffix}");
        let start = u32::try_from(ORIGINAL.len()).unwrap();
        let span = Span::new(start, start + u32::try_from(suffix.len()).unwrap());
        let refusal = match kind {
            "comment" => Refusal::Comment { span },
            "interpolation" => Refusal::Interpolation { span },
            "attribute" => Refusal::UnsupportedAttribute {
                span: Span::new(start + 6, start + 16),
            },
            _ => Refusal::UnsupportedContext { span },
        };
        assert_refused(&source, "Later.vue", &[[5, 21]], refusal);
    }
}

#[test]
fn component_dynamic_context_and_unprovided_instance_are_not_clean_certificates() {
    let linter = configured(Locale::En, HelpLevel::Full, None);
    for (source, spans) in [
        ("<Widget is=\"Other\"/>", vec![]),
        ("<component is=\"Other\"/>", vec![]),
        ("<template is=\"Other\"/>", vec![[10, 20]]),
    ] {
        let result = linter.lint_template(source, "Owner.vue");
        assert_eq!(
            complete(&result),
            expected(
                "Owner.vue",
                &spans,
                Severity::Error,
                Locale::En,
                HelpLevel::Full
            )
        );
        assert_eq!(
            linter
                .lint_native_template(source, "Owner.vue")
                .unwrap_err(),
            Refusal::UnsupportedContext {
                span: Span::new(0, u32::try_from(source.len()).unwrap())
            }
        );
    }
    for (linter, expected) in [
        (
            configured(Locale::En, HelpLevel::Full, None).with_vue_version(Some(VueVersion::V2)),
            Refusal::UnsupportedVueVersion {
                requested: VueVersion::V2,
            },
        ),
        (
            configured(Locale::En, HelpLevel::Full, None).with_vapor_mode(Some(true)),
            Refusal::UnsupportedVaporMode { requested: true },
        ),
    ] {
        assert_eq!(
            linter
                .lint_native_template(ORIGINAL, "Dialect.vue")
                .unwrap_err(),
            expected
        );
    }
}

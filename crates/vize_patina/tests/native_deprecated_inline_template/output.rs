use super::support::*;
use vize_l0::Span;
use vize_patina::{
    HelpLevel, LintPreset, Linter, Locale, Rule, Severity, rules::vue::NoDeprecatedInlineTemplate,
};

#[test]
fn original_native_element_input_and_complete_output_survive_all_configured_views() {
    // Exact authored literal in the old reports_inline_template_on_native_element
    // test, now physically shared by registered and native product calls.
    assert_eq!(ORIGINAL, "<div inline-template>Content</div>");
    let mut observations = Vec::new();
    for locale in LOCALES {
        for help in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
            for severity in [Severity::Error, Severity::Warning] {
                observations.push(observe(
                    &configured(locale, help, severity),
                    ORIGINAL,
                    expected(vec![finding(locale, help, severity, Span::new(5, 20))]),
                ));
            }
        }
    }
    println!("original-inline-template-observations {observations:#?}");
    assert_eq!(observations.len(), 18);
    for observation in observations {
        observation.assert_pair();
    }
}

#[test]
fn complete_attribute_values_case_unicode_child_order_and_clean_controls_match() {
    let cases = [
        (
            "<div inline-template='雪&amp;' />",
            Some("inline-template='雪&amp;'"),
        ),
        (
            "<div INLINE-TEMPLATE=literal></div>",
            Some("INLINE-TEMPLATE=literal"),
        ),
        (
            "<div InLiNe-TeMpLaTe=\"\"></div>",
            Some("InLiNe-TeMpLaTe=\"\""),
        ),
        ("雪\r\n<div inline-template></div>", Some("inline-template")),
        (
            "<div data-inline-template title='inline-template'>Content</div>",
            None,
        ),
        ("<div inline-templatex></div>", None),
        ("<div>Content</div>", None),
    ];
    let mut observations = Vec::new();
    for (source, attribute) in cases {
        for locale in LOCALES {
            let diagnostics = attribute
                .map(|attribute| {
                    finding(
                        locale,
                        HelpLevel::Full,
                        Severity::Error,
                        span(source, attribute),
                    )
                })
                .into_iter()
                .collect();
            observations.push(observe(
                &configured(locale, HelpLevel::Full, Severity::Error),
                source,
                expected(diagnostics),
            ));
        }
    }
    let source = "<div inline-template='parent'><span INLINE-TEMPLATE='child'>literal</span></div>";
    observations.push(observe(
        &configured(Locale::En, HelpLevel::Full, Severity::Error),
        source,
        expected(vec![
            finding(
                Locale::En,
                HelpLevel::Full,
                Severity::Error,
                span(source, "inline-template='parent'"),
            ),
            finding(
                Locale::En,
                HelpLevel::Full,
                Severity::Error,
                span(source, "INLINE-TEMPLATE='child'"),
            ),
        ]),
    ));
    println!("inline-template-static-observations {observations:#?}");
    for observation in observations {
        observation.assert_pair();
    }
}

#[test]
fn disabled_precedence_and_unrelated_default_product_output_stay_complete() {
    let observations: Vec<_> = [
        configured(Locale::En, HelpLevel::Full, Severity::Error)
            .with_enabled_rules(Some(vec![RULE.into()]))
            .with_disabled_rules(vec![RULE.into()]),
        configured(Locale::En, HelpLevel::Full, Severity::Error).with_enabled_rules(Some(vec![])),
    ]
    .iter()
    .map(|linter| observe(linter, ORIGINAL, expected(vec![])))
    .collect();
    let builtin =
        Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![RULE.into()]));
    let builtin = observe(
        &builtin,
        ORIGINAL,
        expected(vec![finding(
            Locale::En,
            HelpLevel::Full,
            Severity::Error,
            Span::new(5, 20),
        )]),
    );
    println!("inline-template-disabled-unselected {observations:#?}\n{builtin:#?}");
    for observation in observations {
        observation.assert_pair();
    }
    builtin.assert_pair();
    let original = NoDeprecatedInlineTemplate;
    assert!(original.as_native_template_rule().is_some());
    assert!(original.as_native_sfc_template_rule().is_none());
    let metadata = original.meta();
    assert_eq!(metadata.name, RULE);
    assert_eq!(metadata.default_severity, Severity::Error);
    assert!(!metadata.fixable);
}

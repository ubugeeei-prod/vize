use vize_canon::{
    SfcTypeCheckOptions, SfcTypeCheckResult, SfcTypeSeverity, type_check_sfc,
    type_check_sfc_with_legacy_vue2, type_check_sfc_with_options_api,
};

const CHECKERS: [fn(&str, &SfcTypeCheckOptions) -> SfcTypeCheckResult; 3] = [
    type_check_sfc,
    type_check_sfc_with_options_api,
    type_check_sfc_with_legacy_vue2,
];

#[test]
fn portable_checker_retains_pug_refusals_at_authored_ranges_through_repairs() {
    for ending in ["\n", "\r\n"] {
        for language in ["pug", "jade", "PuG", "JADE"] {
            let source = include_str!(
                "../../_fixtures/differential/typechecker/portable-pug-10434/src/Invalid.vue.txt"
            )
            .replace("\r\n", "\n")
            .replace('\n', ending)
            .replace("lang=\"JADE\"", &format!("lang=\"{language}\""));
            let valid = source.replace("include portable.pug", "p text");
            for checker in CHECKERS {
                for (input, refused) in [(&valid, false), (&source, true), (&valid, false)] {
                    let result = checker(input, &SfcTypeCheckOptions::default());
                    let diagnostics: Vec<_> = result
                        .diagnostics
                        .iter()
                        .map(|diagnostic| {
                            assert!(diagnostic.help.is_none());
                            assert!(diagnostic.related.is_empty());
                            (
                                diagnostic.message.as_str(),
                                diagnostic.start as usize,
                                diagnostic.end as usize,
                                diagnostic.code.as_deref(),
                                diagnostic.severity,
                            )
                        })
                        .collect();
                    let start = source.find("include").unwrap();
                    let expected = if refused {
                        vec![(
                            "pug `include` is not supported in Vue templates: it links other pug files at build time",
                            start,
                            start + "include portable.pug".len(),
                            Some("pug-template-error"),
                            SfcTypeSeverity::Error,
                        )]
                    } else {
                        Vec::new()
                    };
                    assert_eq!(diagnostics, expected, "{language} {ending:?}");
                    assert_eq!(result.error_count, usize::from(refused));
                    assert_eq!(result.warning_count, 0);
                }
            }
        }
    }
}

#[test]
fn portable_pug_and_html_report_the_same_binding_errors_at_authored_bytes() {
    for ending in ["\n", "\r\n"] {
        for language in ["pug", "jade", "PuG", "JADE"] {
            for (pug, html) in [
                ("p {{ missing }}", "<p>{{ missing }}</p>"),
                (
                    "p(:title=\"missing\") text",
                    "<p :title=\"missing\">text</p>",
                ),
                ("p(v-if=\"missing\") text", "<p v-if=\"missing\">text</p>"),
            ] {
                let pug = format!(
                    "<!-- 日本 🧭 -->{ending}<script setup>const sentinel = 1;</script>{ending}<template lang=\"{language}\">{ending}{pug}{ending}</template>"
                );
                let html = format!(
                    "<script setup>const sentinel = 1;</script><template>{html}</template>"
                );
                for checker in CHECKERS {
                    let options = SfcTypeCheckOptions::new("Example.vue");
                    let result = checker(&pug, &options);
                    let reference = checker(&html, &options);
                    assert_eq!(
                        reference.error_count, 1,
                        "HTML control: {html}: {reference:?}"
                    );
                    assert_eq!(result.error_count, reference.error_count);
                    assert_eq!(result.warning_count, reference.warning_count);
                    let reference_diagnostic = reference.diagnostics.first().unwrap();
                    let actual: Vec<_> = result
                        .diagnostics
                        .iter()
                        .map(|diagnostic| {
                            assert!(diagnostic.related.is_empty());
                            (
                                diagnostic.message.as_str(),
                                diagnostic.start as usize,
                                diagnostic.end as usize,
                                diagnostic.code.as_deref(),
                                diagnostic.severity,
                                diagnostic.help.as_deref(),
                            )
                        })
                        .collect();
                    let start = pug.find("missing").unwrap();
                    assert_eq!(
                        actual,
                        vec![(
                            reference_diagnostic.message.as_str(),
                            start,
                            start + "missing".len(),
                            reference_diagnostic.code.as_deref(),
                            reference_diagnostic.severity,
                            reference_diagnostic.help.as_deref(),
                        )],
                        "{pug}"
                    );
                }
            }
        }
    }
}

use super::support::*;
use vize_l0::Allocator;
use vize_patina::{
    HelpLevel, Rule, RuleCategory, Severity, rules::vue::ComponentDefinitionNameCasing,
};

#[test]
fn all_eight_original_filename_inputs_have_complete_native_public_results() {
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/component-name-history/cases.json"
    ))
    .unwrap();
    for locale in LOCALES {
        let linter = configured(locale, HelpLevel::Full);
        for case in cases.as_array().unwrap() {
            assert_eq!(case["history"], "eaafa5a1f67883277407fcf1c5f3f2b2101ef2ed");
            assert_eq!(case["source"], SOURCE);
            assert_eq!(case["entry"], "template");
            assert_eq!(case["rule"], RULE);
            assert!(case["vue_version"].is_null() && case["vapor"].is_null());
            assert_eq!(case["fixes"], 0);
            let filename = case["filename"].as_str().unwrap();
            let findings = if case["diagnostics"] == 0 {
                vec![]
            } else {
                let stem = filename.trim_end_matches(".vue");
                vec![warning(locale, HelpLevel::Full, stem, Severity::Warning)]
            };
            assert_pair(&linter, SOURCE, filename, expected(filename, findings));
        }
    }
}

#[test]
fn actual_registered_metadata_and_callback_capability_are_preserved() {
    let rule = ComponentDefinitionNameCasing;
    let meta = rule.meta();
    assert_eq!(meta.name, RULE);
    assert!(matches!(meta.category, RuleCategory::StronglyRecommended));
    assert_eq!(meta.default_severity, Severity::Warning);
    assert!(!meta.fixable);
    assert!(rule.as_native_template_rule().is_some());
}

#[test]
fn pascal_initialisms_digits_and_single_letters_are_complete_clean_controls() {
    for locale in LOCALES {
        let linter = configured(locale, HelpLevel::Full);
        for filename in [
            "MyComponent.vue",
            "UI.vue",
            "X.vue",
            "Grid2.vue",
            "A1.vue",
            "App.vue",
        ] {
            assert_pair(&linter, SOURCE, filename, expected(filename, vec![]));
        }
    }
}

#[test]
fn kebab_digits_and_lowercase_or_empty_stems_preserve_original_exceptions() {
    for locale in LOCALES {
        let linter = configured(locale, HelpLevel::Full);
        for filename in [
            "my-component.vue",
            "grid-2-col.vue",
            "job-board-2.vue",
            "index.vue",
            "app.vue",
            ".vue",
            ".vue.vue",
        ] {
            assert_pair(&linter, SOURCE, filename, expected(filename, vec![]));
        }
    }
}

#[test]
fn invalid_casing_separator_unicode_and_digit_branches_keep_whole_diagnostics() {
    for locale in LOCALES {
        let linter = configured(locale, HelpLevel::Full);
        for stem in [
            "myComponent",
            "my-Component",
            "-my-component",
            "my-component-",
            "my--component",
            "page.block",
            "my_component",
            "my component",
            "2foo",
            "2-foo",
            "日本語",
            "ÄFoo",
            "foo-Ä",
            "_",
        ] {
            let filename = vize_l0::cstr!("{stem}.vue");
            assert_pair(
                &linter,
                SOURCE,
                &filename,
                expected(
                    &filename,
                    vec![warning(locale, HelpLevel::Full, stem, Severity::Warning)],
                ),
            );
        }
    }
}

#[test]
fn actual_path_basename_and_repeated_suffix_semantics_remain_exact() {
    for locale in LOCALES {
        let linter = configured(locale, HelpLevel::Full);
        for filename in [
            "src/myComponent.vue",
            "C:\\components\\myComponent.vue",
            "src\\mixed/path\\myComponent.vue",
            "myComponent.vue.vue",
        ] {
            assert_pair(
                &linter,
                SOURCE,
                filename,
                expected(
                    filename,
                    vec![warning(
                        locale,
                        HelpLevel::Full,
                        "myComponent",
                        Severity::Warning,
                    )],
                ),
            );
        }
        assert_pair(
            &linter,
            SOURCE,
            "invalid.folder/my-component.vue",
            expected("invalid.folder/my-component.vue", vec![]),
        );
    }
}

#[test]
fn extension_case_and_non_vue_filenames_preserve_original_noop() {
    for locale in LOCALES {
        let linter = configured(locale, HelpLevel::Full);
        for filename in [
            "myComponent.VUE",
            "myComponent.ts",
            "myComponent",
            "myComponent.vue.bak",
        ] {
            assert_pair(&linter, SOURCE, filename, expected(filename, vec![]));
        }
    }
}

#[test]
fn exact_pages_segments_and_windows_dynamic_routes_keep_original_exemption() {
    for locale in LOCALES {
        let linter = configured(locale, HelpLevel::Full);
        for filename in [
            "pages/myComponent.vue",
            "src/pages/myComponent.vue",
            "C:\\pages\\myComponent.vue",
            "[slug].vue",
            "[...param].vue",
            "[unfinished.vue",
        ] {
            assert_pair(&linter, SOURCE, filename, expected(filename, vec![]));
        }
        for filename in [
            "Pages/myComponent.vue",
            "mypages/myComponent.vue",
            "pages-suffix/myComponent.vue",
            "src/pages.vue/myComponent.vue",
        ] {
            assert_pair(
                &linter,
                SOURCE,
                filename,
                expected(
                    filename,
                    vec![warning(
                        locale,
                        HelpLevel::Full,
                        "myComponent",
                        Severity::Warning,
                    )],
                ),
            );
        }
    }
}

#[test]
fn unicode_crlf_original_source_keeps_standalone_root_point_and_owned_output() {
    let source = "前置\r\n<div title='値'>中🍣<span>子</span></div>後置";
    for locale in LOCALES {
        let linter = configured(locale, HelpLevel::Full);
        assert_pair(
            &linter,
            source,
            "myComponent.vue",
            expected(
                "myComponent.vue",
                vec![warning(
                    locale,
                    HelpLevel::Full,
                    "myComponent",
                    Severity::Warning,
                )],
            ),
        );
        let result = {
            let arena = Allocator::default();
            linter
                .lint_native_template_with_allocator(&arena, source, "myComponent.vue")
                .unwrap()
        };
        assert_eq!(
            complete(&result),
            expected(
                "myComponent.vue",
                vec![warning(
                    locale,
                    HelpLevel::Full,
                    "myComponent",
                    Severity::Warning
                )]
            )
        );
    }
}

#[test]
fn empty_original_input_still_runs_actual_root_callback() {
    for locale in LOCALES {
        assert_pair(
            &configured(locale, HelpLevel::Full),
            "",
            "myComponent.vue",
            expected(
                "myComponent.vue",
                vec![warning(
                    locale,
                    HelpLevel::Full,
                    "myComponent",
                    Severity::Warning,
                )],
            ),
        );
    }
}

#[test]
fn original_static_opaque_attributes_and_entities_keep_complete_root_output() {
    for source in [
        "<div data-x='a &quot; b' title='値' disabled>内容</div>",
        "<section title='{{ opaque }}' data-x=v-if><span>content</span></section>",
        "<div id = \"name\" aria-label='value'></div>",
    ] {
        for locale in LOCALES {
            assert_pair(
                &configured(locale, HelpLevel::Full),
                source,
                "myComponent.vue",
                expected(
                    "myComponent.vue",
                    vec![warning(
                        locale,
                        HelpLevel::Full,
                        "myComponent",
                        Severity::Warning,
                    )],
                ),
            );
        }
    }
}

#[test]
fn ordinary_self_closing_compatibility_notice_is_not_a_reported_parser_diagnostic() {
    for source in ["<div />", "<span/>", "<div>before<br/>after</div>"] {
        for locale in LOCALES {
            assert_pair(
                &configured(locale, HelpLevel::Full),
                source,
                "myComponent.vue",
                expected(
                    "myComponent.vue",
                    vec![warning(
                        locale,
                        HelpLevel::Full,
                        "myComponent",
                        Severity::Warning,
                    )],
                ),
            );
        }
    }
}

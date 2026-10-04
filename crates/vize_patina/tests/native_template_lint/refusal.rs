use super::support::*;
use vize_l0::Span;
use vize_patina::{
    HelpLevel, LintPreset, Linter, Severity,
    native::{NativeLintRefusal, template::NativeTemplateLintRefusal as Refusal},
};

const FILE: &str = "myComponent.vue";
fn span(source: &str, text: &str) -> Span {
    let start = source.find(text).unwrap() as u32;
    Span::new(start, start + text.len() as u32)
}
fn root_warning(locale: vize_patina::Locale) -> serde_json::Value {
    warning(locale, HelpLevel::Full, "myComponent", Severity::Warning)
}
fn original_warning(source: &str) {
    for locale in LOCALES {
        assert_eq!(
            complete(&configured(locale, HelpLevel::Full).lint_template(source, FILE)),
            expected(FILE, vec![root_warning(locale)])
        );
    }
}

#[test]
fn ordinary_comments_refuse_even_when_the_full_original_root_warning_is_unsuppressed() {
    for source in [
        "<!-- ordinary --><div>Content</div>",
        "<div>Content<!-- ordinary --></div>",
    ] {
        original_warning(source);
        for locale in LOCALES {
            assert_eq!(
                configured(locale, HelpLevel::Full)
                    .lint_native_template(source, FILE)
                    .unwrap_err(),
                Refusal::Comment {
                    span: span(source, "<!-- ordinary -->")
                }
            );
        }
    }
}

#[test]
fn late_true_global_suppression_discards_pending_root_output_without_clean_credit() {
    let source = "<div>Content</div><!-- eslint-disable -->";
    for locale in LOCALES {
        let linter = configured(locale, HelpLevel::Full);
        assert_eq!(
            complete(&linter.lint_template(source, FILE)),
            expected(FILE, vec![])
        );
        assert_eq!(
            linter.lint_native_template(source, FILE).unwrap_err(),
            Refusal::Comment {
                span: span(source, "<!-- eslint-disable -->")
            }
        );
    }
}

#[test]
fn ignore_region_and_opening_line_gaps_keep_distinct_complete_original_results() {
    for (source, suppressed) in [
        (
            "<!-- @vize:ignore-start --><div>Content</div><!-- @vize:ignore-end -->",
            true,
        ),
        (
            "<!-- eslint-disable-next-line -->\n<div>Content</div>",
            false,
        ),
    ] {
        for locale in LOCALES {
            let linter = configured(locale, HelpLevel::Full);
            let diagnostics = if suppressed {
                vec![]
            } else {
                vec![root_warning(locale)]
            };
            assert_eq!(
                complete(&linter.lint_template(source, FILE)),
                expected(FILE, diagnostics)
            );
            assert!(matches!(
                linter.lint_native_template(source, FILE),
                Err(Refusal::Comment { .. })
            ));
        }
    }
}

#[test]
fn late_comment_refusal_still_occurs_with_no_active_registry_callback() {
    let source = "<div>Content</div><!-- ordinary -->";
    let linter = Linter::with_preset(LintPreset::Incremental);
    assert_eq!(
        complete(&linter.lint_template(source, FILE)),
        expected(FILE, vec![])
    );
    assert_eq!(
        linter.lint_native_template(source, FILE).unwrap_err(),
        Refusal::Comment {
            span: span(source, "<!-- ordinary -->")
        }
    );
}

#[test]
fn complete_and_empty_original_interpolations_refuse_without_reparsing_expressions() {
    for source in ["<div>{{ value }}</div>", "<div>{{}}</div>", "{{ unknown }}"] {
        original_warning(source);
        for locale in LOCALES {
            assert!(matches!(
                configured(locale, HelpLevel::Full).lint_native_template(source, FILE),
                Err(Refusal::Interpolation { .. })
            ));
        }
    }
}

#[test]
fn structural_event_and_static_bind_semantic_branches_remain_refused() {
    for (source, head) in [
        ("<div v-if='condition'>Content</div>", "v-if='condition'"),
        ("<div @click='handler'>Content</div>", "@click='handler'"),
        ("<div :title='opaque'>Content</div>", ":title='opaque'"),
        ("<div .title='opaque'>Content</div>", ".title='opaque'"),
    ] {
        original_warning(source);
        for locale in LOCALES {
            assert_eq!(
                configured(locale, HelpLevel::Full)
                    .lint_native_template(source, FILE)
                    .unwrap_err(),
                Refusal::UnsupportedAttribute {
                    span: span(source, head)
                }
            );
        }
    }
}

#[test]
fn dynamic_and_object_binding_keep_precise_existing_header_refusal() {
    for (source, head) in [
        ("<div :[name]='opaque'>Content</div>", ":[name]"),
        ("<div v-bind='attrs'>Content</div>", "v-bind"),
    ] {
        original_warning(source);
        for locale in LOCALES {
            assert_eq!(
                configured(locale, HelpLevel::Full)
                    .lint_native_template(source, FILE)
                    .unwrap_err(),
                Refusal::Header(NativeLintRefusal::UnresolvedBinding {
                    span: span(source, head)
                })
            );
        }
    }
}

#[test]
fn own_exact_and_modified_pre_contexts_cannot_grant_body_or_whole_clean_credit() {
    for source in [
        "<div v-pre><span>{{ raw }}</span></div>",
        "<div v-pre.camel><textarea>{{ raw }}</textarea></div>",
    ] {
        original_warning(source);
        for locale in LOCALES {
            let refused = configured(locale, HelpLevel::Full)
                .lint_native_template(source, FILE)
                .unwrap_err();
            assert!(matches!(
                refused,
                Refusal::UnsupportedAttribute { .. }
                    | Refusal::Header(NativeLintRefusal::LintTag { .. })
            ));
        }
    }
}

#[test]
fn raw_foreign_and_component_owners_remain_refused_with_original_warning_vectors() {
    for (source, opening) in [
        ("<script>plain</script>", "<script>"),
        ("<style>plain</style>", "<style>"),
        ("<textarea>plain</textarea>", "<textarea>"),
        ("<title>plain</title>", "<title>"),
        ("<svg><path /></svg>", "<svg>"),
        ("<math><mi>x</mi></math>", "<math>"),
        ("<Foo />", "<Foo />"),
        ("<DIV>plain</DIV>", "<DIV>"),
    ] {
        original_warning(source);
        for locale in LOCALES {
            assert_eq!(
                configured(locale, HelpLevel::Full)
                    .lint_native_template(source, FILE)
                    .unwrap_err(),
                Refusal::UnsupportedContext {
                    span: span(source, opening)
                }
            );
        }
    }
}

#[test]
fn own_recovery_and_table_roots_refuse_without_relying_on_inherited_facts() {
    for (source, opening) in [
        ("<p>plain</p>", "<p>"),
        ("<form>plain</form>", "<form>"),
        ("<a>plain</a>", "<a>"),
        ("<button>plain</button>", "<button>"),
        ("<b>plain</b>", "<b>"),
        ("<table><tr><td>plain</td></tr></table>", "<table>"),
    ] {
        original_warning(source);
        for locale in LOCALES {
            assert_eq!(
                configured(locale, HelpLevel::Full)
                    .lint_native_template(source, FILE)
                    .unwrap_err(),
                Refusal::UnsupportedContext {
                    span: span(source, opening)
                }
            );
        }
    }
}

#[test]
fn late_text_grammar_gap_discards_root_warning_even_for_original_clean_text() {
    let source = "<div>price < 100</div>";
    original_warning(source);
    for locale in LOCALES {
        assert!(matches!(
            configured(locale, HelpLevel::Full).lint_native_template(source, FILE),
            Err(Refusal::UnsupportedText { .. })
        ));
    }
}

mod parser;

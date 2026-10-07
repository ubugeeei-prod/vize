//! #7905: authored HTML closing and component casing edits.

use serde::Deserialize;
use vize_patina::rules::{
    ComponentCasing, HtmlSelfClosingHtmlOptions, HtmlSelfClosingOptions, HtmlSelfClosingStyle,
};
use vize_patina::{HelpLevel, JsxLang, LintPreset, LintResult, Linter, Severity};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Edit {
    start: u32,
    end: u32,
    new_text: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    rule: String,
    policy: String,
    entry: String,
    source: String,
    fixed: String,
    target: String,
    start: u32,
    end: u32,
    message: String,
    help: String,
    edits: Vec<Edit>,
}

fn linter(rule: &str, policy: &str) -> Linter {
    let rule_name = if rule == "html" {
        "vue/html-self-closing"
    } else {
        "vue/component-name-in-template-casing"
    };
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![rule_name.into()]))
        .with_help_level(HelpLevel::Full);
    if rule == "html" {
        let options = if policy == "never" {
            HtmlSelfClosingOptions {
                html: HtmlSelfClosingHtmlOptions {
                    void: HtmlSelfClosingStyle::Never,
                    normal: HtmlSelfClosingStyle::Never,
                    component: HtmlSelfClosingStyle::Never,
                },
                svg: HtmlSelfClosingStyle::Never,
                math: HtmlSelfClosingStyle::Never,
            }
        } else {
            HtmlSelfClosingOptions::default()
        };
        linter.with_html_self_closing_options(options)
    } else {
        linter.with_component_name_in_template_casing(if policy == "kebab" {
            ComponentCasing::KebabCase
        } else {
            ComponentCasing::PascalCase
        })
    }
}

fn lint(linter: &Linter, source: &str, entry: &str) -> LintResult {
    if entry == "jsx" {
        linter.lint_jsx(source, "Card.jsx", JsxLang::Jsx)
    } else {
        linter.lint_template(source, "Card.vue")
    }
}

fn apply_edits(source: &str, result: &LintResult) -> String {
    let mut edits: Vec<_> = result
        .diagnostics
        .iter()
        .filter_map(|d| d.fix.as_ref())
        .flat_map(|fix| fix.edits.iter())
        .collect();
    edits.sort_by_key(|edit| (edit.start, edit.end));
    assert!(edits.windows(2).all(|pair| pair[0].end <= pair[1].start));
    let mut fixed = source.to_owned();
    for edit in edits.into_iter().rev() {
        fixed.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
    }
    fixed
}

#[test]
fn whole_authored_style_cases_preserve_diagnostics_edits_and_final_bytes() {
    let cases: Vec<Case> = serde_json::from_str(include_str!(
        "fixtures/issue-7905-template-style/cases.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 23);
    for case in cases {
        let linter = linter(&case.rule, &case.policy);
        let result = lint(&linter, &case.source, &case.entry);
        assert_eq!(
            result.filename.as_str(),
            if case.entry == "jsx" {
                "Card.jsx"
            } else {
                "Card.vue"
            },
            "{}",
            case.id
        );
        assert_eq!(result.error_count, 0, "{}", case.id);
        assert_eq!(result.warning_count, 1, "{}", case.id);
        assert_eq!(result.diagnostics.len(), 1, "{}", case.id);
        let d = &result.diagnostics[0];
        assert_eq!(
            d.rule_name,
            if case.rule == "html" {
                "vue/html-self-closing"
            } else {
                "vue/component-name-in-template-casing"
            },
            "{}",
            case.id
        );
        assert_eq!(d.severity, Severity::Warning, "{}", case.id);
        assert_eq!(d.message, case.message, "{}", case.id);
        assert_eq!(d.start, case.start, "{}", case.id);
        assert_eq!(d.end, case.end, "{}", case.id);
        assert_eq!(
            &case.source.as_bytes()[case.start as usize..case.end as usize],
            case.target.as_bytes()
        );
        assert_eq!(d.help.as_deref(), Some(case.help.as_str()), "{}", case.id);
        assert!(d.labels.is_empty());
        if case.edits.is_empty() {
            assert!(d.fix.is_none(), "{}", case.id);
        } else {
            let fix = d.fix.as_ref().unwrap();
            assert_eq!(fix.message, case.help, "{}", case.id);
            assert_eq!(fix.edits.len(), case.edits.len(), "{}", case.id);
            for (actual, expected) in fix.edits.iter().zip(&case.edits) {
                assert_eq!(actual.start, expected.start, "{}", case.id);
                assert_eq!(actual.end, expected.end, "{}", case.id);
                assert_eq!(actual.new_text, expected.new_text, "{}", case.id);
            }
        }
        assert_eq!(
            apply_edits(&case.source, &result),
            case.fixed,
            "{}",
            case.id
        );
        for _ in 0..3 {
            let final_result = lint(&linter, &case.fixed, &case.entry);
            assert_eq!(
                final_result.warning_count,
                usize::from(case.edits.is_empty()),
                "{}",
                case.id
            );
            assert_eq!(
                apply_edits(&case.fixed, &final_result),
                case.fixed,
                "{}",
                case.id
            );
        }
    }
}

#[test]
fn all_four_original_requested_edits_are_complete_and_stable() {
    let source = include_str!("fixtures/issue-7905/CardList.vue.fixture");
    let expected = include_str!("fixtures/issue-7905/original-requested-all-four.vue.fixture");
    let linter = Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![
        "vue/html-self-closing".into(),
        "vue/component-name-in-template-casing".into(),
        "vue/v-slot-style".into(),
        "vue/no-boolean-attr-value".into(),
    ]));
    let result = linter.lint_sfc(source, "CardList.vue");
    assert_eq!(result.diagnostics.len(), 4);
    let fixed = apply_edits(source, &result);
    let template = fixed.get(fixed.find("<template>").unwrap()..).unwrap();
    assert_eq!(template, expected);
    assert_eq!(
        fixed.get(..fixed.find("<template>").unwrap()),
        source.get(..source.find("<template>").unwrap())
    );
    for _ in 0..3 {
        let final_result = linter.lint_sfc(&fixed, "CardList.vue");
        assert_eq!(final_result.error_count, 0);
        assert_eq!(final_result.warning_count, 0);
        assert!(final_result.diagnostics.is_empty());
        assert_eq!(apply_edits(&fixed, &final_result), fixed);
    }
}

#[test]
fn petite_and_pug_casing_keep_diagnostics_without_unsafe_edits() {
    let linter = linter("casing", "default");
    let petite = "<script src=\"https://unpkg.com/petite-vue@0.4.1/dist/petite-vue.iife.js\"></script><my-card></my-card>";
    let pug = "<template lang=\"pug\">my-card</template>";
    for result in [
        linter.lint_standalone_html(petite, "Card.html"),
        linter.lint_sfc(pug, "Card.vue"),
    ] {
        assert_eq!(result.warning_count, 1);
        assert!(result.diagnostics[0].fix.is_none());
    }
}

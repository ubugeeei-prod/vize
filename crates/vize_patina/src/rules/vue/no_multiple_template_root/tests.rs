//! Differential cases captured from `eslint-plugin-vue@10.9.2`.
//!
//! Each fragment was wrapped in an SFC `<template>`, linted through ESLint's
//! flat `Linter` API with `vue-eslint-parser@10.4.1`, then translated back to
//! template-local byte offsets. Mostly ASCII fixtures make that translation
//! exact and keep the expected upstream messages and spans reviewable offline;
//! a multi-byte fixture pins the byte offsets real templates produce.

use super::NoMultipleTemplateRoot;
use crate::linter::{LintResult, Linter};
use crate::preset::LintPreset;
use crate::rule::RuleRegistry;
use vize_l0::String;

const RULE: &str = "vue/no-multiple-template-root";
const PARSER: &str = "parser/template";
const MULTIPLE_ROOT: &str = "The template root requires exactly one element.";
const TEXT_ROOT: &str = "The template root requires an element rather than texts.";
const V_FOR_ROOT: &str = "The template root disallows 'v-for' directives.";
type ExpectedFinding = (&'static str, &'static str, u32, u32);
type DifferentialCase<'a> = (&'a str, &'a [ExpectedFinding]);

fn linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoMultipleTemplateRoot));
    Linter::with_registry(registry)
}

#[test]
fn browser_repaired_nested_interactive_content_is_not_an_extra_vue_root() {
    for source in [
        r#"<button><span>label</span><button>eject</button></button>"#,
        r#"<form><section><form>nested</form></section></form>"#,
    ] {
        let result = linter().lint_template(source, "Probe.vue");
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != RULE),
            "{source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn nested_template_in_an_authored_extra_root_is_still_reported() {
    for source in [
        r#"<first-root/><br/><br/><second-root><template #label><span>label</span></template></second-root>"#,
        r#"<first-root><span>one</span></first-root><second-root><template #icon><span>two</span></template></second-root>"#,
        r#"
  <a-float-button-group shape="circle">
<a-float-button />
  </a-float-button-group>
  <a-float-button-group shape="square">
<a-float-button>
  <template #icon><span>icon</span></template>
</a-float-button>
  </a-float-button-group>
"#,
    ] {
        let result = linter().lint_template(source, "Probe.vue");
        assert_eq!(
            result
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.rule_name == RULE)
                .count(),
            1,
            "{source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn sfc_custom_block_does_not_hide_an_extra_root_with_a_nested_template() {
    let source = r#"<docs>
`<FloatButton.Group />`
</docs>

<template>
  <a-float-button-group shape="circle">
<a-float-button />
  </a-float-button-group>
  <a-float-button-group shape="square">
<a-float-button>
  <template #icon><span>icon</span></template>
</a-float-button>
  </a-float-button-group>
</template>"#;
    let result = crate::linter::Linter::with_all()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_sfc(source, "Probe.vue");
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == RULE)
            .count(),
        1,
        "{:?}",
        result.diagnostics
    );
}

fn reported(result: &LintResult) -> Vec<(&'static str, String, u32, u32)> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name,
                diagnostic.message.clone(),
                diagnostic.start,
                diagnostic.end,
            )
        })
        .collect()
}

fn lint(source: &str) -> Vec<(&'static str, String, u32, u32)> {
    reported(&linter().lint_template(source, "test.vue"))
}

fn expected(items: &[ExpectedFinding]) -> Vec<(&'static str, String, u32, u32)> {
    items
        .iter()
        .map(|&(rule, message, start, end)| (rule, String::from(message), start, end))
        .collect()
}

#[test]
fn eslint_vue_10_9_2_differential_roots() {
    let cases: &[DifferentialCase<'_>] = &[
        ("<div></div>", &[]),
        ("\n <!-- comment --> <div></div> \n", &[]),
        ("", &[]),
        ("<div></div><div></div>", &[(RULE, MULTIPLE_ROOT, 11, 22)]),
        (
            "<div></div><div></div><div></div>",
            &[(RULE, MULTIPLE_ROOT, 22, 33)],
        ),
        (
            "<div></div><section><div></div></section>",
            &[(RULE, MULTIPLE_ROOT, 11, 41)],
        ),
        (
            "<div></div><section><!-- </section> --><p>x</p></section>",
            &[(RULE, MULTIPLE_ROOT, 11, 57)],
        ),
        ("<slot></slot><div></div>", &[(RULE, MULTIPLE_ROOT, 13, 24)]),
        (r#"<div v-if="a"></div><div v-else></div>"#, &[]),
        (r#"<div v-if="a"></div><!-- gap --><div v-else></div>"#, &[]),
        (r#"<c1 v-if="a"/><c2 v-else-if="b"/><c3 v-else/>"#, &[]),
        (
            r#"<div v-if="a"></div><div v-else></div><p></p>"#,
            &[(RULE, MULTIPLE_ROOT, 38, 45)],
        ),
        (
            r#"<div></div><div v-else-if="a"></div>"#,
            &[(RULE, MULTIPLE_ROOT, 11, 36)],
        ),
        // Multi-byte attribute and text content must not shift the span.
        (
            r#"<div title="日本語">あ</div><section>x</section>"#,
            &[(RULE, MULTIPLE_ROOT, 32, 52)],
        ),
        // An unclosed extra root falls back to its start-tag span.
        (
            "<div></div><section>",
            &[
                (PARSER, "Element is missing end tag.", 11, 20),
                (RULE, MULTIPLE_ROOT, 11, 20),
            ],
        ),
    ];

    for &(source, expected_reports) in cases {
        assert_eq!(lint(source), expected(expected_reports), "source: {source}");
    }
}

#[test]
fn eslint_vue_10_9_2_differential_text_roots() {
    let cases: &[DifferentialCase<'_>] = &[
        ("{{ a }}", &[(RULE, TEXT_ROOT, 0, 7)]),
        ("hello", &[(RULE, TEXT_ROOT, 0, 5)]),
        ("<div></div>text<div></div>", &[(RULE, TEXT_ROOT, 11, 15)]),
        ("<slot></slot>text", &[(RULE, TEXT_ROOT, 13, 17)]),
    ];

    for &(source, expected_reports) in cases {
        assert_eq!(lint(source), expected(expected_reports), "source: {source}");
    }
}

#[test]
fn eslint_vue_10_9_2_differential_disallowed_roots() {
    let cases: &[DifferentialCase<'_>] = &[
        (
            "<slot></slot>",
            &[(RULE, "The template root disallows '<slot>' elements.", 0, 6)],
        ),
        (
            "<template></template>",
            &[(
                RULE,
                "The template root disallows '<template>' elements.",
                0,
                10,
            )],
        ),
        (
            r#"<div v-for="x in xs"></div>"#,
            &[(RULE, V_FOR_ROOT, 0, 21)],
        ),
        (
            r#"<template v-for="x in xs"><li/></template>"#,
            &[
                (
                    RULE,
                    "The template root disallows '<template>' elements.",
                    0,
                    26,
                ),
                (RULE, V_FOR_ROOT, 0, 26),
            ],
        ),
        (
            r#"<div v-if="a" v-for="x in xs"></div><div v-else v-for="y in ys"></div>"#,
            &[(RULE, V_FOR_ROOT, 0, 30), (RULE, V_FOR_ROOT, 36, 64)],
        ),
        (
            r#"<div v-for="a in b" :title="a > b"></div>"#,
            &[(RULE, V_FOR_ROOT, 0, 35)],
        ),
    ];

    for &(source, expected_reports) in cases {
        assert_eq!(lint(source), expected(expected_reports), "source: {source}");
    }
}

#[test]
fn sfc_diagnostics_use_absolute_offsets() {
    let source = "<template>\n  <div>a</div>\n  <div>b</div>\n</template>\n";
    let result = linter().lint_sfc(source, "test.vue");
    assert_eq!(
        reported(&result),
        expected(&[(RULE, MULTIPLE_ROOT, 28, 40)])
    );
    assert_eq!(&source[28..40], "<div>b</div>");
}

#[test]
fn stays_opt_in_for_general_vue_but_can_be_enabled_by_name() {
    assert!(!RuleRegistry::default().has_rule(RULE));
    assert!(!RuleRegistry::with_essential().has_rule(RULE));
    assert!(RuleRegistry::with_opt_in_rules().has_rule(RULE));

    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![String::from(RULE)]))
        .lint_template("<div></div><div></div>", "page.vue");
    assert_eq!(
        reported(&result),
        expected(&[(RULE, MULTIPLE_ROOT, 11, 22)])
    );
}

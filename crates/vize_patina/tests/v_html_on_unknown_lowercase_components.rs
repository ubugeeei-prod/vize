//! Native identity is independent of lexical namespace, as in Vue's compiler.
use vize_patina::{HelpLevel, LintDiagnostic, LintPreset, LintResult, Linter};

const RULE: &str = "vue/no-v-text-v-html-on-component";
const HELP: &str = "**Why:** Components have their own rendering logic. `v-text`/`v-html` would overwrite the component's content unexpectedly.\n\n**Fix:** Pass content via props or slots:\n```vue\n<!-- Instead of -->\n<MyComponent v-html=\"content\" />\n\n<!-- Use slots -->\n<MyComponent><div v-html=\"content\" /></MyComponent>\n```";

fn linter() -> Linter {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_help_level(HelpLevel::Full)
}

fn assert_report(source: &str, filename: &str, component: Option<&str>, sfc: bool) {
    let diagnostics = component
        .map(|tag| {
            let (directive, before, after) = source
                .split_once("v-html=")
                .map(|(before, after)| ("html", before, after))
                .or_else(|| {
                    source
                        .split_once("v-text=")
                        .map(|(before, after)| ("text", before, after))
                })
                .expect("authored content directive");
            let quote = after.chars().next().expect("quoted directive value");
            let (value, _) = after
                .strip_prefix(quote)
                .expect("opening quote")
                .split_once(quote)
                .expect("closing quote");
            let start = u32::try_from(before.len()).expect("fixture start fits diagnostic span");
            let length = format!("v-{directive}=").len() + value.len() + 2 * quote.len_utf8();
            let end =
                start + u32::try_from(length).expect("fixture directive fits diagnostic span");
            LintDiagnostic::error(
                RULE,
                format!("v-{directive} cannot be used on component <{tag}>"),
                start,
                end,
            )
            .with_help(HELP)
        })
        .into_iter()
        .collect();
    let expected = LintResult {
        filename: filename.into(),
        error_count: usize::from(component.is_some()),
        warning_count: 0,
        diagnostics,
    };
    let actual = if sfc {
        linter().lint_sfc(source, filename)
    } else {
        linter().lint_template(source, filename)
    };
    assert_eq!(format!("{actual:#?}"), format!("{expected:#?}"), "{source}");
}

#[test]
fn known_svg_names_remain_native_and_unknown_lowercase_tags_are_components() {
    // Preserve every original source; SVG names receive their native identity
    // even at the component root or across a foreignObject boundary.
    for (source, component) in [
        (r#"<view class="bar" v-html="title"></view>"#, None),
        (r#"<text v-text="label" />"#, None),
        (r#"<my-element v-html="content" />"#, Some("my-element")),
        (
            r#"<svg><foreignObject><view v-html="content" /></foreignObject></svg>"#,
            None,
        ),
    ] {
        assert_report(source, "test.vue", component, false);
    }
}

#[test]
fn native_html_svg_and_math_tags_are_not_components() {
    for source in [
        r#"<section v-html="content"></section>"#,
        r#"<svg><text v-text="label" /></svg>"#,
        r#"<svg><g><view v-html="content" /></g></svg>"#,
        r#"<math><mi v-text="x" /></math>"#,
        r#"<template v-if="ok"><p v-html="content" /></template>"#,
    ] {
        assert_report(source, "test.vue", None, false);
    }
}

#[test]
fn native_tags_and_actual_components_match_pinned_vue_classification() {
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/native-svg-content/cases.json"
    ))
    .expect("original complete Vue/reference matrix");
    for case in cases.as_array().expect("case array") {
        let id = case.get("id").and_then(|id| id.as_str()).expect("case id");
        let template = case
            .get("template")
            .and_then(|source| source.as_str())
            .expect("whole authored template");
        let component = match id {
            "unknown-lowercase"
            | "unknown-lowercase-text"
            | "unknown-in-svg"
            | "unknown-in-math"
            | "unknown-through-foreign-object" => Some("widget"),
            "kebab-component" => Some("my-element"),
            "uppercase-component" => Some("G"),
            "explicit-native-component" => Some("g"),
            "html"
            | "root-svg-group"
            | "root-svg-text"
            | "root-svg-view"
            | "root-math"
            | "chart-svg-slot"
            | "svg-text"
            | "svg-view"
            | "math"
            | "native-through-foreign-object"
            | "nested-svg" => None,
            _ => panic!("unreviewed classification case: {id}"),
        };
        assert_eq!(
            case.get("component").and_then(|value| value.as_bool()),
            Some(component.is_some()),
            "{id}"
        );
        assert_report(template, "test.vue", component, false);
        assert_report(
            &format!("<template>{template}</template>"),
            "test.vue",
            component,
            true,
        );
    }
}

const NPMX_SOURCES: [(&str, &str); 2] = [
    (
        "FacetBarChart.vue",
        include_str!(
            "../../../tests/_fixtures/differential/lint/native-svg-content/FacetBarChart.vue"
        ),
    ),
    (
        "TimelineChartDepSizeSvgSlot.vue",
        include_str!(
            "../../../tests/_fixtures/differential/lint/native-svg-content/TimelineChartDepSizeSvgSlot.vue"
        ),
    ),
];

#[test]
fn original_npmx_chart_slot_and_component_root_svg_are_native() {
    for (filename, source) in NPMX_SOURCES {
        assert_report(source, filename, None, true);
    }
}

#[test]
fn changing_original_svg_groups_to_components_retains_the_error() {
    for (filename, source) in NPMX_SOURCES {
        let source = source
            .replace("<g", "<Watermark")
            .replace("</g>", "</Watermark>");
        assert_report(&source, filename, Some("Watermark"), true);
    }
}

//! Component identity follows Vue's global native-tag set. A component's root
//! SVG nodes and named-slot contents need not have a lexical `<svg>` ancestor.

use vize_patina::{LintPreset, Linter};

fn linter() -> Linter {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["vue/no-v-text-v-html-on-component".into()]))
}

#[test]
fn native_tags_and_actual_components_match_pinned_vue_classification() {
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/native-svg-content/cases.json"
    ))
    .unwrap();
    let linter = linter();
    for case in cases.as_array().unwrap() {
        let source = case["template"].as_str().unwrap();
        let expected = usize::from(case["component"].as_bool().unwrap());
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(
            result.error_count, expected,
            "{case}: {:?}",
            result.diagnostics
        );
        let result = linter.lint_sfc(&format!("<template>{source}</template>"), "test.vue");
        assert_eq!(
            result.error_count, expected,
            "{case}: {:?}",
            result.diagnostics
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
        let result = linter().lint_sfc(source, filename);
        assert_eq!(
            result.error_count, 0,
            "{filename}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn changing_original_svg_groups_to_components_retains_the_error() {
    for (filename, source) in NPMX_SOURCES {
        let source = source
            .replace("<g", "<Watermark")
            .replace("</g>", "</Watermark>");
        let result = linter().lint_sfc(&source, filename);
        assert_eq!(
            result.error_count, 1,
            "{filename}: {:?}",
            result.diagnostics
        );
    }
}

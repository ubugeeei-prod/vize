use vize_croquis::{Analyzer, AnalyzerOptions};

use crate::virtual_ts::generate_virtual_ts;

/// Every generated line that re-emits `void 0, (value) => value`, trimmed, in
/// emission order: the generic child's props resolver, then the per-prop check
/// that reads its instantiated type. The selector call and the whole-props
/// checker call both replace an inline callback with `undefined as any`, so
/// they are deliberately absent — an entry appearing here for either of them
/// would mean the callback is checked, and reported, twice (#3446).
const SEQUENCE_VALUE_LINES: [&str; 2] = [
    r#""transform": (void 0, (value) => value),"#,
    "const __vize_prop_check_0_transform: __VizeResolvedProp<typeof __vize_resolved_Child_props_0, typeof __vize_selected_Child_props_0, 'transform', __Child_0_prop_transform> = (void 0, (value) => value);",
];

#[test]
fn sequence_prop_values_are_grouped_without_mapping_synthetic_parentheses() {
    let script = r#"import Child from "./Child.vue"
"#;
    let expression = "void 0, (value) => value";
    let template = r#"<Child :transform="void 0, (value) => value" />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    // Asserted as whole lines rather than as substrings: a sequence has to be
    // grouped at *every* site that re-emits the authored value, and only an
    // exhaustive list can show that none was missed.
    let emitted_value_lines: Vec<&str> = output
        .code
        .lines()
        .map(str::trim)
        .filter(|line| line.contains(expression))
        .collect();
    assert_eq!(emitted_value_lines, SEQUENCE_VALUE_LINES, "{}", output.code);

    let value_start = template.find(expression).expect("bound expression present");
    let value_range = value_start..value_start + expression.len();
    let value_spans: Vec<_> = output
        .mappings
        .iter()
        .flat_map(|mapping| &mapping.sub_spans)
        .filter(|span| span.src_range == value_range)
        .collect();
    assert_eq!(
        value_spans.len(),
        1,
        "the resolved per-prop check must own the authored expression mapping"
    );
    for span in value_spans {
        assert_eq!(
            &output.code[span.gen_range.clone()],
            expression,
            "synthetic parentheses must stay outside the exact value sub-span"
        );
        assert_eq!(output.code.as_bytes()[span.gen_range.start - 1], b'(');
        assert_eq!(output.code.as_bytes()[span.gen_range.end], b')');
    }
}

#[test]
fn already_parenthesized_sequence_prop_values_are_not_double_wrapped() {
    let script = r#"import Child from "./Child.vue"
"#;
    let template = r#"<Child :transform="(void 0, (value) => value)" />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    assert!(
        !output.code.contains("((void 0, (value) => value))"),
        "authored grouping should not gain redundant parentheses:\n{}",
        output.code
    );
}

#[test]
fn sequence_values_stay_single_in_merged_class_and_spread_entries() {
    let script = r#"import Child from "./Child.vue"
const classes = { active: true }
const props = { id: "child" }
"#;
    let template = r#"<Child class="base" :class="void 0, classes" v-bind="void 0, props" />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    assert!(
        output
            .code
            .contains(r#""class": ["base", (void 0, classes)]"#),
        "a sequence class binding must remain one merged array element:\n{}",
        output.code
    );
    assert!(
        output.code.contains("...(void 0, props),"),
        "a sequence spread binding must remain one spread operand:\n{}",
        output.code
    );
}

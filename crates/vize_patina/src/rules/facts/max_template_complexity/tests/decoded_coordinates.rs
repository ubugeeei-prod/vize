use super::{LintDiagnostic, LintResult, RULE, lint};

const INPUTS: [(&str, &str); 4] = [
    (
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/linter/complexity-decoded-coordinates-10473/Dashboard.double.lf.vue.txt"
        )),
        "'😀'",
    ),
    (
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/linter/complexity-decoded-coordinates-10473/Dashboard.double.crlf.vue.txt"
        )),
        "'😀'",
    ),
    (
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/linter/complexity-decoded-coordinates-10473/Dashboard.single.lf.vue.txt"
        )),
        "\"😀\"",
    ),
    (
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/linter/complexity-decoded-coordinates-10473/Dashboard.single.crlf.vue.txt"
        )),
        "\"😀\"",
    ),
];

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "compare exact complete diagnostic packets"
)]
fn encoded_values_preserve_complete_packet_labels_across_quotes_and_line_endings() {
    for (source, literal) in INPUTS {
        let tag = source.find("<template>").unwrap() as u32;
        let mut expected = LintDiagnostic::warn(
            RULE,
            "Template complexity is too high: cyclomatic 14 (limit 11), cognitive 13 (limit 16)",
            tag,
            tag + "<template>".len() as u32,
        )
        .with_help(
            "Move the deepest branches or loops into child components: a child's complexity \
             never counts toward this component's own score",
        );
        let outer = format!("a &amp;&amp; (b ? {literal} : c)");
        let inner = format!("b ? {literal} : c");
        for (text, message) in [
            (outer.as_str(), "logical: +1 cognitive, +1 cyclomatic"),
            (inner.as_str(), "conditional: +1 cognitive, +1 cyclomatic"),
        ] {
            let start = source.find(text).unwrap() as u32;
            expected = expected.with_label(message, start, start + text.len() as u32);
        }
        // All increments tie, so the first three child conditions complete
        // the five labels after the two authored binding contributions.
        for (at, text) in source.match_indices("x ? y : z").take(3) {
            expected = expected.with_label(
                "conditional: +1 cognitive, +1 cyclomatic",
                at as u32,
                (at + text.len()) as u32,
            );
        }
        let expected = LintResult {
            filename: "Dashboard.vue".into(),
            diagnostics: vec![expected],
            error_count: 0,
            warning_count: 1,
        };
        // Full ordered packet: primary/secondary ranges, message, help,
        // severity, stable rule, counts, filename and explicit fix absence.
        assert_eq!(format!("{:?}", lint(source)), format!("{expected:?}"));
    }
}

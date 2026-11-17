//! `linter/css-style-lane-11550`: the deliberate findings of the L1 style
//! lane (#13222, #13234) in every style language, pinned as authored slices.

use vize_patina::{LintPreset, Linter};

const SOURCE: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/css-style-lane-11550/StyleLanguages.vue.txt"
);

#[test]
fn style_lane_findings_are_authored_slices_in_every_language() {
    let linter = Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![
        "css/no-important".into(),
        "css/no-id-selectors".into(),
    ]));
    let result = linter.lint_sfc(SOURCE, "StyleLanguages.vue");
    let findings: Vec<(&str, usize, &str)> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let line = SOURCE[..diagnostic.start as usize].matches('\n').count() + 1;
            (
                diagnostic.rule_name,
                line,
                &SOURCE[diagnostic.start as usize..diagnostic.end as usize],
            )
        })
        .collect();
    assert_eq!(
        findings,
        [
            // Plain CSS: the flag after `min(…)` (#13222) and an ID selector;
            // a quoted `!important` is not a flag.
            ("css/no-important", 7, "!important"),
            ("css/no-id-selectors", 8, "#main"),
            // SCSS: not the `!default` variable, the `//` comment or `&#{…}`.
            ("css/no-important", 16, "!important"),
            ("css/no-id-selectors", 17, "#title"),
            // Less: a mixin call is not a declaration; `#ns` is a selector.
            ("css/no-important", 25, "!important"),
            ("css/no-id-selectors", 24, "#ns"),
            // Sass: not the commented declaration.
            ("css/no-important", 30, "!important"),
            ("css/no-id-selectors", 29, "#panel"),
            // Stylus, colonless.
            ("css/no-important", 36, "!important"),
        ]
    );
}

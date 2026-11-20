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

const DECLARATIONS: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/css-style-lane-declarations-11550/Declarations.vue.txt"
);

#[test]
fn ported_declaration_rules_report_authored_slices_in_every_language() {
    let linter = Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![
        "css/prefer-logical-properties".into(),
        "css/require-font-display".into(),
        "css/prefer-slotted".into(),
    ]));
    let result = linter.lint_sfc(DECLARATIONS, "Declarations.vue");
    let findings: Vec<(&str, usize, &str)> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let line = DECLARATIONS[..diagnostic.start as usize]
                .matches('\n')
                .count()
                + 1;
            (
                diagnostic.rule_name,
                line,
                &DECLARATIONS[diagnostic.start as usize..diagnostic.end as usize],
            )
        })
        .collect();
    assert_eq!(
        findings,
        [
            // Plain CSS: `@font-face` is reported at its keyword (it used to be
            // the block's first ten bytes); keyframe declarations stay out of
            // scope; `>>>` was never seen because it is not CSS.
            ("css/prefer-logical-properties", 7, "margin-left"),
            ("css/require-font-display", 6, "@font-face"),
            ("css/prefer-slotted", 9, ">>>"),
            // SCSS: not the `$left` variable or an interpolated name.
            ("css/prefer-logical-properties", 15, "padding-right"),
            ("css/prefer-slotted", 17, "/deep/"),
            // Less: not the `@margin-left` variable.
            ("css/prefer-logical-properties", 23, "right"),
            // Sass and Stylus.
            ("css/prefer-logical-properties", 30, "border-left-width"),
            ("css/require-font-display", 27, "@font-face"),
            ("css/prefer-logical-properties", 35, "margin-right"),
        ]
    );
}

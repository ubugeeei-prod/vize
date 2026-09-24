#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "fixture files and signatures are std strings"
)]
#![expect(clippy::string_slice, reason = "tests assert by panicking")]
//! Davinci P4-12c — `<template lang="pug">` lints through its derived Vue
//! template. Differential: a pug SFC reports exactly the diagnostics
//! (rule, severity, message) of the same SFC with its template replaced by
//! the pinned pug's HTML rendering, and every range lands on the authored
//! pug bytes that wrote the construct.

use std::path::PathBuf;

use vize_patina::{LintResult, Linter};

fn signature(result: &LintResult) -> Vec<(String, String, String)> {
    let mut rows: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name.to_owned(),
                format!("{:?}", diagnostic.severity),
                diagnostic.message.as_str().to_owned(),
            )
        })
        .collect();
    rows.sort();
    rows
}

fn spans<'s>(result: &LintResult, source: &'s str) -> Vec<(&'static str, &'s str)> {
    result
        .diagnostics
        .iter()
        .map(|d| (d.rule_name, &source[d.start as usize..d.end as usize]))
        .collect()
}

const SCRIPT: &str = "\n<script setup>\nvar items = [1]\ndebugger\n</script>\n";

#[test]
fn pug_diagnostics_match_the_html_twin_and_land_on_pug() {
    let pug = [
        "<template lang=\"pug\">\nul\n  li(v-for=\"item in items\") {{ item }}\n</template>",
        SCRIPT,
    ]
    .concat();
    let html = [
        "<template><ul><li v-for=\"item in items\">{{ item }}</li></ul></template>",
        SCRIPT,
    ]
    .concat();
    let linter = Linter::new();
    let (from_pug, from_html) = (
        linter.lint_sfc(&pug, "A.vue"),
        linter.lint_sfc(&html, "A.vue"),
    );
    assert!(
        !from_html.diagnostics.is_empty(),
        "the twin has findings to compare"
    );
    assert_eq!(signature(&from_pug), signature(&from_html));
    // Each range covers the authored pug that wrote the construct — here the
    // same bytes the HTML twin's range covers.
    assert_eq!(spans(&from_pug, &pug), spans(&from_html, &html));
    assert_eq!(
        spans(&from_pug, &pug),
        [
            ("vue/component-definition-name-casing", ""),
            ("vue/require-v-for-key", "v-for=\"item in items\""),
        ]
    );
}

#[test]
fn attribute_order_judges_the_authored_order_not_pugs_class_hoisting() {
    // pug renders `.card(v-if="ok")` as `<div class="card" v-if="ok">`;
    // the author wrote the class shorthand first by pug convention, and the
    // lint view keeps `v-if` where it was authored relative to `class`.
    let linter = Linter::new();
    let rule = "vue/attribute-order";
    let count = |source: &str| {
        let result = linter.lint_sfc(source, "A.vue");
        result
            .diagnostics
            .iter()
            .filter(|d| d.rule_name == rule)
            .count()
    };
    let hoisted = "<template lang=\"pug\">\ndiv(v-if=\"ok\" class=\"card\")\n</template>\n";
    assert_eq!(count(hoisted), 0, "authored v-if before class is in order");
    let authored = "<template lang=\"pug\">\ndiv(class=\"card\" v-if=\"ok\")\n</template>\n";
    let twin = "<template><div class=\"card\" v-if=\"ok\"></div></template>\n";
    assert_eq!(
        count(authored),
        count(twin),
        "authored order lints like its HTML"
    );
    assert_eq!(count(twin), 1);
}

#[test]
fn every_matrix_fixture_lints_like_its_pug_rendered_html() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures/davinci-pug/matrix");
    let mut paths: Vec<_> = std::fs::read_dir(&root)
        .expect("fixtures")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "pug"))
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 26);
    let linter = Linter::new();
    let mut findings = 0;
    for path in &paths {
        let pug = std::fs::read_to_string(path).expect("pug");
        let html = std::fs::read_to_string(path.with_extension("html")).expect("html");
        let pug_sfc = ["<template lang=\"pug\">", &pug, "</template>", SCRIPT].concat();
        let html_sfc = ["<template>", &html, "</template>", SCRIPT].concat();
        let (from_pug, from_html) = (
            linter.lint_sfc(&pug_sfc, "Fixture.vue"),
            linter.lint_sfc(&html_sfc, "Fixture.vue"),
        );
        // Every rule but attribute order, which judges the authored order
        // (pug hoists `class`; see the test above), matches the twin.
        let order_blind = |result: &LintResult| {
            let mut rows = signature(result);
            rows.retain(|(rule, _, _)| rule != "vue/attribute-order");
            rows
        };
        assert_eq!(
            order_blind(&from_pug),
            order_blind(&from_html),
            "{}",
            path.display()
        );
        for diagnostic in &from_pug.diagnostics {
            assert!(diagnostic.end as usize <= pug_sfc.len() && diagnostic.start <= diagnostic.end);
        }
        findings += from_pug.diagnostics.len();
    }
    assert!(findings > 0, "the matrix exercises lint findings");
}

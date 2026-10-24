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
    // The finding lands on the element head: the opening tag in HTML, the
    // tag name that opens the element in pug.
    assert_eq!(
        spans(&from_html, &html),
        [("vue/require-v-for-key", "<li v-for=\"item in items\">")]
    );
    assert_eq!(spans(&from_pug, &pug), [("vue/require-v-for-key", "li")]);
}

#[test]
fn jade_alias_reports_the_same_authored_diagnostics_as_pug() {
    let pug = [
        "<template lang=\"pug\">\nul\n  li(v-for=\"item in items\") {{ item }}\n</template>",
        SCRIPT,
    ]
    .concat();
    let jade = pug.replacen("lang=\"pug\"", "lang=\"JADE\"", 1);
    let linter = Linter::new();
    let (from_pug, from_jade) = (
        linter.lint_sfc(&pug, "Pug.vue"),
        linter.lint_sfc(&jade, "Jade.vue"),
    );
    assert_eq!(signature(&from_jade), signature(&from_pug));
    assert_eq!(spans(&from_jade, &jade), [("vue/require-v-for-key", "li")]);
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

#[test]
fn refused_pug_and_jade_report_the_shared_lowering_errors_at_authored_bytes() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures/davinci-pug/refused");
    let mut paths: Vec<_> = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "pug"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty());
    for path in paths {
        let body = std::fs::read_to_string(&path).unwrap();
        let lowered = vize_l1_to_l2::lower::pug::derive_template_source(&body);
        assert!(lowered.has_errors(), "{}", path.display());
        for lang in ["pug", "jade", "PuG", "JADE"] {
            let prefix = format!("<!-- 日本 🧭 -->\r\n<template lang=\"{lang}\">");
            let source = format!("{prefix}{body}</template>\r\n");
            let result = Linter::new().lint_sfc(&source, "Refused.vue");
            let actual: Vec<_> = result
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.rule_name == "parser/template")
                .map(|diagnostic| {
                    (
                        diagnostic.message.as_str(),
                        diagnostic.start,
                        diagnostic.end,
                        diagnostic.severity,
                    )
                })
                .collect();
            let expected: Vec<_> = lowered
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.severity() == vize_l0::diag::Severity::Error)
                .map(|diagnostic| {
                    (
                        diagnostic.message.as_str(),
                        prefix.len() as u32 + diagnostic.span.start,
                        prefix.len() as u32 + diagnostic.span.end,
                        vize_patina::Severity::Error,
                    )
                })
                .collect();
            assert_eq!(actual, expected, "{} ({lang})", path.display());
            assert!(result.has_errors(), "{} ({lang})", path.display());
            assert_eq!(
                result.error_count,
                result
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| diagnostic.severity == vize_patina::Severity::Error)
                    .count()
            );
            for diagnostic in &result.diagnostics {
                assert!(
                    source
                        .get(diagnostic.start as usize..diagnostic.end as usize)
                        .is_some()
                );
                assert!(diagnostic.fix.is_none());
            }
        }
    }
}

#[test]
fn a_refused_template_keeps_complete_independent_script_and_style_diagnostics() {
    let complete =
        |result: &LintResult| -> Vec<serde_json::Value> {
            result.diagnostics.iter()
            .filter(|diagnostic| diagnostic.rule_name != "parser/template")
            .map(|diagnostic| {
                let labels: Vec<_> = diagnostic.labels.iter().map(|label| serde_json::json!({
                    "message": label.message, "start": label.start, "end": label.end,
                })).collect();
                serde_json::json!({
                    "rule_name": diagnostic.rule_name, "severity": diagnostic.severity,
                    "message": diagnostic.message, "start": diagnostic.start, "end": diagnostic.end,
                    "help": diagnostic.help, "labels": labels, "fix": diagnostic.fix,
                })
            }).collect()
        };
    let linter =
        Linter::with_preset(vize_patina::LintPreset::Incremental).with_enabled_rules(Some(vec![
            "script/prefer-ref-over-reactive".into(),
            "css/no-important".into(),
        ]));
    let refused = "include other.pug";
    let accepted = format!("div{}", " ".repeat(refused.len() - 3));
    for lang in ["pug", "jade"] {
        let source = format!(
            "<!-- 🧭 -->\r\n<template lang=\"{lang}\">\r\n{refused}\r\n</template>\r\n\
             <script setup>\r\nimport {{ reactive }} from 'vue';\r\nconst state = reactive({{ count: 0 }});\r\n</script>\r\n\
             <style>.item {{ color: red !important; }}</style>"
        );
        let control = source.replacen(refused, &accepted, 1);
        let clean_template = linter.lint_sfc(&control, "Independent.vue");
        let refused_template = linter.lint_sfc(&source, "Independent.vue");
        assert_eq!(clean_template.diagnostics.len(), 2);
        assert_eq!(complete(&refused_template), complete(&clean_template));
        assert_eq!(refused_template.error_count, clean_template.error_count + 1);
        assert_eq!(refused_template.warning_count, clean_template.warning_count);
        assert_eq!(refused_template.diagnostics.len(), 3);
    }
}

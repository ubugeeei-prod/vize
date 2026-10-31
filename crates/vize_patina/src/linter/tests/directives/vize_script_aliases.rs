use super::Linter;
use crate::{LintResult, OutputFormat, format_results};
use serde_json::{Value, json};
use vize_l0::{String, cstr};

const RULE: &str = "script/no-restricted-globals";
const MESSAGE: &str = "[vize:script/no-restricted-globals] Don't reference `process` directly. Read environment values through a typed config helper so server and client paths stay distinct.";

fn packet(result: LintResult, source: &str, filename: &str) -> Value {
    let output = format_results(
        &[result],
        &[(String::from(filename), String::from(source))],
        OutputFormat::Json,
    );
    serde_json::from_str(&output).unwrap()
}

fn expected(filename: &str, lines: &[u32]) -> Value {
    let messages = lines
        .iter()
        .map(|line| {
            json!({
                "ruleId": RULE,
                "ruleDocsPath": "docs/content/rules/type-and-script.md",
                "severity": 2,
                "message": MESSAGE,
                "line": line,
                "column": 1,
                "endLine": line,
                "endColumn": 8,
            })
        })
        .collect::<Vec<_>>();
    json!([{
        "file": filename,
        "messages": messages,
        "errorCount": lines.len(),
        "warningCount": 0,
    }])
}

#[test]
fn vize_script_aliases_preserve_exact_line_scope_and_enable_order() {
    let cases: &[(&str, &[u32])] = &[
        (
            "// vize-disable-next-line script/no-restricted-globals\nprocess.env.HOME;\nprocess.env.HOME;\n",
            &[3],
        ),
        (
            "process.env.HOME; // vize-disable-line script/no-restricted-globals\nprocess.env.HOME;\n",
            &[2],
        ),
        (
            "/* vize-disable script/no-restricted-globals */\nprocess.env.HOME;\n/* vize-enable script/no-restricted-globals */\nprocess.env.HOME;\n",
            &[4],
        ),
        (
            "/* vize-disable */\nprocess.env.HOME;\n/* vize-enable script/no-restricted-globals */\nprocess.env.HOME;\n",
            &[4],
        ),
        (
            "// vize-disable-next-line script/no-restricted-members\nprocess.env.HOME;\n",
            &[2],
        ),
        (
            "// vize-disable-next-line script/no-restricted-members, vize/script/no-restricted-globals -- migration\nprocess.env.HOME;\nprocess.env.HOME;\n",
            &[3],
        ),
        (
            "// vize-disable-next-lineage script/no-restricted-globals\nprocess.env.HOME;\n",
            &[2],
        ),
        (
            "/* vize-disable */ process.env.HOME;\n/* vize-enable */\nprocess.env.HOME;\n",
            &[3],
        ),
        (
            "// eslint-disable-next-line script/no-restricted-globals\nprocess.env.HOME;\nprocess.env.HOME;\n",
            &[3],
        ),
        (
            "// oxlint-disable-next-line script/no-restricted-globals\nprocess.env.HOME;\nprocess.env.HOME;\n",
            &[3],
        ),
    ];
    let linter = Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_report_unused_disable_directives(true);
    for (body, lines) in cases {
        for newline in ["\n", "\r\n"] {
            let source = cstr!("// 雪🌸\n{body}").replace('\n', newline);
            let expected_lines = lines.iter().map(|line| line + 1).collect::<Vec<_>>();
            assert_eq!(
                packet(linter.lint_script(&source, "case.ts"), &source, "case.ts"),
                expected("case.ts", &expected_lines),
                "{source}"
            );
            let sfc = cstr!(
                "<script setup lang=\"ts\">{newline}{source}</script>{newline}<template><p /></template>{newline}"
            );
            let expected_lines = lines.iter().map(|line| line + 2).collect::<Vec<_>>();
            assert_eq!(
                packet(linter.lint_sfc(&sfc, "Case.vue"), &sfc, "Case.vue"),
                expected("Case.vue", &expected_lines),
                "{sfc}"
            );
        }
    }
}

#[test]
fn vize_script_aliases_only_admit_real_comments() {
    let cases: &[(&str, u32)] = &[
        (
            "const marker = 'vize-disable-next-line script/no-restricted-globals';\nprocess.env.HOME;\n",
            2,
        ),
        (
            "const marker = /vize-disable-next-line script\\/no-restricted-globals/;\nprocess.env.HOME;\n",
            2,
        ),
        (
            "const marker = `vize-disable-next-line script/no-restricted-globals\ntext`;\nprocess.env.HOME;\n",
            3,
        ),
        (
            "// notvize-disable-next-line script/no-restricted-globals\nprocess.env.HOME;\n",
            2,
        ),
    ];
    let linter = Linter::new().with_enabled_rules(Some(vec![RULE.into()]));
    for (source, line) in cases {
        assert_eq!(
            packet(linter.lint_script(source, "case.ts"), source, "case.ts"),
            expected("case.ts", &[*line]),
            "{source}"
        );
        let sfc =
            cstr!("<script setup lang=\"ts\">\n{source}</script>\n<template><p /></template>\n");
        assert_eq!(
            packet(linter.lint_sfc(&sfc, "Case.vue"), &sfc, "Case.vue"),
            expected("Case.vue", &[line + 1]),
            "{sfc}"
        );
    }
}

#[test]
fn unused_vize_script_aliases_do_not_report_unused_template_directives() {
    let source = "<script setup lang=\"ts\">\n// vize-disable-next-line script/no-restricted-globals\nconst home = 1;\n</script>\n<template><p /></template>\n";
    let linter = Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_report_unused_disable_directives(true);
    assert_eq!(
        packet(linter.lint_sfc(source, "Case.vue"), source, "Case.vue"),
        expected("Case.vue", &[])
    );
}

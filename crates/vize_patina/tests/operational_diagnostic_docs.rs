//! Execute authored operational examples and retain complete diagnostic/report packets.

use serde_json::{Value, json};
use std::path::Path;
use vize_atelier_jsx::JsxLang;
use vize_l0::String;
use vize_patina::{
    LintDiagnostic, LintPreset, LintResult, Linter, OutputFormat, RuleRegistry, format_results,
    rule_docs_path,
};

fn page(slug: &str) -> std::string::String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/content/rules/reference")
            .join(format!("{slug}.md")),
    )
    .unwrap()
}

fn example(page: &str, heading: &str) -> String {
    let section = page.split_once(heading).unwrap().1;
    let fence = section.split_once("```").unwrap().1;
    fence
        .split_once('\n')
        .unwrap()
        .1
        .split_once("\n```")
        .unwrap()
        .0
        .into()
}

#[test]
fn operational_diagnostic_paths_resolve_to_their_own_bilingual_authority() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let registry = RuleRegistry::with_all();
    for (id, slug) in [
        ("parser/template", "parser-template"),
        ("parser/sfc", "parser-sfc"),
        ("parser/jsx", "parser-jsx"),
        ("type/project-configuration", "type-project-configuration"),
    ] {
        assert!(!registry.has_rule(id), "operational diagnostic: {id}");
        assert_eq!(
            rule_docs_path(id),
            format!("docs/content/rules/reference/{slug}.md")
        );
        for locale in ["", "ja/"] {
            let page = root.join(format!("docs/content/{locale}rules/reference/{slug}.md"));
            let source = std::fs::read_to_string(&page).unwrap();
            assert_eq!(
                source.lines().find(|line| line.starts_with("# ")),
                Some(format!("# `{id}`").as_str()),
                "{}",
                page.display()
            );
        }
    }
}

fn assert_reports(
    result: &LintResult,
    source: &str,
    expected: Vec<LintDiagnostic>,
    messages: Value,
) {
    assert!(
        result.has_errors(),
        "parser/project failure stays a failure"
    );
    assert_eq!(result.error_count, expected.len());
    assert_eq!(result.warning_count, 0);
    assert_eq!(
        format!("{:?}", result.diagnostics),
        format!("{expected:?}"),
        "complete diagnostic IDs, severity, message, byte spans, help, labels and fixes"
    );
    let sources = [(result.filename.clone(), String::from(source))];
    let output = format_results(std::slice::from_ref(result), &sources, OutputFormat::Json);
    assert_eq!(
        serde_json::from_str::<Value>(&output).unwrap(),
        json!([{
            "file": result.filename, "messages": messages,
            "errorCount": expected.len(), "warningCount": 0,
        }]),
        "complete actual formatted report"
    );
    let mut plain = format!(
        "Patina lint report: {} {} in 1 file\n\n{}\n",
        expected.len(),
        if expected.len() == 1 {
            "error"
        } else {
            "errors"
        },
        result.filename
    );
    for (diagnostic, message) in expected.iter().zip(messages.as_array().unwrap()) {
        plain.push_str(&format!(
            "  {}:{}:{} error {} {}\n    Reference: {}\n",
            result.filename,
            message["line"],
            message["column"],
            diagnostic.rule_name,
            diagnostic.message,
            message["ruleDocsPath"].as_str().unwrap()
        ));
        if let Some(help) = &diagnostic.help {
            plain.push_str(&format!("    Help:\n      {help}\n"));
        }
    }
    assert_eq!(
        format_results(std::slice::from_ref(result), &sources, OutputFormat::Plain),
        plain,
        "complete plain report"
    );
}

#[test]
fn documented_template_failure_and_repair_keep_the_exact_report() {
    let page = page("parser-template");
    let bad = example(&page, "## Bad\n");
    assert_eq!(bad, "<template><div><span>Notice</div></template>");
    let linter = Linter::with_preset(LintPreset::Incremental);
    let result = linter.lint_sfc(&bad, "OperationalExample.vue");
    assert_reports(
        &result,
        &bad,
        vec![LintDiagnostic::error(
            "parser/template",
            "Element is missing end tag.",
            15,
            21,
        )],
        json!([{
            "ruleId": "parser/template",
            "ruleDocsPath": "docs/content/rules/reference/parser-template.md",
            "severity": 2, "message": "[vize:parser/template] Element is missing end tag.",
            "line": 1, "column": 16, "endLine": 1, "endColumn": 22,
        }]),
    );
    let good = linter.lint_sfc(&example(&page, "## Good\n"), "OperationalExample.vue");
    assert_eq!((good.error_count, good.warning_count), (0, 0));
    assert_eq!(format!("{:?}", good.diagnostics), "[]");
}

#[test]
fn documented_sfc_failure_and_repair_keep_the_exact_report() {
    let page = page("parser-sfc");
    let bad = example(&page, "## Bad\n");
    assert_eq!(bad, "<template>\n  <div>Notice</div>");
    let linter = Linter::new();
    let result = linter.lint_sfc(&bad, "OperationalExample.vue");
    assert_reports(
        &result,
        &bad,
        vec![LintDiagnostic::error(
            "parser/sfc",
            "Malformed <template> block: the closing tag is missing.",
            0,
            30,
        )],
        json!([{
            "ruleId": "parser/sfc",
            "ruleDocsPath": "docs/content/rules/reference/parser-sfc.md",
            "severity": 2,
            "message": "[vize:parser/sfc] Malformed <template> block: the closing tag is missing.",
            "line": 1, "column": 1, "endLine": 2, "endColumn": 20,
        }]),
    );
    let good = linter.lint_sfc(&example(&page, "## Good\n"), "OperationalExample.vue");
    assert_eq!((good.error_count, good.warning_count), (0, 0));
    assert_eq!(format!("{:?}", good.diagnostics), "[]");
}

#[test]
fn documented_jsx_failure_and_repair_keep_both_exact_parser_messages() {
    let page = page("parser-jsx");
    let bad = example(&page, "## Bad\n");
    assert_eq!(bad, "export default () => <div><span>Notice</div>;");
    let linter = Linter::with_preset(LintPreset::Incremental);
    let result = linter.lint_jsx(&bad, "OperationalExample.tsx", JsxLang::Tsx);
    assert_reports(
        &result,
        &bad,
        vec![
            LintDiagnostic::error(
                "parser/jsx",
                "Expected corresponding JSX closing tag for 'span'.",
                40,
                43,
            ),
            LintDiagnostic::error("parser/jsx", "Unexpected token", 44, 45),
        ],
        json!([
            {
                "ruleId": "parser/jsx",
                "ruleDocsPath": "docs/content/rules/reference/parser-jsx.md",
                "severity": 2,
                "message": "[vize:parser/jsx] Expected corresponding JSX closing tag for 'span'.",
                "line": 1, "column": 41, "endLine": 1, "endColumn": 44,
            },
            {
                "ruleId": "parser/jsx",
                "ruleDocsPath": "docs/content/rules/reference/parser-jsx.md",
                "severity": 2, "message": "[vize:parser/jsx] Unexpected token",
                "line": 1, "column": 45, "endLine": 1, "endColumn": 46,
            },
        ]),
    );
    let good = linter.lint_jsx(
        &example(&page, "## Good\n"),
        "OperationalExample.tsx",
        JsxLang::Tsx,
    );
    assert_eq!((good.error_count, good.warning_count), (0, 0));
    assert_eq!(format!("{:?}", good.diagnostics), "[]");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn documented_project_configuration_failure_precedes_runtime_and_is_reported_once() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path().canonicalize().unwrap();
    std::fs::create_dir(root.join("src")).unwrap();
    let filename = root.join("src/OperationalExample.vue");
    let tsconfig = root.join("tsconfig.json");
    let page = page("type-project-configuration");
    std::fs::write(&tsconfig, example(&page, "## Bad\n")).unwrap();
    let source = "<script setup lang=\"ts\">const value: any = 1;</script>\n\
                  <template><p>{{ value }}</p></template>";
    std::fs::write(&filename, source).unwrap();
    let configured = || {
        Linter::with_preset(LintPreset::Incremental)
            .with_type_aware_lint(true)
            .with_enabled_rules(Some(vec!["type/no-unsafe-template-binding".into()]))
            .with_type_aware_tsconfig(Some(tsconfig.clone()))
            .with_corsa_path(Some(root.join("runtime-that-must-not-start")))
    };
    let linter = configured();
    let result = linter.lint_sfc(source, filename.to_str().unwrap());
    let message = format!(
        "Cannot load type-aware project configuration {}:\n- File './.generated/tsconfig.base.json' not found.",
        tsconfig.display()
    );
    let help = "Generate or repair the referenced TypeScript configuration before running type-aware lint.";
    assert_reports(
        &result,
        source,
        vec![
            LintDiagnostic::error("type/project-configuration", message.clone(), 0, 0)
                .with_help(help),
        ],
        json!([{
            "ruleId": "type/project-configuration",
            "ruleDocsPath": "docs/content/rules/reference/type-project-configuration.md",
            "severity": 2, "message": format!("[vize:type/project-configuration] {message}"),
            "line": 1, "column": 1, "endLine": 1, "endColumn": 1, "help": help,
        }]),
    );
    let repeated = linter.lint_sfc(source, filename.to_str().unwrap());
    assert_eq!((repeated.error_count, repeated.warning_count), (0, 0));
    assert_eq!(format!("{:?}", repeated.diagnostics), "[]");

    // No type query means no project/runtime admission, even for the invalid config.
    let no_query = configured().lint_sfc(
        "<template><p>Notice</p></template>",
        filename.to_str().unwrap(),
    );
    assert_eq!((no_query.error_count, no_query.warning_count), (0, 0));
    assert_eq!(format!("{:?}", no_query.diagnostics), "[]");

    // Check the documented repair through the actual configuration reader; no
    // native-runtime success or completed type-query result is claimed here.
    std::fs::create_dir(root.join(".generated")).unwrap();
    std::fs::write(
        root.join(".generated/tsconfig.base.json"),
        example(&page, "## Good\n"),
    )
    .unwrap();
    let mut repaired = vize_canon::VirtualProject::new(&root).unwrap();
    repaired.set_tsconfig_path(Some(tsconfig));
    repaired.checked_config_chain_paths().unwrap();
    assert!(repaired.unresolved_tsconfig_extends().is_empty());
}

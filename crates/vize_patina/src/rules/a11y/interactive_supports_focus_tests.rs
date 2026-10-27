use super::InteractiveSupportsFocus;
use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::output::{OutputFormat, format_results};
use crate::rule::RuleRegistry;
use serde_json::Value;
use vize_atelier_jsx::JsxLang;

const CASES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/a11y-listbox-focus-10925/cases.json"
));

fn linter(legacy: bool) -> Linter {
    if legacy {
        return super::tests::create_legacy_linter();
    }
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(InteractiveSupportsFocus));
    Linter::with_registry(registry)
}

fn packet(result: LintResult, source: &str, filename: &str) -> Value {
    for diagnostic in &result.diagnostics {
        assert_eq!(diagnostic.rule_name, "a11y/interactive-supports-focus");
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert!(diagnostic.help.is_some());
        assert!(diagnostic.labels.is_empty());
        assert!(diagnostic.fix.is_none());
    }
    serde_json::from_str(&format_results(
        &[result],
        &[(filename.into(), source.into())],
        OutputFormat::Json,
    ))
    .expect("complete lint JSON")
}

#[test]
fn original_composite_widgets_keeps_children_as_the_focus_owner() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/linter/a11y-listbox-focus-10925/originalCompositeWidgets.vue.txt"
    ));
    for legacy in [false, true] {
        let linter = linter(legacy);
        for newline in ["\n", "\r\n"] {
            let source = source.replace('\n', newline);
            let result = linter.lint_sfc(&source, "CompositeWidgets.vue");
            assert_eq!(result.error_count, 0);
            assert_eq!(result.warning_count, 0, "{legacy}/{newline:?}: {result:?}");
            assert!(result.diagnostics.is_empty());
        }
    }
}

#[test]
fn listbox_ownership_controls_keep_complete_packets_on_native_and_legacy_hosts() {
    let corpus: Value = serde_json::from_str(CASES).expect("pinned listbox corpus");
    let cases = corpus["cases"].as_array().expect("cases");
    assert_eq!(cases.len(), 37);
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/a11y-listbox-focus-10925");
    let native = linter(false);
    let legacy = linter(true);
    for case in cases {
        for route_name in ["vue", "html", "jsx", "tsx"] {
            let route = &case["routes"][route_name];
            let source = std::fs::read_to_string(
                directory.join(route["sourceFile"].as_str().expect("source file")),
            )
            .expect("whole source");
            let filename = route["filename"].as_str().expect("filename");
            for repetition in 0..2 {
                let result = match route_name {
                    "vue" => native.lint_sfc(&source, filename),
                    "html" => native.lint_standalone_html(&source, filename),
                    "jsx" => native.lint_jsx(&source, filename, JsxLang::Jsx),
                    "tsx" => native.lint_jsx(&source, filename, JsxLang::Tsx),
                    _ => unreachable!(),
                };
                assert_eq!(
                    packet(result, &source, filename),
                    route["afterPacket"],
                    "{}/{route_name}/{repetition}",
                    case["id"]
                );
                if matches!(route_name, "vue" | "html") {
                    let result = if route_name == "vue" {
                        legacy.lint_sfc(&source, filename)
                    } else {
                        legacy.lint_template(&source, filename)
                    };
                    assert_eq!(
                        packet(result, &source, filename),
                        route["afterPacket"],
                        "legacy {}/{route_name}/{repetition}",
                        case["id"]
                    );
                }
            }
        }
    }
}

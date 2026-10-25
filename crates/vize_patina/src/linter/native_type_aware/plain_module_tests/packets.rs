//! Complete public JSX/TSX packets; the source replay appends this test only.
use super::{JsxLang, LintPreset, LintResult, Linter};
use serde_json::{Value, json};
use std::path::PathBuf;

const RULES: [&str; 2] = ["type/no-floating-promises", "type/no-reactivity-loss"];

fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result.diagnostics.iter().map(|d| {
        let labels: Vec<_> = d.labels.iter().map(|l| json!({"message":l.message.as_str(),"start":l.start,"end":l.end})).collect();
        json!({"rule_name":d.rule_name,"severity":d.severity,"message":d.message.as_str(),"start":d.start,"end":d.end,"help":d.help.as_ref().map(|help|help.as_str()),"labels":labels,"fix":d.fix})
    }).collect();
    json!({"filename":result.filename.as_str(),"error_count":result.error_count,"warning_count":result.warning_count,"diagnostics":diagnostics})
}

fn warning(rule: &str, message: &str, help: &str, start: usize, end: usize) -> Value {
    json!({"rule_name":rule,"severity":"warning","message":message,"start":start,"end":end,"help":help,"labels":[],"fix":null})
}

fn capture(version: &Value, packets: &[Value]) {
    if let Some(directory) = std::env::var_os("VIZE_PLAIN_TYPE_PACKETS_CAPTURE_DIR") {
        let directory = PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("packets.json"),
            serde_json::to_vec_pretty(&json!({"provider_version":version,"packets":packets}))
                .unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn conditional_plain_type_packets_are_complete() {
    let required = std::env::var("VIZE_PLAIN_TYPE_PACKETS_REQUIRED").as_deref() == Ok("1");
    if std::env::var("VIZE_TEST_DISABLE_TSGO").as_deref() == Ok("1") {
        assert!(
            !required,
            "required native packets cannot run with the checker disabled"
        );
        return;
    }
    let runtime = std::env::var_os("VIZE_PLAIN_TYPE_PACKETS_CORSA_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../node_modules/@typescript")
                .join(format!(
                    "typescript-{}",
                    vize_carton::corsa_resolver::platform_suffix()
                ))
                .join("lib")
                .join(format!("tsc{}", std::env::consts::EXE_SUFFIX))
        });
    if !runtime.is_file() {
        assert!(!required, "required native packet provider is missing");
        return;
    }
    let package: Value = serde_json::from_slice(
        &std::fs::read(
            runtime
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("package.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        package["version"], "7.0.2",
        "packet authority must be the pinned provider"
    );
    let temporary = tempfile::tempdir().unwrap();
    let root = std::env::var_os("VIZE_PLAIN_TYPE_PACKETS_CASE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| temporary.path().to_owned());
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("package.json"), "{\"type\":\"module\"}\n").unwrap();
    std::fs::write(root.join("tsconfig.json"), "{\"compilerOptions\":{\"strict\":true,\"target\":\"ESNext\",\"module\":\"ESNext\",\"moduleResolution\":\"Bundler\",\"jsx\":\"preserve\"},\"include\":[\"*.tsx\"]}\n").unwrap();
    let missing = root.join("missing-provider");
    assert!(!missing.exists());
    let inputs = std::env::var_os("VIZE_PLAIN_TYPE_PACKETS_INPUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/_fixtures/differential/linter/conditional-plain-types-10517")
        });
    let mut packets = Vec::new();
    for kind in ["promise", "reactivity"] {
        for (case, extension) in [
            ("active", "tsx"),
            ("javascript", "jsx"),
            ("declaration-ts", "d.ts"),
            ("declaration-mts", "d.mts"),
            ("declaration-cts", "d.cts"),
            ("default", "tsx"),
            ("host-disabled", "tsx"),
            ("rule-disabled", "tsx"),
            ("unavailable", "tsx"),
            ("no-query", "tsx"),
        ] {
            let filename = root.join(format!("{kind}-{case}.{extension}"));
            let filename = filename.to_str().unwrap();
            let input = match case {
                "no-query" => "no-query.tsx.txt".to_owned(),
                "default" => format!("{kind}-default.tsx.txt"),
                _ => format!("{kind}.{extension}.txt"),
            };
            let current = std::fs::read_to_string(inputs.join(&input)).unwrap();
            std::fs::write(filename, &current).unwrap();
            let selected = if case == "unavailable" || case == "no-query" {
                &missing
            } else {
                &runtime
            };
            let mut linter = Linter::with_preset(LintPreset::Opinionated)
                .with_corsa_path(Some(selected.clone()));
            if case != "default" {
                linter = linter
                    .with_enabled_rules(Some(RULES.iter().map(|rule| (*rule).into()).collect()));
            }
            if case == "host-disabled" {
                linter = linter.with_type_aware_lint(false);
            }
            if case == "rule-disabled" {
                linter =
                    linter.with_disabled_rules(RULES.iter().map(|rule| (*rule).into()).collect());
            }
            let result = linter.lint_jsx(
                &current,
                filename,
                if extension == "jsx" {
                    JsxLang::Jsx
                } else {
                    JsxLang::Tsx
                },
            );
            let diagnostics = if case == "active" {
                if kind == "promise" {
                    let start = current.find("save();").unwrap();
                    vec![warning(
                        "type/no-floating-promises",
                        "Floating Promise must be awaited, returned, or explicitly ignored with `void`",
                        "Add `await`, return the Promise, or prefix it with `void` when the fire-and-forget behavior is intentional.",
                        start,
                        start + 6,
                    )]
                } else {
                    let start = current.find("state.count").unwrap();
                    vec![warning(
                        "type/no-reactivity-loss",
                        "Assigning 'state.count' to 'count' stores a plain snapshot",
                        "Use `toRef(source, 'key')`, `toRefs(source)`, or access the property on the reactive object.",
                        start,
                        start + 11,
                    )]
                }
            } else if case == "unavailable" {
                vec![warning(
                    "type/corsa-runtime",
                    &format!(
                        "Configured Corsa executable does not exist: {} (from configuration)",
                        missing.display()
                    ),
                    "Type-aware lint rules were skipped because the Corsa runtime could not be started. Configure `typeChecker.corsaPath` or install `typescript@^7`.",
                    0,
                    0,
                )]
            } else {
                Vec::new()
            };
            let expected = json!({"filename":filename,"error_count":0,"warning_count":diagnostics.len(),"diagnostics":diagnostics});
            let actual = complete(&result);
            packets.push(json!({"kind":kind,"case":case,"source":current,"expected":expected,"actual":actual}));
            capture(&package["version"], &packets);
            assert_eq!(actual, expected, "{kind}/{case} complete public packet");
            Linter::finish_type_aware_lint(std::slice::from_ref(&linter), &[]);
        }
    }
}

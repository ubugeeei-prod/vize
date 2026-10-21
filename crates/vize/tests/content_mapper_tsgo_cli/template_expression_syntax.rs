//! Pin authored template-expression syntax diagnostics against vue-tsc.

use std::path::PathBuf;

use serde_json::json;
use vize_l0::{String as CompactString, cstr};

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const EXPECTATIONS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/content-mapper-template-expression-syntax/expectations.json"
));

fn fixture_root() -> PathBuf {
    workspace_root()
        .join("tests/_fixtures/differential/typechecker/content-mapper-template-expression-syntax")
}

#[test]
fn standard_tsgo_keeps_template_syntax_diagnostics_authored_and_complete() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let expectations: serde_json::Value = serde_json::from_str(EXPECTATIONS).unwrap();

    for case in expectations["cases"].as_array().unwrap() {
        let source = std::fs::read_to_string(
            fixture_root().join(case["file"].as_str().expect("fixture file")),
        )
        .unwrap();
        for newline in ["\n", "\r\n"] {
            let cases_root = workspace_root().join("target/vize-tests/tests");
            std::fs::create_dir_all(&cases_root).unwrap();
            let project = tempfile::Builder::new()
                .prefix("content-mapper-template-expression-syntax-")
                .tempdir_in(cases_root)
                .unwrap();
            install_mapper_manifest(project.path());
            install_vue_package(project.path());
            std::fs::create_dir(project.path().join("src")).unwrap();
            std::fs::write(
                project.path().join("src/App.vue"),
                source.replace('\n', newline),
            )
            .unwrap();
            std::fs::write(
                project.path().join("tsconfig.json"),
                serde_json::to_vec_pretty(&json!({
                    "compilerOptions": {
                        "strict": true, "noEmit": true, "skipLibCheck": true,
                        "target": "ESNext", "module": "preserve", "types": []
                    },
                    "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
                    "include": ["src"]
                }))
                .unwrap(),
            )
            .unwrap();

            let output = check_project(&tsgo, project.path(), "tsconfig.json");
            assert!(!output.status.success(), "{}", output_text(&output));
            assert!(output.stderr.is_empty(), "{}", output_text(&output));
            let actual = std::str::from_utf8(&output.stdout)
                .unwrap()
                .lines()
                .filter(|line| !line.is_empty())
                .map(|line| CompactString::from(line.replace('\\', "/")))
                .collect::<Vec<_>>();
            let expected = case
                .get("tsgoDiagnostics")
                .unwrap_or(&case["diagnostics"])
                .as_array()
                .unwrap()
                .iter()
                .map(|diagnostic| {
                    cstr!(
                        "src/App.vue({},{}): error {}: {}",
                        diagnostic["line"].as_u64().unwrap(),
                        diagnostic["column"].as_u64().unwrap(),
                        diagnostic["code"].as_str().unwrap(),
                        diagnostic["message"].as_str().unwrap()
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "{}", output_text(&output));
        }
    }
}

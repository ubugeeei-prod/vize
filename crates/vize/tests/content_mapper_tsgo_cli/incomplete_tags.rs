//! Pin standard-tsgo parser diagnostics and script diagnostics after repair.

use std::path::PathBuf;
use std::process::Output;

use serde_json::json;
use vize_l0::{String as CompactString, cstr};

use super::{
    TSGO_ENV, assert_success, check_project, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

const EXPECTATIONS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/content-mapper-incomplete-tag/expectations.json"
));

fn fixture_root() -> PathBuf {
    workspace_root().join("tests/_fixtures/differential/typechecker/content-mapper-incomplete-tag")
}

fn diagnostic_lines(output: &Output) -> Vec<CompactString> {
    assert!(output.stderr.is_empty(), "{}", output_text(output));
    std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| CompactString::from(line.replace('\\', "/")))
        .collect()
}

fn expected_diagnostics(
    oracle: &serde_json::Value,
    case: &serde_json::Value,
    script: &CompactString,
    unfinished: bool,
) -> Vec<CompactString> {
    let mut diagnostics = Vec::new();
    if unfinished {
        for diagnostic in oracle["templateDiagnostics"].as_array().unwrap() {
            diagnostics.push(cstr!(
                "src/Recovery.vue({},{}): error vize100002: {}",
                case["templateLine"].as_u64().unwrap(),
                diagnostic["column"].as_u64().unwrap(),
                diagnostic["message"].as_str().unwrap()
            ));
        }
    } else {
        diagnostics.push(script.clone());
    }
    diagnostics.sort();
    diagnostics
}

#[test]
fn standard_tsgo_pins_unfinished_template_diagnostics_and_restored_script_errors() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let oracle: serde_json::Value = serde_json::from_str(EXPECTATIONS).unwrap();
    for case in oracle["cases"].as_array().unwrap() {
        let original =
            std::fs::read_to_string(fixture_root().join(case["file"].as_str().unwrap())).unwrap();
        for newline in ["\n", "\r\n"] {
            let cases_root = workspace_root().join("target/vize-tests/tests");
            std::fs::create_dir_all(&cases_root).unwrap();
            let project = tempfile::Builder::new()
                .prefix("content-mapper-incomplete-tag-")
                .tempdir_in(cases_root)
                .unwrap();
            install_mapper_manifest(project.path());
            install_vue_package(project.path());
            std::fs::create_dir(project.path().join("src")).unwrap();
            std::fs::write(
                project.path().join("tsconfig.json"),
                serde_json::to_vec_pretty(&json!({
                    "compilerOptions": {
                        "strict": true, "skipLibCheck": true, "target": "ES2022",
                        "module": "ESNext", "moduleResolution": "bundler",
                        "jsx": "preserve", "jsxImportSource": "vue"
                    },
                    "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
                    "files": ["src/Recovery.vue"]
                }))
                .unwrap(),
            )
            .unwrap();
            let complete = original.replace('\n', newline);
            let template = complete.find("<template>").unwrap();
            let unfinished = cstr!(
                "{}<template><Loc</template>{newline}",
                complete.get(..template).unwrap()
            );
            let script_diagnostic = cstr!(
                "src/Recovery.vue({},{}): error TS2322: {}",
                case["line"].as_u64().unwrap(),
                case["column"].as_u64().unwrap(),
                oracle["scriptMessage"].as_str().unwrap()
            );
            for (state, source) in [
                ("complete", complete.as_str()),
                ("unfinished-tag", unfinished.as_str()),
                ("template-repaired", complete.as_str()),
            ] {
                std::fs::write(project.path().join("src/Recovery.vue"), source).unwrap();
                let output = check_project(&tsgo, project.path(), "tsconfig.json");
                assert!(
                    !output.status.success(),
                    "{state}: {}",
                    output_text(&output)
                );
                let mut actual = diagnostic_lines(&output);
                actual.sort();
                let expected = expected_diagnostics(
                    &oracle,
                    case,
                    &script_diagnostic,
                    state == "unfinished-tag",
                );
                assert_eq!(actual, expected, "{state}: {}", output_text(&output));
                if state == "unfinished-tag" {
                    // Official tsgo treats mapper diagnostics as syntactic errors
                    // and skips semantic diagnostics project-wide. Pin that
                    // provider limit without dropping the authored parser errors.
                    assert_eq!(expected.len(), 2);
                } else {
                    assert_eq!(expected, vec![script_diagnostic.clone()]);
                }
            }
            let repaired = complete.replace("'broken'", "1");
            std::fs::write(project.path().join("src/Recovery.vue"), repaired).unwrap();
            let output = check_project(&tsgo, project.path(), "tsconfig.json");
            assert_success(&output);
            assert!(
                diagnostic_lines(&output).is_empty(),
                "{}",
                output_text(&output)
            );
        }
    }
}

//! Pin standard-tsgo acceptance for an authored `props` setup binding (#10196).

use std::path::PathBuf;

use serde::Deserialize;

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const CASES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/content-mapper-props-binding-10196/cases.json"
));

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Fixture {
    historical_issue: u32,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    inputs: Vec<Input>,
    diagnostics: Vec<String>,
}

#[derive(Deserialize)]
struct Input {
    file: String,
    source: String,
}

fn fixture_root() -> PathBuf {
    workspace_root().join(
        "tests/_fixtures/differential/typechecker/content-mapper-props-binding-10196",
    )
}

#[test]
fn standard_tsgo_accepts_authored_props_setup_binding() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");

    let fixture: Fixture = serde_json::from_str(CASES).unwrap();
    assert_eq!(fixture.historical_issue, 10196);
    assert_eq!(fixture.cases.len(), 1, "retain the reported collision");
    for case in fixture.cases {
        assert!(
            case.diagnostics.is_empty(),
            "the reported project is accepted"
        );
        let cases_root = workspace_root().join("target/vize-tests/tests");
        std::fs::create_dir_all(&cases_root).unwrap();
        let project = tempfile::Builder::new()
            .prefix("content-mapper-props-binding-")
            .tempdir_in(cases_root)
            .unwrap();
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        for input in case.inputs {
            let target = project.path().join(input.file);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(fixture_root().join(input.source), target).unwrap();
        }

        let output = check_project(&tsgo, project.path(), "tsconfig.json");
        if !output.status.success() {
            let path = project.path().join("src/MyField.vue");
            let authored = std::fs::read_to_string(&path).unwrap();
            let transform =
                vize_canon::generate_vue_content_mapper_transform(&path, &authored).unwrap();
            panic!(
                "{}: {}\nvirtual TypeScript:\n{}",
                case.id,
                output_text(&output),
                transform.text,
            );
        }
        assert_eq!(output.stdout, b"", "{}: {}", case.id, output_text(&output));
        assert_eq!(output.stderr, b"", "{}: {}", case.id, output_text(&output));
    }
}

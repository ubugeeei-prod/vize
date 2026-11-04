//! A legal Vue boundary must not block another file's semantic diagnostics.
use super::{
    CompactString, TSGO_ENV, check_project, install_mapper_manifest, install_vue_package,
    workspace_root,
};
use std::path::PathBuf;

#[test]
fn legal_quoted_boundaries_keep_neighbor_and_unused_diagnostics() {
    let Some(host) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "unexecuted #11731 cohort: required real host is absent"
        );
        eprintln!("unexecuted #11731 cohort: {TSGO_ENV} is absent");
        return;
    };
    assert!(host.is_file(), "{TSGO_ENV} must identify a real host");
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/recovered-attribute-boundary-11731");
    let original = std::fs::read_to_string(fixture.join("Gap.vue.txt")).unwrap();
    let unused = std::fs::read_to_string(fixture.join("Unused.vue.txt")).unwrap();
    let other = std::fs::read_to_string(fixture.join("Other.vue.txt")).unwrap();
    let config = std::fs::read_to_string(fixture.join("tsconfig.json.txt")).unwrap();
    for newline in ["\n", "\r\n"] {
        for no_unused in [false, true] {
            for variant in ["reported", "valid", "same-file", "unused"] {
                let project = tempfile::tempdir().unwrap();
                install_mapper_manifest(project.path());
                install_vue_package(project.path());
                let gap = match variant {
                    "same-file" => original.replace("const a = 1;", "const a: number = 'x';"),
                    "unused" => unused.clone(),
                    _ => original.clone(),
                };
                let sibling = if variant == "valid" {
                    other.replace("const count: number = 'y';", "const count: number = 1;")
                } else {
                    other.clone()
                };
                std::fs::write(project.path().join("Gap.vue"), gap.replace('\n', newline)).unwrap();
                std::fs::write(
                    project.path().join("Other.vue"),
                    sibling.replace('\n', newline),
                )
                .unwrap();
                let mut options: serde_json::Value = serde_json::from_str(&config).unwrap();
                options["compilerOptions"]["noUnusedLocals"] = no_unused.into();
                std::fs::write(
                    project.path().join("tsconfig.json"),
                    serde_json::to_vec_pretty(&options).unwrap(),
                )
                .unwrap();
                let output = check_project(&host, project.path(), "tsconfig.json");
                let mut expected = CompactString::default();
                if variant == "same-file" {
                    expected.push_str("Gap.vue(2,7): error TS2322: Type 'string' is not assignable to type 'number'.\n");
                }
                if variant == "unused" && no_unused {
                    expected.push_str("Gap.vue(3,7): error TS6133: 'unused' is declared but its value is never read.\n");
                }
                if variant != "valid" {
                    expected.push_str("Other.vue(2,7): error TS2322: Type 'string' is not assignable to type 'number'.\n");
                }
                assert_eq!(
                    output.stdout,
                    expected.as_bytes(),
                    "{variant}/{newline:?}/{no_unused}"
                );
                assert!(output.stderr.is_empty(), "{output:?}");
                assert_eq!(
                    output.status.code(),
                    Some(if expected.is_empty() { 0 } else { 2 })
                );
                eprintln!(
                    "11731 cell variant={variant} newline={newline:?} noUnusedLocals={no_unused} provider={} status={} stdout={expected:?}",
                    host.display(),
                    output.status,
                );
            }
        }
    }
}

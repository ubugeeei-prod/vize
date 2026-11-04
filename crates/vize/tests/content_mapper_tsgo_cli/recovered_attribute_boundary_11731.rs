//! A legal Vue boundary must not block another file's semantic diagnostics.
use super::{
    CompactString, TSGO_ENV, check_project, install_mapper_manifest, install_vue_package,
    workspace_root,
};
use std::path::PathBuf;

const BOUNDARY_TSGO_ENV: &str = "VIZE_TEST_RECOVERED_ATTRIBUTE_CONTENT_MAPPER_TSGO";

/// A missing host is an unexecuted cohort. `VIZE_TEST_REQUIRE_TSGO` makes that a failure.
fn attribute_boundary_host() -> Option<PathBuf> {
    select_attribute_boundary_host(
        std::env::var_os(BOUNDARY_TSGO_ENV).map(PathBuf::from),
        std::env::var_os(TSGO_ENV).map(PathBuf::from),
        std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_some(),
    )
}

fn select_attribute_boundary_host(
    attribute_boundary: Option<PathBuf>,
    conformance: Option<PathBuf>,
    required: bool,
) -> Option<PathBuf> {
    let Some(host) = attribute_boundary.or(conformance) else {
        assert!(
            !required,
            "unexecuted #11731 cohort: VIZE_TEST_REQUIRE_TSGO is set and both {BOUNDARY_TSGO_ENV} and {TSGO_ENV} are absent"
        );
        eprintln!("unexecuted #11731 cohort: neither {BOUNDARY_TSGO_ENV} nor {TSGO_ENV} is set");
        return None;
    };
    assert!(
        host.is_file(),
        "selected #11731 Content Mapper host is not a file: {}",
        host.display()
    );
    eprintln!(
        "#11731 cohort provider={} require={required}",
        host.display()
    );
    Some(host)
}

#[test]
fn attribute_boundary_host_prefers_its_explicit_provider_over_the_conformance_provider() {
    let attribute_boundary = tempfile::NamedTempFile::new().unwrap();
    let conformance = tempfile::NamedTempFile::new().unwrap();
    assert_eq!(
        select_attribute_boundary_host(
            Some(attribute_boundary.path().to_path_buf()),
            Some(conformance.path().to_path_buf()),
            true,
        ),
        Some(attribute_boundary.path().to_path_buf())
    );
}

#[test]
fn attribute_boundary_host_keeps_the_existing_conformance_provider_as_a_fallback() {
    let conformance = tempfile::NamedTempFile::new().unwrap();
    assert_eq!(
        select_attribute_boundary_host(None, Some(conformance.path().to_path_buf()), true),
        Some(conformance.path().to_path_buf())
    );
}

#[test]
fn optional_attribute_boundary_without_either_provider_remains_unexecuted() {
    assert_eq!(select_attribute_boundary_host(None, None, false), None);
}

#[test]
#[should_panic(expected = "unexecuted #11731 cohort: VIZE_TEST_REQUIRE_TSGO is set")]
fn required_attribute_boundary_without_either_provider_fails_closed() {
    let _ = select_attribute_boundary_host(None, None, true);
}

#[test]
#[should_panic(expected = "selected #11731 Content Mapper host is not a file")]
fn invalid_explicit_attribute_boundary_provider_does_not_fall_back_to_another_host() {
    let root = tempfile::tempdir().unwrap();
    let conformance = tempfile::NamedTempFile::new().unwrap();
    let _ = select_attribute_boundary_host(
        Some(root.path().join("missing-tsgo")),
        Some(conformance.path().to_path_buf()),
        true,
    );
}

#[test]
fn legal_quoted_boundaries_keep_neighbor_and_unused_diagnostics() {
    let Some(host) = attribute_boundary_host() else {
        return;
    };
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
                    "11731 cell variant={variant} newline={newline:?} noUnusedLocals={no_unused} provider={} status={} stdout={:?}",
                    host.display(),
                    output.status,
                    std::str::from_utf8(&output.stdout).unwrap(),
                );
            }
        }
    }
}

//! #11729: template quasis are text, but their substitutions are expressions.

use std::{fs, path::PathBuf};

use super::super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

#[test]
fn reserved_props_in_substitutions_keep_original_acceptance_and_authored_errors() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("exact Content Mapper conformance requires {TSGO_ENV}");
        return;
    };
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/reserved-template-substitutions-11729");
    let original = fs::read_to_string(fixture.join("Probe.vue.txt")).unwrap();
    let config = fs::read(fixture.join("tsconfig.json.txt")).unwrap();
    let mut hosts = vec![tsgo];
    if let Some(tsrs) = std::env::var_os("VIZE_TEST_CONTENT_MAPPER_TSRS") {
        hosts.push(PathBuf::from(tsrs));
    }
    for host in hosts {
        assert!(host.is_file(), "{}", host.display());
        for newline in ["\n", "\r\n"] {
            let project = tempfile::tempdir().unwrap();
            install_mapper_manifest(project.path());
            install_vue_package(project.path());
            fs::write(project.path().join("tsconfig.json"), &config).unwrap();
            let original = original.replace('\n', newline);
            let invalid = original.replace("label(zone, from)", "label(true, from)");
            for (source, expected) in [
                (original.as_str(), ""),
                (
                    invalid.as_str(),
                    "Probe.vue(7,32): error TS2345: Argument of type 'boolean' is not assignable to parameter of type 'string'.\nProbe.vue(8,27): error TS2345: Argument of type 'boolean' is not assignable to parameter of type 'string'.\n",
                ),
                (original.as_str(), ""),
            ] {
                fs::write(project.path().join("Probe.vue"), source).unwrap();
                let output = check_project(&host, project.path(), "tsconfig.json");
                assert_eq!(
                    output.status.success(),
                    expected.is_empty(),
                    "{}",
                    output_text(&output)
                );
                assert_eq!(output.stderr.as_slice(), b"", "{}", output_text(&output));
                assert_eq!(
                    std::str::from_utf8(&output.stdout)
                        .unwrap()
                        .replace('\r', ""),
                    expected,
                    "{}",
                    output_text(&output)
                );
            }
        }
    }
}

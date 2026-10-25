//! Complete native and pinned-host Unicode prop edit cycles (#10573).

use std::{path::PathBuf, process::Command};

use serde::Deserialize;
use serde_json::{Value, json};
use vize_canon::sfc_typecheck::{SfcTypeCheckOptions, type_check_sfc};

use super::{TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text};
use crate::support::corsa_requirement;

#[derive(Deserialize)]
struct Fixture {
    issue: u32,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: String,
    valid: String,
    invalid: String,
    mapper_diagnostic: String,
    cli_diagnostic: String,
}

fn cases() -> Vec<Case> {
    let fixture: Fixture = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/unicode-prop-bindings-10573/edit-cycles.json"
    )))
    .unwrap();
    assert_eq!(fixture.issue, 10573);
    assert_eq!(fixture.cases.len(), 11);
    fixture.cases
}

fn capture(provider: &str, case: &str, phase: &str, value: &Value) {
    if let Some(directory) = std::env::var_os("VIZE_UNICODE_PROP_CAPTURE") {
        let directory = PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join(format!("{provider}-{case}-{phase}.json")),
            serde_json::to_vec_pretty(value).unwrap(),
        )
        .unwrap();
    }
}

fn project(mapper: bool) -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(project.path().join("src")).unwrap();
    std::fs::create_dir_all(project.path().join("node_modules")).unwrap();
    let mut config = json!({
        "compilerOptions": {
            "strict": true, "skipLibCheck": true, "target": "ES2022",
            "module": "ESNext", "moduleResolution": "bundler", "types": [],
        },
        "include": ["src/**/*.vue"],
    });
    if mapper {
        config["contentMappers"] = json!([{ "package": "vize", "extensions": [".vue"] }]);
        install_mapper_manifest(project.path());
    }
    std::fs::write(
        project.path().join("tsconfig.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
    install_vue_package(project.path());
    project
}

#[test]
fn native_projection_preserves_all_authored_unicode_mapping_boundaries() {
    for case in cases() {
        for (phase, source) in [
            ("valid", &case.valid),
            ("invalid", &case.invalid),
            ("repaired", &case.valid),
        ] {
            let result = type_check_sfc(
                source,
                &SfcTypeCheckOptions::new("src/Case.vue").with_virtual_ts(),
            );
            let generated = result
                .virtual_ts
                .as_ref()
                .expect("complete native projection");
            let mut packet = serde_json::to_value(&result).unwrap();
            packet.as_object_mut().unwrap().remove("analysis_time_ms");
            packet["virtual_ts_mappings"] = Value::Array(
                result
                    .virtual_ts_mappings
                    .iter()
                    .map(|mapping| {
                        json!({
                            "generated": mapping.gen_range,
                            "authored": mapping.src_range,
                            "subSpans": mapping.sub_spans.iter().map(|span| json!({
                                "generated": span.gen_range,
                                "authored": span.src_range,
                            })).collect::<Vec<_>>(),
                        })
                    })
                    .collect(),
            );
            packet["virtual_ts_semantic_links"] = Value::Array(
                result
                    .virtual_ts_semantic_links
                    .iter()
                    .map(|link| {
                        json!({
                            "source": link.source_range,
                            "target": link.target_range,
                            "kind": format!("{:?}", link.kind),
                        })
                    })
                    .collect(),
            );
            capture("projection", &case.id, phase, &packet);
            assert_eq!(result.error_count, 0, "{} {phase}: {packet}", case.id);
            assert_eq!(result.warning_count, 0, "{} {phase}: {packet}", case.id);
            assert!(
                result.diagnostics.is_empty(),
                "{} {phase}: {packet}",
                case.id
            );
            for mapping in &result.virtual_ts_mappings {
                assert!(
                    generated.get(mapping.gen_range.clone()).is_some(),
                    "{} {phase}: {mapping:?}",
                    case.id
                );
                assert!(
                    source.get(mapping.src_range.clone()).is_some(),
                    "{} {phase}: {mapping:?}",
                    case.id
                );
            }
            for link in &result.virtual_ts_semantic_links {
                assert!(
                    generated.get(link.source_range.clone()).is_some(),
                    "{} {phase}: {link:?}",
                    case.id
                );
                assert!(
                    generated.get(link.target_range.clone()).is_some(),
                    "{} {phase}: {link:?}",
                    case.id
                );
            }
        }
    }
}

#[test]
fn standard_mapper_unicode_props_keep_complete_valid_invalid_repair_packets() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Unicode Content Mapper host: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} must name an executable");
    for case in cases() {
        let project = project(true);
        for (phase, source, invalid) in [
            ("valid", &case.valid, false),
            ("invalid", &case.invalid, true),
            ("repaired", &case.valid, false),
        ] {
            std::fs::write(project.path().join("src/Case.vue"), source).unwrap();
            let output = check_project(&tsgo, project.path(), "tsconfig.json");
            capture(
                "mapper",
                &case.id,
                phase,
                &json!({ "source": source, "status": output.status.code(), "stdout": std::str::from_utf8(&output.stdout).unwrap(), "stderr": std::str::from_utf8(&output.stderr).unwrap() }),
            );
            assert_eq!(
                output.status.code(),
                Some(if invalid { 2 } else { 0 }),
                "{} {phase}: {}",
                case.id,
                output_text(&output)
            );
            assert_eq!(
                std::str::from_utf8(&output.stdout).unwrap(),
                if invalid { &case.mapper_diagnostic } else { "" },
                "{} {phase}: {}",
                case.id,
                output_text(&output)
            );
            assert_eq!(
                output.stderr,
                b"",
                "{} {phase}: {}",
                case.id,
                output_text(&output)
            );
        }
    }
}

#[test]
fn native_cli_unicode_props_keep_complete_valid_invalid_repair_packets() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    for case in cases() {
        let project = project(false);
        for (phase, source, invalid) in [
            ("valid", &case.valid, false),
            ("invalid", &case.invalid, true),
            ("repaired", &case.valid, false),
        ] {
            std::fs::write(project.path().join("src/Case.vue"), source).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_vize"))
                .current_dir(project.path())
                .env("CORSA_PATH", &corsa)
                .args([
                    "check",
                    "--no-config",
                    "--tsconfig",
                    "tsconfig.json",
                    "--format",
                    "json",
                    "--quiet",
                ])
                .output()
                .unwrap();
            let report: Value = serde_json::from_slice(&output.stdout).unwrap();
            capture(
                "cli",
                &case.id,
                phase,
                &json!({ "source": source, "status": output.status.code(), "stdout": std::str::from_utf8(&output.stdout).unwrap(), "stderr": std::str::from_utf8(&output.stderr).unwrap(), "report": report }),
            );
            assert_eq!(
                output.status.code(),
                Some(if invalid { 1 } else { 0 }),
                "{} {phase}: {}",
                case.id,
                output_text(&output)
            );
            assert_eq!(
                output.stderr,
                b"",
                "{} {phase}: {}",
                case.id,
                output_text(&output)
            );
            assert_eq!(report["errorCount"], if invalid { 1 } else { 0 });
            assert_eq!(report["warningCount"], 0);
            assert_eq!(report["fileCount"], 1);
            let diagnostics: Vec<&str> = if invalid {
                vec![case.cli_diagnostic.as_str()]
            } else {
                vec![]
            };
            assert_eq!(
                report["files"],
                json!([{ "file": "src/Case.vue", "diagnostics": diagnostics }]),
                "{} {phase}: {report}",
                case.id
            );
        }
    }
}

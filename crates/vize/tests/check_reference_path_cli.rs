//! Execute the complete #7832 project through the source-built production CLI.
#![expect(clippy::unwrap_used, reason = "fixture assertions panic")]

use std::path::Path;
use std::process::Command;

#[path = "support/corsa_path.rs"]
mod corsa_path;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "../../vize_canon/tests/support/reference_path_project.rs"]
mod project;

#[test]
fn reference_path_modules_keep_complete_cli_diagnostics() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Some(backend) = corsa_requirement::required_or_skip(corsa_path::resolve(&workspace)) else {
        return;
    };
    let cli_digest = std::env::var_os("VIZE_REFERENCE_PATH_CAPTURE_DIR")
        .map(|_| project::digest(&std::fs::read(env!("CARGO_BIN_EXE_vize")).unwrap()));
    for case in project::CASES {
        for invalid in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().canonicalize().unwrap();
            let paths = project::prepare(&root, case, invalid);
            let originals = paths
                .iter()
                .map(|path| std::fs::read(path).unwrap())
                .collect::<Vec<_>>();
            let output = Command::new(env!("CARGO_BIN_EXE_vize"))
                .current_dir(&root)
                .env("CORSA_PATH", &backend)
                .args(["check", "--format", "json"])
                .output()
                .unwrap();
            let report: serde_json::Value = serde_json::from_slice(&output.stdout)
                .unwrap_or_else(|error| panic!("{error}: {output:?}"));
            if let Some(output_dir) = std::env::var_os("VIZE_REFERENCE_PATH_CAPTURE_DIR") {
                let output_dir = std::path::PathBuf::from(output_dir);
                std::fs::create_dir_all(&output_dir).unwrap();
                let record = serde_json::json!({
                    "sourceSha":std::env::var("SOURCE_SHA").unwrap(),
                    "cli":env!("CARGO_BIN_EXE_vize"),"cliSha256":cli_digest,"backend":backend,
                    "case":case.0,"invalid":invalid,"argv":["check","--format","json"],
                    "exitCode":output.status.code(),"stdout":output.stdout,"stderr":output.stderr,
                    "report":report
                });
                std::fs::write(
                    output_dir.join(vize_l0::cstr!("{}-{invalid}-cli.json", case.0).as_str()),
                    serde_json::to_vec_pretty(&record).unwrap(),
                )
                .unwrap();
            }
            let files = report["files"].as_array().unwrap();
            let diagnostics = files
                .iter()
                .flat_map(|file| file["diagnostics"].as_array().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(
                serde_json::json!(diagnostics),
                if invalid {
                    serde_json::json!([
                        "error:2:14 [TS2322] Type 'string' is not assignable to type 'number'."
                    ])
                } else {
                    serde_json::json!([])
                },
                "{}: {report}, stderr={:?}",
                case.0,
                output.stderr
            );
            assert_eq!(output.status.code(), Some(i32::from(invalid)), "{output:?}");
            assert_eq!(report["errorCount"], usize::from(invalid));
            assert_eq!(report["warningCount"], 0);
            let reported_paths = if case.2 {
                &paths[4..]
            } else {
                &[paths[1].clone(), paths[4].clone()]
            };
            assert_eq!(files.len(), reported_paths.len());
            for path in reported_paths {
                assert!(
                    files
                        .iter()
                        .any(|file| root.join(file["file"].as_str().unwrap()) == *path),
                    "missing {path:?}: {report}"
                );
            }
            assert_eq!(report["fileCount"], files.len());
            let programs = report["programs"].as_array().unwrap();
            assert_eq!(programs.len(), 1);
            assert_eq!(programs[0]["root"], ".");
            assert_eq!(programs[0]["tsconfig"], "tsconfig.json");
            assert_eq!(programs[0]["compilerOptions"]["strict"], true);
            for (path, original) in paths.iter().zip(originals) {
                assert_eq!(std::fs::read(path).unwrap(), original);
            }
        }
    }
}

//! Authored TSGO oracle; the existing native File refusal stays explicit.
#![expect(clippy::unwrap_used, reason = "fixture assertions panic")]

use crate::corsa_bridge::{CorsaBridge, CorsaBridgeConfig};
use crate::{BatchTypeChecker, BatchTypeCheckerTrait};
use corsa::runtime::block_on;
use lsp_types::{DocumentDiagnosticReport, DocumentDiagnosticReportResult};
use std::path::Path;
use vize_l0::{Allocator, SourceRoot};
use vize_l1::embed::{
    EmbedSource, Lang,
    syntax::{ProgramOptions, parse_program_once},
};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};
use vize_l4::targets::ts::{ProjectionError, project_program};

#[path = "../support/reference_path_project.rs"]
mod project;

#[test]
fn typed_import_binding_native_file_coverage_remains_unfinished() {
    let arena = Allocator::default();
    let source = project::SOURCE;
    let block = SourceRoot::new(source).unwrap().whole_block();
    let syntax = parse_program_once(
        &arena,
        EmbedSource::authored(source, block.span()).unwrap(),
        ProgramOptions::module(Lang::Ts),
    );
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(syntax.admitted_program().unwrap(), block, 17).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    let original = producer.finish().unwrap();
    assert_eq!(original.artifact().source(), source);
    assert!(!original.is_complete(), "{:?}", original.issues());
    assert!(matches!(
        project_program(&original),
        Err(ProjectionError::IncompleteFile)
    ));
}

#[test]
fn path_references_retain_original_sources_native_reports_and_configuration() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let backend = vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
            project_root: Some(&workspace),
        },
    )
    .unwrap();
    for case in project::CASES {
        for invalid in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().canonicalize().unwrap();
            let paths = project::prepare(&root, case, invalid);
            let originals = paths
                .iter()
                .map(|path| std::fs::read(path).unwrap())
                .collect::<Vec<_>>();
            let source_path = root.join("src/a.ts");
            let source = std::fs::read_to_string(&source_path).unwrap();
            let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
                corsa_path: Some(backend.clone()),
                working_dir: Some(root.clone()),
                ..Default::default()
            });
            block_on(bridge.spawn()).unwrap();
            let source_uri = crate::file_uri::path_to_file_uri(&source_path);
            let uri = source_uri.clone();
            let config = root.join("tsconfig.json");
            let authored = vize_l0::String::from(source.as_str());
            // Read-only authored overlay through the existing configured TSGO
            // request. This is an oracle, never a native File projection.
            let (report, configuration, configuration_path, custody) =
                block_on(bridge.with_client(move |client| {
                    Ok(client.original_program_diagnostics(&uri, &authored, &config, false))
                }))
                .unwrap()
                .unwrap();
            assert!(!custody.session().session_id.is_empty());
            assert_ne!(custody.before().handle(), custody.after().handle());
            for observed in [custody.before(), custody.after()] {
                assert_eq!(observed.project().compiler_options, configuration.options);
                assert_eq!(
                    Path::new(&observed.project().config_file_name),
                    configuration_path
                );
                assert!(
                    observed
                        .projects()
                        .iter()
                        .any(|project| project.id == observed.project().id)
                );
            }
            assert_eq!(configuration_path, root.join("tsconfig.json"));
            assert_eq!(configuration.options["strict"], true);
            assert_eq!(
                configuration.options["skipLibCheck"]
                    .as_bool()
                    .unwrap_or(false),
                case.4
            );
            let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(full)) =
                &report
            else {
                panic!("complete native report required")
            };
            let expected = if invalid {
                serde_json::json!([{
                    "range":{"start":{"line":1,"character":13},"end":{"line":1,"character":17}},
                    "severity":1,"code":2322,"source":"ts","message":"Type 'string' is not assignable to type 'number'."
                }])
            } else {
                serde_json::json!([])
            };
            assert_eq!(
                serde_json::to_value(&full.full_document_diagnostic_report.items).unwrap(),
                expected,
                "{}: full authored TSGO vector, invalid={invalid}",
                case.0
            );
            block_on(bridge.shutdown()).unwrap();
            let mut checker = BatchTypeChecker::new(&root).unwrap();
            checker
                .scan_paths(&[paths[1].clone(), paths[4].clone()])
                .unwrap();
            let checked = checker.check_project().unwrap();
            let diagnostics = checked.diagnostics.iter().map(|diagnostic| serde_json::json!({
                "file":diagnostic.file.strip_prefix(&root).unwrap(),"line":diagnostic.line,"column":diagnostic.column,
                "severity":diagnostic.severity,"code":diagnostic.code,"message":diagnostic.message.as_str()
            })).collect::<Vec<_>>();
            if let Some(output) = std::env::var_os("VIZE_REFERENCE_PATH_CAPTURE_DIR") {
                let output = std::path::PathBuf::from(output);
                std::fs::create_dir_all(&output).unwrap();
                let observation = |snapshot: &crate::DiagnosingSnapshot| {
                    serde_json::json!({
                        "handle":snapshot.handle(),"projects":snapshot.projects(),"changes":snapshot.changes(),"project":snapshot.project()
                    })
                };
                let inputs = paths.iter().zip(&originals).map(|(path, bytes)| serde_json::json!({
                    "file":path.strip_prefix(&root).unwrap(),"source":std::str::from_utf8(bytes).unwrap(),"sha256":project::digest(bytes).as_str()
                })).collect::<Vec<_>>();
                let environment = checker
                    .virtual_files()
                    .into_iter()
                    .find(|file| file.original_path == paths[1])
                    .unwrap();
                let mut virtual_root = environment.virtual_path.as_path();
                for _ in paths[1].strip_prefix(&root).unwrap().components() {
                    virtual_root = virtual_root.parent().unwrap();
                }
                let native_cli = |directory: &Path| {
                    let output = std::process::Command::new(&backend)
                        .current_dir(directory)
                        .args(["--checkers", "1", "--pretty", "false", "--project"])
                        .arg(directory.join("tsconfig.json"))
                        .arg("--listFiles")
                        .output()
                        .unwrap();
                    serde_json::json!({"backend":backend,"directory":directory,
                        "config":std::fs::read_to_string(directory.join("tsconfig.json")).unwrap(),
                        "exitCode":output.status.code(),"stdout":output.stdout,"stderr":output.stderr})
                };
                let authored_cli = native_cli(&root);
                let mirrored_cli = native_cli(virtual_root);
                let record = serde_json::json!({
                    "sourceSha":std::env::var("SOURCE_SHA").unwrap(),"testBinary":std::env::current_exe().unwrap(),
                    "case":case.0,"invalid":invalid,"inputs":inputs,
                    "nativeCliPrograms":{"authored":authored_cli,"mirrored":mirrored_cli},
                    "nativeFileProjection":"unfinished: typed imported const annotation refused",
                    "authoredTsgo":{"sourcePath":source_path,"sourceUri":source_uri,"sourceDigest":project::digest(source.as_bytes()).as_str(),
                        "report":report,"configuration":configuration,"configurationPath":configuration_path,
                        "session":custody.session(),"before":observation(custody.before()),"after":observation(custody.after())},
                    "batch":{"diagnostics":diagnostics,"success":checked.success,"exitCode":checked.exit_code,
                        "virtualFiles":checker.virtual_files().iter().map(|file| serde_json::json!({"originalPath":file.original_path,"virtualPath":file.virtual_path,"source":file.content.as_str()})).collect::<Vec<_>>()}
                });
                std::fs::write(
                    output.join(vize_l0::cstr!("{}-{invalid}-native.json", case.0).as_str()),
                    serde_json::to_vec_pretty(&record).unwrap(),
                )
                .unwrap();
                assert_eq!(authored_cli["exitCode"], i32::from(invalid));
                assert_eq!(mirrored_cli["exitCode"], i32::from(invalid));
            }
            assert_eq!(
                serde_json::json!(diagnostics),
                if invalid {
                    serde_json::json!([{
                        "file":"src/a.ts","line":1,"column":13,"severity":1,"code":2322,"message":"Type 'string' is not assignable to type 'number'."
                    }])
                } else {
                    serde_json::json!([])
                }
            );
            assert_eq!(checked.success, !invalid);
            for (path, original) in paths.iter().zip(originals) {
                assert_eq!(std::fs::read(path).unwrap(), original);
            }
        }
    }
}

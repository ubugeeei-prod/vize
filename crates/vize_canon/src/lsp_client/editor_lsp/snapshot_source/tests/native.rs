//! Full original stdlib bytes versus the same acknowledged snapshot's root text.

use super::{
    super::{SnapshotSourceOwner, SourceTextOutcome, SourceTextRefusal},
    hash,
};
use crate::{file_uri::path_to_file_uri, lsp_client::editor_lsp::EditorLspSession};
use serde_json::json;
use std::{path::Path, process::Command};

#[test]
#[ignore = "requires the locked native 7.0.2 runtime; Actions qualification only"]
fn actual_native_compiler_kind_reads_match_whole_ts_and_generated_vue_pull_packets() {
    use crate::{
        batch::{ImportRewriter, generate_vue_document_virtual_ts},
        lsp_client::initial_diagnostics::native_diagnostics,
        virtual_ts::VirtualTsOptions,
    };
    use corsa::runtime::block_on;
    use lsp_types::{DocumentDiagnosticReport, DocumentDiagnosticReportResult};

    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let executable = vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
            project_root: Some(repo),
        },
    )
    .unwrap()
    .canonicalize()
    .unwrap();
    let version = Command::new(&executable).arg("--version").output().unwrap();
    assert!(version.status.success());
    assert_eq!(version.stdout, b"Version 7.0.2\n");
    assert_eq!(version.stderr, b"");
    // Real dependency lookup remains under the authenticated repository install.
    // No synthetic package links, replacement SDK or hand-authored generated TS.
    let parent = repo.join("tests/target/vize-tests");
    std::fs::create_dir_all(&parent).unwrap();
    let directory = tempfile::tempdir_in(&parent).unwrap();
    let root = directory.path().canonicalize().unwrap();
    let config = root.join("tsconfig.json");
    let config_bytes = br#"{"compilerOptions":{"strict":true,"target":"ESNext","module":"ESNext","moduleResolution":"Bundler","types":[],"declaration":true,"noEmit":true},"files":["source.ts","View.vue.ts"]}"#;
    std::fs::write(&config, config_bytes).unwrap();
    let script = "/*😀\u{2028}\u{2029}*/ export const value: string = 1;\r\n";
    let source = root.join("source.ts");
    let host = root.join("View.vue");
    let authored =
        "<script lang=\"ts\">\n/*😀\u{2028}\u{2029}*/ export const value: string = 1;\n</script>";
    let generated = generate_vue_document_virtual_ts(
        &host,
        authored,
        &VirtualTsOptions::default(),
        &ImportRewriter::new(),
        false,
    )
    .unwrap();
    std::fs::write(&source, "export const value: number = 1;\r\n").unwrap();
    std::fs::write(&host, authored).unwrap();
    let virtual_path = root.join("View.vue.ts");
    std::fs::write(&virtual_path, generated.code.as_bytes()).unwrap();
    let disk = std::fs::read(&source).unwrap();
    let mut editor = EditorLspSession::spawn_with_config(
        executable.to_str().unwrap(),
        &root,
        &root,
        Some(&config),
    )
    .unwrap();
    for (path, text) in [(&source, script), (&virtual_path, generated.code.as_str())] {
        let uri = path_to_file_uri(path);
        editor.mirror(&uri, text).unwrap();
    }
    editor.ready_workspace_request().unwrap();
    let client = editor.configured_api.as_ref().unwrap().client.clone();
    let snapshot = block_on(client.update_snapshot(Default::default())).unwrap();
    let [project] = snapshot.projects.as_slice() else {
        panic!("one configured native project required")
    };
    assert_eq!(Path::new(&project.config_file_name), config);
    let capture =
        std::path::PathBuf::from(std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR").unwrap())
            .join("compiler-kinds");
    std::fs::create_dir_all(&capture).unwrap();
    std::fs::write(capture.join("source.disk.ts"), &disk).unwrap();
    std::fs::write(capture.join("source.overlay.ts"), script).unwrap();
    std::fs::write(capture.join("View.vue"), authored).unwrap();
    std::fs::write(capture.join("View.vue.ts"), generated.code.as_bytes()).unwrap();
    std::fs::write(capture.join("tsconfig.json"), config_bytes).unwrap();
    let binary_sha = hash(&std::fs::read(&executable).unwrap());
    let package = executable
        .ancestors()
        .find(|path| path.join("package.json").is_file())
        .unwrap();
    let package_bytes = std::fs::read(package.join("package.json")).unwrap();
    let package_json: serde_json::Value = serde_json::from_slice(&package_bytes).unwrap();
    assert_eq!(package_json["name"], "@typescript/typescript-linux-x64");
    assert_eq!(package_json["version"], "7.0.2");
    std::fs::write(
        capture.join("provider.json"),
        serde_json::to_vec_pretty(&json!({
            "runtimePath":executable,"runtimeSha256":binary_sha,
            "versionStdout":version.stdout,"versionStderr":version.stderr,
            "package":package_json,"packageSha256":hash(&package_bytes),
        }))
        .unwrap(),
    )
    .unwrap();
    for (path, text) in [(&source, script), (&virtual_path, generated.code.as_str())] {
        assert_eq!(text.match_indices("/*😀\u{2028}\u{2029}*/").count(), 1);
        let uri = path_to_file_uri(path);
        let actual = native_diagnostics::read(&client, &snapshot, project, &uri)
            .unwrap()
            .expect("real lower compiler methods must complete, not fall through");
        let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(legacy)) =
            editor.diagnostics(&uri).unwrap()
        else {
            panic!("whole native pull packet required")
        };
        let evidence = json!({"uri":uri,"snapshot":snapshot.handle,"project":project,
            "runtimePath":executable,"runtimeSha256":binary_sha,
            "sourceSha256":hash(text.as_bytes()),"lower":actual,
            "nativePull":legacy.full_document_diagnostic_report.items});
        std::fs::write(
            capture.join(
                vize_l0::cstr!("{}.json", path.file_name().unwrap().to_str().unwrap()).as_str(),
            ),
            serde_json::to_vec_pretty(&evidence).unwrap(),
        )
        .unwrap();
        assert_eq!(actual, legacy.full_document_diagnostic_report.items);
        let packet = |marker: &str, offset: usize, width: usize, code, severity, message: &str| {
            let positions = text
                .match_indices(marker)
                .map(|(offset, _)| offset)
                .collect::<Vec<_>>();
            let [position] = positions.as_slice() else {
                panic!("exactly one authored diagnostic marker required: {marker}")
            };
            let prefix = text.get(..position + offset).unwrap();
            let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
            let column = prefix
                .rsplit_once('\n')
                .map_or(prefix, |(_, last)| last)
                .encode_utf16()
                .count();
            json!({"range":{"start":{"line":line,"character":column},
                "end":{"line":line,"character":column+width}},
                "severity":severity,"code":code,"source":"ts","message":message})
        };
        let mut expected = vec![packet(
            "value: string",
            0,
            5,
            2322,
            1,
            "Type 'number' is not assignable to type 'string'.",
        )];
        if path == &virtual_path {
            // Native suggestion diagnostics include these four genuine unused
            // Canon helper declarations, in source order. Retain the whole set.
            for name in [
                "__Ref",
                "__VizePrettify",
                "__VizeComponentFallthroughProps",
                "__VizeComponentMissingInputGuard",
            ] {
                let marker = vize_l0::cstr!("type {name}<");
                let message = vize_l0::cstr!("'{name}' is declared but never used.");
                expected.push(packet(
                    &marker,
                    5,
                    name.encode_utf16().count(),
                    6196,
                    4,
                    &message,
                ));
            }
        }
        assert_eq!(serde_json::to_value(&actual).unwrap(), json!(expected));
    }
    block_on(snapshot.release()).unwrap();
    assert_eq!(std::fs::read(&source).unwrap(), disk);
    assert_eq!(std::fs::read(&host).unwrap(), authored.as_bytes());
    assert_eq!(
        std::fs::read(&virtual_path).unwrap(),
        generated.code.as_bytes()
    );
    assert_eq!(std::fs::read(&config).unwrap(), config_bytes);
}

#[test]
#[ignore = "requires the locked native 7.0.2 runtime; existing native-phase Actions executes it"]
fn actual_native_snapshot_retains_complete_stdlib_and_overlay_text_with_owner_refusals() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let executable = vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
            project_root: Some(repo),
        },
    )
    .unwrap()
    .canonicalize()
    .unwrap();
    let version = Command::new(&executable).arg("--version").output().unwrap();
    let binary_sha = hash(&std::fs::read(&executable).unwrap());
    assert!(version.status.success());
    assert_eq!(version.stdout, b"Version 7.0.2\n");
    assert_eq!(version.stderr, b"");
    let package = executable
        .ancestors()
        .find(|directory| directory.join("package.json").is_file())
        .unwrap();
    let package_bytes = std::fs::read(package.join("package.json")).unwrap();
    let metadata: serde_json::Value = serde_json::from_slice(&package_bytes).unwrap();
    assert_eq!(metadata["version"], "7.0.2");
    assert_eq!(metadata["name"], "@typescript/typescript-linux-x64");

    // Independent physical input oracles are read BEFORE creating any snapshot.
    // The provider itself never opens these paths, before or after its request.
    let mut libraries = Vec::new();
    for name in ["lib.es5.d.ts", "lib.es2017.object.d.ts"] {
        let paths = walkdir::WalkDir::new(package)
            .into_iter()
            .map(Result::unwrap)
            .filter(|entry| entry.file_type().is_file() && entry.file_name() == name)
            .map(|entry| entry.path().to_path_buf())
            .collect::<Vec<_>>();
        assert_eq!(paths.len(), 1);
        libraries.push((name, paths[0].clone(), std::fs::read(&paths[0]).unwrap()));
    }
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("tsconfig.json");
    let config_bytes = br#"{"compilerOptions":{"strict":true,"target":"ESNext","module":"ESNext","moduleResolution":"Bundler","moduleDetection":"force","types":[],"noEmit":true},"files":["source.ts"]}"#;
    let source = root.path().join("source.ts");
    let disk_source = b"export const original: number = 1;\n";
    let overlay = "export const emoji = '😀';\r\nexport const nullable: object | null = null; Object.keys(nullable);\r\n";
    std::fs::write(&config, config_bytes).unwrap();
    std::fs::write(&source, disk_source).unwrap();
    let foreign = tempfile::tempdir().unwrap();
    let foreign_source = foreign.path().join("source.ts");
    let foreign_config = foreign.path().join("tsconfig.json");
    std::fs::write(&foreign_source, disk_source).unwrap();
    std::fs::write(&foreign_config, config_bytes).unwrap();
    let capture = std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap();
    std::fs::create_dir_all(&capture).unwrap();
    std::fs::write(capture.join("source.disk.ts"), disk_source).unwrap();
    std::fs::write(capture.join("source.overlay.ts"), overlay).unwrap();
    std::fs::write(capture.join("tsconfig.json"), config_bytes).unwrap();
    let mut editor = EditorLspSession::spawn_with_config(
        executable.to_str().unwrap(),
        root.path(),
        root.path(),
        Some(&config),
    )
    .unwrap();
    let uri = path_to_file_uri(&source);
    editor.mirror(&uri, overlay).unwrap();
    editor.ready_workspace_request().unwrap();
    let api = &editor.configured_api.as_ref().unwrap().client;
    let mut owner = SnapshotSourceOwner::create(api).unwrap();
    let mut rows = Vec::new();
    let mut refusals = Vec::new();
    {
        let project = owner.project(&config).unwrap().unwrap();
        refusals.push(matches!(
            owner.project(&foreign_config).unwrap(),
            Err(SourceTextRefusal::ProjectIdentity)
        ));
        refusals.push(matches!(
            project.read(&path_to_file_uri(&foreign_source)).unwrap(),
            SourceTextOutcome::Refused {
                reason: SourceTextRefusal::SourceIdentity,
                encoded: None
            }
        ));
        let absent = path_to_file_uri(&root.path().join("absent.ts"));
        refusals.push(matches!(
            project.read(&absent).unwrap(),
            SourceTextOutcome::Refused {
                reason: SourceTextRefusal::SourceIdentity,
                encoded: None
            }
        ));
        for (name, path, oracle) in libraries
            .iter()
            .map(|(name, path, bytes)| (*name, path, bytes.as_slice()))
            .chain(std::iter::once(("source.ts", &source, overlay.as_bytes())))
        {
            let source_uri = path_to_file_uri(path);
            let outcome = project.read(&source_uri).unwrap();
            std::fs::write(capture.join(format!("{name}.oracle")), oracle).unwrap();
            match outcome {
                SourceTextOutcome::Complete(view) => {
                    std::fs::write(
                        capture.join(format!("{name}.encoded.bin")),
                        view.encoded_bytes(),
                    )
                    .unwrap();
                    std::fs::write(capture.join(format!("{name}.decoded")), view.text()).unwrap();
                    rows.push(json!({
                        "name":name,"sourceUri":source_uri,"snapshot":view.snapshot_handle(),
                        "project":view.project_descriptor(),"fileName":view.file_name(),
                        "path":view.path(),"oracleSha256":hash(oracle),
                        "encodedSha256":hash(view.encoded_bytes()),"decodedSha256":hash(view.text().as_bytes()),
                        "bytesEqual":view.text().as_bytes()==oracle,
                        "identityEqual":view.file_name()==path.to_str().unwrap() && view.path()==path.to_str().unwrap(),
                    }));
                    // Real stdlib bytes also exercise malformed and foreign roots.
                    let mut malformed = view.encoded_bytes().to_vec();
                    malformed[3] = 4;
                    std::fs::write(capture.join(format!("{name}.unsupported.bin")), &malformed)
                        .unwrap();
                    refusals.push(
                        super::super::protocol::source_text(&malformed, view.file_name())
                            == Err(SourceTextRefusal::Version),
                    );
                    refusals.push(
                        super::super::protocol::source_text(
                            view.encoded_bytes(),
                            foreign_source.to_str().unwrap(),
                        ) == Err(SourceTextRefusal::SourceIdentity),
                    );
                }
                SourceTextOutcome::Refused { reason, encoded } => {
                    if let Some(encoded) = encoded {
                        std::fs::write(
                            capture.join(format!("{name}.refused.bin")),
                            encoded.as_bytes(),
                        )
                        .unwrap();
                    }
                    rows.push(
                        json!({"name":name,"sourceUri":source_uri,"refusal":format!("{reason:?}")}),
                    );
                }
            }
        }
        std::fs::write(
            capture.join("whole-snapshot-source.json"),
            serde_json::to_vec_pretty(&json!({
                "sourceSha":std::env::var("SOURCE_SHA").ok(),"nativeBinary":executable,
                "nativeBinarySha256":binary_sha,
                "nativePackage":metadata,"nativePackageSha256":hash(&package_bytes),
                "snapshot":owner.handle(),"project":project.descriptor(),"rows":rows,
                "sourceFileNames":project.source_names(),
                "foreignConfig":foreign_config,"foreignSource":foreign_source,
                "refusalControls":["foreign-config","foreign-source","absent-source","unsupported-version","foreign-root"],
                "refusals":refusals,
            }))
            .unwrap(),
        )
        .unwrap();
    }
    // All full sides and opaque bytes are saved before qualification assertions.
    owner.release().unwrap();
    assert!(matches!(
        owner.project(&config).unwrap(),
        Err(SourceTextRefusal::ReleasedSnapshot)
    ));
    drop(owner);
    editor.shutdown().unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(refusals, [true; 9]);
    for row in &rows {
        assert_eq!(row["bytesEqual"], true, "{row}");
        assert_eq!(row["identityEqual"], true, "{row}");
    }
    assert_eq!(std::fs::read(&source).unwrap(), disk_source);
    assert_eq!(std::fs::read(&config).unwrap(), config_bytes);
}

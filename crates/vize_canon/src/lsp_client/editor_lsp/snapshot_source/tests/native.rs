//! Full original stdlib bytes versus the same acknowledged snapshot's root text.

use super::super::{SnapshotSourceOwner, SourceTextOutcome, SourceTextRefusal};
use crate::{file_uri::path_to_file_uri, lsp_client::editor_lsp::EditorLspSession};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{path::Path, process::Command};

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

fn hash(bytes: &[u8]) -> vize_l0::String {
    vize_l0::cstr!("{:x}", Sha256::digest(bytes))
}

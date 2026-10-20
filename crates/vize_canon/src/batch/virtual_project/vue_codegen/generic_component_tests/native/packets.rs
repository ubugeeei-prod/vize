//! Complete immutable blobs avoid duplicating large programs across controls.

use super::super::{ROOT_PATH, STORY_PATH};
use crate::{
    batch::virtual_project::VirtualFile,
    corsa_client::{CorsaProjectClient, LspDiagnostic},
    file_uri::{file_uri_to_path, path_to_file_uri},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use vize_carton::{String, cstr};

#[derive(Clone, Copy)]
pub(super) struct Replay<'a> {
    pub(super) path: &'a Path,
    pub(super) code: &'a str,
    pub(super) scope: &'a str,
}

pub(super) type Row = (String, u32, u32, Option<u32>, Option<i32>, String);

fn directory() -> Option<PathBuf> {
    std::env::var_os("VIZE_NATIVE_SYNTAX_CAPTURE_DIR")
        .or_else(|| std::env::var_os("VIZE_TEST_FIX_HISTORY_CAPTURE_DIR"))
        .map(|directory| PathBuf::from(directory).join("generic-component"))
}

pub(super) fn retain(phase: &str, packet: &Value) {
    if let Some(directory) = directory() {
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join(cstr!("{phase}.json").as_str()),
            serde_json::to_vec_pretty(packet).unwrap(),
        )
        .unwrap();
    }
}

pub(super) fn blob(bytes: &[u8]) -> Value {
    let hash: String = Sha256::digest(bytes)
        .iter()
        .map(|byte| cstr!("{byte:02x}"))
        .collect();
    if let Some(directory) = directory() {
        let directory = directory.join("bytes");
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join(hash.as_str());
        if path.exists() {
            assert_eq!(fs::read(&path).unwrap(), bytes);
        } else {
            fs::write(&path, bytes).unwrap();
        }
        json!({"sha256":hash,"bytes":bytes.len(),"relativeBlob":cstr!("bytes/{hash}")})
    } else {
        json!({"sha256":hash,"bytes":bytes.len(),"completeUtf8":std::str::from_utf8(bytes).unwrap()})
    }
}

pub(super) fn diagnose(
    root: &Path,
    native: &Path,
    files: &[&VirtualFile],
    case: &str,
    phase: &str,
    replay: Option<Replay<'_>>,
    project_paths: (&Path, &Path),
) -> (Vec<Row>, Value) {
    diagnose_selected(
        root,
        native,
        files,
        case,
        phase,
        replay,
        project_paths,
        &[root.join(ROOT_PATH), root.join(STORY_PATH)],
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "complete native program and replay authority"
)]
pub(super) fn diagnose_selected(
    root: &Path,
    native: &Path,
    files: &[&VirtualFile],
    case: &str,
    phase: &str,
    replay: Option<Replay<'_>>,
    project_paths: (&Path, &Path),
    original_paths: &[PathBuf],
) -> (Vec<Row>, Value) {
    let (mirror, config) = project_paths;
    assert!(
        config.is_file(),
        "the actual materialized config is required: {config:?}"
    );
    let mut client = CorsaProjectClient::new_for_workspace_config(
        Some(native.to_str().unwrap()),
        mirror,
        config,
    )
    .unwrap();
    let requested = files
        .iter()
        .filter(|file| original_paths.contains(&file.original_path))
        .map(|file| path_to_file_uri(&file.virtual_path))
        .collect::<Vec<_>>();
    assert_eq!(
        requested.len(),
        original_paths.len(),
        "every selected complete authored module is required"
    );
    client.begin_batch_test_receipt(true);
    let raw = client.request_diagnostics_batch(&requested).unwrap();
    let authority = client.batch_test_receipt(requested.first().unwrap());
    let phase = cstr!("{case}-{phase}");
    let mut packet = json!({
        "case":case,"phase":phase,"requestedUris":requested,"nativeAuthority":authority,
        "wholeRawDiagnosticVector":raw,
        "wholeGeneratedConfig":fs::read_to_string(config).unwrap(),
        "generatedConfigPath":config,
        "wholeGeneratedPackage":fs::read_to_string(mirror.join("package.json")).ok(),
        "wholeGeneratedProgram":files.iter().map(|file| {
            let code = replay.filter(|replay| replay.path == file.virtual_path).map_or(file.content.as_str(), |replay| replay.code);
            json!({"originalPath":file.original_path,"virtualPath":file.virtual_path,
                "completeCode":blob(code.as_bytes()),
                "completeAuthoredSource":match fs::read(&file.original_path) {
                    Ok(bytes) => blob(&bytes),
                    Err(error) => json!({"observedReadError":cstr!("{error}")}),
                },
                "completeSourceMap":blob(cstr!("{:#?}",file.source_map).as_bytes())})
        }).collect::<Vec<_>>(),
        "replayScope":replay.map(|replay| replay.scope),
        "productionScope":"actual whole-SFC factory and generated config; replay is explicitly separate from unavailable old factory/native execution",
    });
    retain(&phase, &packet);
    let rows = mapped_rows(root, files, &raw, replay);
    packet["wholeMappedDiagnosticVector"] = json!(rows);
    retain(&phase, &packet);
    assert_eq!(authority["selectedConfig"], json!(config), "{authority}");
    assert_eq!(
        authority["project"]["compilerOptions"], authority["normalizedRequestedOptions"],
        "{authority}"
    );
    assert!(
        authority["normalizedRequestedOptions"].is_object(),
        "{authority}"
    );
    assert!(authority["snapshotProjects"].is_array(), "{authority}");
    assert!(
        matches!(
            authority["mode"].as_str(),
            Some("project-api" | "editor-lsp")
        ),
        "{authority}"
    );
    let roots = authority["project"]["rootFiles"].as_array().unwrap();
    for file in files
        .iter()
        .filter(|file| original_paths.contains(&file.original_path))
    {
        assert!(roots.contains(&json!(file.virtual_path)), "{authority}");
    }
    (rows, packet)
}

fn mapped_rows(
    root: &Path,
    files: &[&VirtualFile],
    raw: &[(String, Vec<LspDiagnostic>)],
    replay: Option<Replay<'_>>,
) -> Vec<Row> {
    let mut rows = Vec::new();
    for (uri, diagnostics) in raw {
        let path = file_uri_to_path(uri).unwrap();
        let file = files.iter().find(|file| file.virtual_path == path).unwrap();
        let source = fs::read_to_string(&file.original_path).unwrap();
        let code = replay
            .filter(|replay| replay.path == file.virtual_path)
            .map_or(file.content.as_str(), |replay| replay.code);
        for diagnostic in diagnostics {
            let start = &diagnostic.range.start;
            let Some(offset) =
                crate::batch::source_map::line_col_to_offset(code, start.line, start.character)
            else {
                continue;
            };
            let Some((original, _, _)) = file.source_map.get_original_position(offset) else {
                continue;
            };
            let (line, column) =
                crate::batch::source_map::offset_to_line_col(&source, original).unwrap();
            let code = diagnostic.code.as_ref().and_then(|code| {
                code.as_u64()
                    .and_then(|code| u32::try_from(code).ok())
                    .or_else(|| code.as_str().and_then(|code| code.parse().ok()))
            });
            rows.push((
                String::from(
                    file.original_path
                        .strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap(),
                ),
                line,
                column,
                code,
                diagnostic.severity,
                String::from(diagnostic.message.as_str()),
            ));
        }
    }
    rows.sort();
    rows
}

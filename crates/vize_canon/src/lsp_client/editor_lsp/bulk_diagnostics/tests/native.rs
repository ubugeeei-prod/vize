//! Whole responses from the same pinned native process, before projection.

use super::super::super::EditorLspSession;
use super::super::BulkDiagnostics;
use crate::file_uri::path_to_file_uri;
use serde_json::json;
use std::path::Path;
use vize_l0::String;

#[test]
fn eight_real_native_profiles_match_complete_per_file_lsp_before_and_after_edits() {
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
    .unwrap();
    let fixtures: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/native-bulk-diagnostics/profiles.json"
    )))
    .unwrap();
    let cases = fixtures["profiles"].as_array().unwrap();
    assert_eq!(cases.len(), 8);
    for (index, fixture) in cases.iter().enumerate() {
        let options = &fixture["compilerOptions"];
        let original = fixture["source"].as_str().unwrap();
        let empty = fixture["emptySource"].as_str().unwrap();
        let repaired = fixture["repair"].as_str().unwrap();
        let a_name = fixture["fileName"].as_str().unwrap();
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join("tsconfig.json");
        let mut compiler_options = json!({
            "module":"ESNext", "moduleResolution":"Bundler", "target":"ESNext",
            "moduleDetection":"force", "types":[], "noEmit":true,
        });
        compiler_options
            .as_object_mut()
            .unwrap()
            .extend(options.as_object().unwrap().clone());
        let config_bytes = serde_json::to_vec(
            &json!({"compilerOptions":compiler_options,"files":[a_name,"b.ts"]}),
        )
        .unwrap();
        std::fs::write(&config, &config_bytes).unwrap();
        let a = root.path().join(a_name);
        let b = root.path().join("b.ts");
        std::fs::write(&a, original).unwrap();
        std::fs::write(&b, empty).unwrap();
        let a_uri = path_to_file_uri(&a);
        let b_uri = path_to_file_uri(&b);
        let uris = [a_uri.clone(), b_uri.clone(), a_uri.clone()];
        let mut editor = EditorLspSession::spawn_with_config(
            executable.to_str().unwrap(),
            root.path(),
            root.path(),
            Some(&config),
        )
        .unwrap();
        println!(
            "bulk native profile {index}, binary {}",
            executable.display()
        );
        editor.mirror(&a_uri, original).unwrap();
        editor.mirror(&b_uri, empty).unwrap();
        compare(&mut editor, &config, &uris, index, "initial");
        assert!(matches!(
            editor
                .bulk_diagnostics(&root.path().join("foreign.json"), &uris)
                .unwrap(),
            BulkDiagnostics::Refused
        ));
        assert!(super::super::take_receipt().is_none());
        let absent = path_to_file_uri(&root.path().join("absent.ts"));
        assert!(matches!(
            editor.bulk_diagnostics(&config, &[absent]).unwrap(),
            BulkDiagnostics::Refused
        ));
        assert!(super::super::take_receipt().is_none());
        compare(&mut editor, &config, &uris, index, "unchanged");
        editor.mirror(&a_uri, repaired).unwrap();
        compare(&mut editor, &config, &uris, index, "edited");
        editor.shutdown().unwrap();
        assert_eq!(std::fs::read(&config).unwrap(), config_bytes);
        assert_eq!(std::fs::read(&a).unwrap(), original.as_bytes());
        assert_eq!(std::fs::read(&b).unwrap(), empty.as_bytes());
    }
}

fn compare(
    editor: &mut EditorLspSession,
    config: &Path,
    uris: &[String],
    case: usize,
    phase: &str,
) {
    let outcome = editor.bulk_diagnostics(config, uris).unwrap();
    let status = vize_l0::cstr!("{outcome:?}");
    let bulk = match outcome {
        BulkDiagnostics::Complete(rows) => Some(rows),
        _ => None,
    };
    let custody = super::super::take_receipt();
    let mut comparisons = Vec::new();
    for (index, uri) in uris.iter().enumerate() {
        let original = serde_json::to_value(editor.diagnostics(uri).unwrap()).unwrap();
        let actual = bulk.as_ref().and_then(|rows| rows.get(index));
        comparisons.push(json!({"uri":uri,"bulk":actual,"lsp":original}));
    }
    if let Some(dir) = std::env::var_os("VIZE_NATIVE_BULK_CAPTURE_DIR") {
        let dir = Path::new(&dir).join(vize_l0::cstr!("profile-{case}/{phase}").as_str());
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("whole-diagnostics.json"),
            serde_json::to_vec_pretty(&json!({
                "bulkOutcome":status, "custody":custody,
                "acknowledgedDocuments":editor.documents,
                "configuration":std::fs::read_to_string(config).unwrap(),
                "comparisons":comparisons,
            }))
            .unwrap(),
        )
        .unwrap();
    }
    // Persist every complete side before a failed comparison can abort CI.
    let bulk =
        bulk.unwrap_or_else(|| panic!("profile {case}/{phase} must exercise bulk: {status}"));
    assert_eq!(bulk.len(), uris.len());
    for comparison in comparisons {
        assert_eq!(
            comparison["lsp"]["kind"], "full",
            "{case}/{phase}: {comparison}"
        );
        assert_eq!(
            comparison["bulk"], comparison["lsp"]["items"],
            "{case}/{phase}: {comparison}"
        );
    }
}

mod deep;

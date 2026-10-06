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
        assert_eq!(super::super::take_receipt().unwrap()["outcome"], "refused");
        let absent = path_to_file_uri(&root.path().join("absent.ts"));
        assert!(matches!(
            editor.bulk_diagnostics(&config, &[absent]).unwrap(),
            BulkDiagnostics::Refused
        ));
        assert_eq!(super::super::take_receipt().unwrap()["outcome"], "refused");
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
    let mut packet = json!({
        "acknowledgedDocuments":editor.documents,
        "configuration":std::fs::read_to_string(config).unwrap(),
        "comparisons":[],"originalResults":[],
    });
    let persist = |packet: &serde_json::Value| {
        if let Some(dir) = std::env::var_os("VIZE_NATIVE_BULK_CAPTURE_DIR") {
            let dir = Path::new(&dir).join(vize_l0::cstr!("profile-{case}/{phase}").as_str());
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join("whole-diagnostics.json"),
                serde_json::to_vec_pretty(packet).unwrap(),
            )
            .unwrap();
        }
    };
    let result = editor.bulk_diagnostics(config, uris);
    let status = match &result {
        Ok(outcome) => vize_l0::cstr!("{outcome:?}"),
        Err(error) => vize_l0::cstr!("Err({error:?})"),
    };
    let custody = super::super::take_receipt();
    packet["bulkResult"] = json!(vize_l0::cstr!("{result:?}"));
    packet["bulkOutcome"] = json!(status);
    packet["custody"] = json!(custody);
    persist(&packet);
    let outcome = result.unwrap();
    let bulk = match outcome {
        BulkDiagnostics::Complete(rows) => Some(rows),
        _ => None,
    };
    let mut comparisons = Vec::new();
    for (index, uri) in uris.iter().enumerate() {
        let result = editor.diagnostics(uri);
        packet["originalResults"]
            .as_array_mut()
            .unwrap()
            .push(json!({"uri":uri,"result":result}));
        persist(&packet);
        let original = serde_json::to_value(result.unwrap()).unwrap();
        let actual = bulk.as_ref().and_then(|rows| rows.get(index));
        comparisons.push(json!({"uri":uri,"bulk":actual,"lsp":original}));
        packet["comparisons"] = json!(comparisons);
        persist(&packet);
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
    let configuration: serde_json::Value =
        serde_json::from_slice(&std::fs::read(config).unwrap()).unwrap();
    let mut expected = vec![
        json!({"method":"getSyntacticDiagnostics","file":null,"acknowledged":true}),
        json!({"method":"getSemanticDiagnostics","file":null,"acknowledged":true}),
        json!({"method":"getSuggestionDiagnostics","file":null,"acknowledged":true}),
    ];
    if configuration["compilerOptions"]["declaration"] == true
        || configuration["compilerOptions"]["composite"] == true
    {
        expected
            .push(json!({"method":"getDeclarationDiagnostics","file":null,"acknowledged":true}));
    }
    let custody = custody.unwrap();
    assert_eq!(custody["categoryRequests"], json!(expected));
    let responses = custody["categoryResponses"].as_array().unwrap();
    assert_eq!(responses.len(), expected.len());
    for (response, request) in responses.iter().zip(&expected) {
        assert_eq!(response["method"], request["method"]);
        assert_eq!(response["file"], serde_json::Value::Null);
        assert!(response.get("value").is_some());
        assert!(response.get("error").is_none());
    }
}

mod deep;

mod selected;

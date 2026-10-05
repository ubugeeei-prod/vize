//! Original selected diagnostics exclude an independently erroneous project file.

use super::{BulkDiagnostics, EditorLspSession, Path, json, path_to_file_uri};

#[test]
fn selected_native_semantics_preserves_dependency_related_rows_and_unrequested_scope() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let executable = vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
            project_root: Some(&repo),
        },
    )
    .unwrap();
    let originals = repo.join(
        "tests/_fixtures/differential/typechecker/native-bulk-diagnostics/selected-semantics",
    );
    let root = tempfile::tempdir().unwrap();
    for name in [
        "tsconfig.json",
        "requested.ts",
        "empty.ts",
        "dependency.ts",
        "unrequested.ts",
    ] {
        std::fs::copy(originals.join(name), root.path().join(name)).unwrap();
    }
    let config = root.path().join("tsconfig.json");
    let requested = root.path().join("requested.ts");
    let empty = root.path().join("empty.ts");
    let requested_uri = path_to_file_uri(&requested);
    let empty_uri = path_to_file_uri(&empty);
    let uris = [
        requested_uri.clone(),
        empty_uri.clone(),
        requested_uri.clone(),
    ];
    let mut editor = EditorLspSession::spawn_with_config(
        executable.to_str().unwrap(),
        root.path(),
        root.path(),
        Some(&config),
    )
    .unwrap();
    editor
        .mirror(
            &requested_uri,
            &std::fs::read_to_string(&requested).unwrap(),
        )
        .unwrap();
    editor
        .mirror(&empty_uri, &std::fs::read_to_string(&empty).unwrap())
        .unwrap();
    let original_files = [
        "tsconfig.json",
        "requested.ts",
        "empty.ts",
        "dependency.ts",
        "unrequested.ts",
    ]
    .map(|name| json!({"name":name,"bytes":std::fs::read(originals.join(name)).unwrap()}));
    let mut packet = json!({"documents":editor.documents,"nativeBinary":executable,
        "originalResults":[],"originalFiles":original_files});
    let persist = |packet: &serde_json::Value| {
        if let Some(dir) = std::env::var_os("VIZE_NATIVE_BULK_CAPTURE_DIR") {
            let dir = Path::new(&dir).join("selected-semantics");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join("whole-diagnostics.json"),
                serde_json::to_vec_pretty(packet).unwrap(),
            )
            .unwrap();
        }
    };
    let bulk_result = editor.bulk_diagnostics(&config, &uris);
    packet["bulkResult"] = json!(vize_l0::cstr!("{bulk_result:?}"));
    packet["custody"] = json!(super::super::super::take_receipt());
    packet["bulk"] = match &bulk_result {
        Ok(BulkDiagnostics::Complete(rows)) => serde_json::to_value(rows).unwrap(),
        _ => serde_json::Value::Null,
    };
    persist(&packet);
    let outcome = bulk_result.unwrap();
    let mut original = Vec::new();
    for uri in &uris {
        let result = editor.diagnostics(uri);
        packet["originalResults"]
            .as_array_mut()
            .unwrap()
            .push(json!({"uri":uri,"result":result}));
        persist(&packet);
        original.push(serde_json::to_value(result.unwrap()).unwrap());
    }
    let BulkDiagnostics::Complete(rows) = outcome else {
        panic!("selected native scope refused: {packet}");
    };
    assert_eq!(rows.len(), 3);
    for (row, lsp) in rows.iter().zip(&original) {
        assert_eq!(serde_json::to_value(row).unwrap(), lsp["items"]);
        assert_eq!(lsp["kind"], "full");
    }
    assert_eq!(rows[0], rows[2]);
    assert!(rows[1].is_empty());
    assert!(!rows[0].is_empty());
    let dependency_uri = path_to_file_uri(&root.path().join("dependency.ts"));
    assert!(rows[0].iter().any(
        |row| row.related_information.as_ref().is_some_and(|related| {
            related
                .iter()
                .any(|info| info.location.uri.as_str() == dependency_uri.as_str())
        })
    ));
    assert_eq!(
        packet["custody"]["categoryRequests"],
        json!([
            {"method":"getSyntacticDiagnostics","file":null,"acknowledged":true},
            {"method":"getSemanticDiagnostics","file":requested.to_str().unwrap(),"acknowledged":true},
            {"method":"getSemanticDiagnostics","file":empty.to_str().unwrap(),"acknowledged":true},
            {"method":"getSuggestionDiagnostics","file":null,"acknowledged":true},
        ])
    );
    assert!(
        packet["custody"]["sourceFileNames"]
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name == root.path().join("unrequested.ts").to_str().unwrap())
    );
    editor.shutdown().unwrap();
    for name in [
        "tsconfig.json",
        "requested.ts",
        "empty.ts",
        "dependency.ts",
        "unrequested.ts",
    ] {
        assert_eq!(
            std::fs::read(root.path().join(name)).unwrap(),
            std::fs::read(originals.join(name)).unwrap()
        );
    }
}

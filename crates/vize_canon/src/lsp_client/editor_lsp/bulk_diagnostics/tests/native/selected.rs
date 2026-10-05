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
    let outcome = editor.bulk_diagnostics(&config, &uris).unwrap();
    let custody = super::super::super::take_receipt();
    let mut original = Vec::new();
    for uri in &uris {
        original.push(serde_json::to_value(editor.diagnostics(uri).unwrap()).unwrap());
    }
    let bulk_rows = match &outcome {
        BulkDiagnostics::Complete(rows) => serde_json::to_value(rows).unwrap(),
        _ => serde_json::Value::Null,
    };
    let packet = json!({"outcome":vize_l0::cstr!("{outcome:?}"),"bulk":bulk_rows,"custody":custody,
        "original":original,"documents":editor.documents,"nativeBinary":executable,
        "originalFiles":["tsconfig.json","requested.ts","empty.ts","dependency.ts","unrequested.ts"].map(|name| json!({"name":name,"bytes":std::fs::read(originals.join(name)).unwrap()}))});
    if let Some(dir) = std::env::var_os("VIZE_NATIVE_BULK_CAPTURE_DIR") {
        let dir = Path::new(&dir).join("selected-semantics");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("whole-diagnostics.json"),
            serde_json::to_vec_pretty(&packet).unwrap(),
        )
        .unwrap();
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
    let semantic: Vec<_> = packet["custody"]["categoryRequests"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|request| request["method"] == "getSemanticDiagnostics")
        .collect();
    assert_eq!(semantic.len(), 2);
    assert_eq!(semantic[0]["file"], requested.to_str().unwrap());
    assert_eq!(semantic[1]["file"], empty.to_str().unwrap());
    assert!(
        semantic
            .iter()
            .all(|request| request["acknowledged"] == true)
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

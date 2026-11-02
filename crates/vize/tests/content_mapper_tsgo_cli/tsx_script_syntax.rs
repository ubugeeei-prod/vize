//! `.ts` syntax that TSX reads as JSX: lone generic arrows and angle-bracket
//! assertions stay checkable and navigable through the exact host.

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use corsa_lsp::{LspClient, LspSpawnConfig, VirtualDocument, jsonrpc::InboundEvent};
use lsp_types::Uri;
use serde_json::{Value, json};
use vize_l0::{String as CompactString, cstr};

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};
use crate::content_mapper_lsp_support::raw_requests::{
    RawInitialize, RawInitialized, RawSetContentMapperContributions,
};
use crate::content_mapper_lsp_support::{
    EditorResponder, StopOnDrop, contains_text_edit, editor_capabilities, file_uri, hover,
    install_packages, position, pull_diagnostics, rename,
};

const CLEAN: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsx-script-syntax-projection/Clean.vue.txt"
));

const PLAIN: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsx-script-syntax-projection/Plain.vue.txt"
));

const GENERIC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsx-script-syntax-projection/Generic.vue.txt"
));

/// Positions match `@vue/content-mapper`, which projects `.ts`.
const ERRORS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsx-script-syntax-projection/Errors.vue.txt"
));

const CONVERSION: &str = "may be a mistake because neither type sufficiently overlaps with the other. If this was intentional, convert the expression to 'unknown' first.";

#[test]
fn standard_tsgo_checks_ts_only_generic_arrows_and_assertions() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let expected = [
        cstr!(
            "src/Errors.vue(3,7): error TS2322: Type 'number' is not assignable to type 'string'."
        ),
        cstr!(
            "src/Errors.vue(5,9): error TS2339: Property 'missing' does not exist on type 'HTMLElement'."
        ),
        cstr!(
            "src/Errors.vue(6,18): error TS2352: Conversion of type 'number' to type 'string' {CONVERSION}"
        ),
        cstr!(
            "src/Errors.vue(8,15): error TS2352: Conversion of type 'string' to type 'number' {CONVERSION}"
        ),
    ];
    for newline in ["\n", "\r\n"] {
        let project = project("content-mapper-tsx-script-syntax-");
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        write_sources(project.path(), newline);
        let output = check_project(&tsgo, project.path(), "tsconfig.json");
        assert!(output.stderr.is_empty(), "{}", output_text(&output));
        let actual = std::str::from_utf8(&output.stdout)
            .unwrap()
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| CompactString::from(line.replace('\\', "/")))
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{}", output_text(&output));
    }
}

#[test]
fn standard_tsgo_lsp_navigates_rewritten_ts_only_syntax() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper LSP conformance: {TSGO_ENV} is not set");
        return;
    };
    let project = project("content-mapper-lsp-tsx-script-syntax-");
    install_packages(project.path());
    write_sources(project.path(), "\n");
    let path = project.path().join("src/Clean.vue");
    let uri = file_uri(&path);
    let root_uri = file_uri(project.path());
    let at = |anchor: &str, inner: &str| {
        let offset = CLEAN.find(anchor).unwrap() + anchor.find(inner).unwrap();
        (
            position(CLEAN, offset),
            position(CLEAN, offset + inner.len()),
        )
    };

    let stop = AtomicBool::new(false);
    let editor = EditorResponder::default();
    std::thread::scope(|scope| {
        let _stop_on_drop = StopOnDrop(&stop);
        corsa::runtime::block_on(async {
            let client = LspClient::spawn(
                LspSpawnConfig::new(&tsgo)
                    .with_cwd(project.path())
                    .with_request_timeout(Some(Duration::from_secs(30))),
            )
            .await
            .unwrap();
            let responder_client = client.clone();
            let events = responder_client.subscribe();
            let (stop_ref, editor_ref) = (&stop, &editor);
            let responder = scope.spawn(move || {
                while !stop_ref.load(Ordering::Relaxed) {
                    if let Ok(InboundEvent::Request { id, method, params }) =
                        events.recv_timeout(Duration::from_millis(50))
                    {
                        let result = editor_ref.respond_to(method.as_str(), &params);
                        let _ = responder_client.respond(id, result);
                    }
                }
            });
            client
                .request::<RawInitialize>(json!({
                    "processId": std::process::id(),
                    "rootUri": root_uri,
                    "workspaceFolders": [{ "uri": root_uri, "name": "tsx-script-syntax" }],
                    "capabilities": editor_capabilities(),
                    "initializationOptions": { "runExternalCode": true }
                }))
                .await
                .unwrap();
            client.notify::<RawInitialized>(json!({})).unwrap();
            client
                .request::<RawSetContentMapperContributions>(json!({
                    "contributions": [{
                        "contributorId": "vize",
                        "extensions": [".vue"],
                        "inferredProjectContribution": {
                            "options": {},
                            "manifest": {
                                "name": "vize",
                                "exec": [env!("CARGO_BIN_EXE_vize"), "content-mapper"],
                                "compilerOptions": ["noUnusedLocals"],
                                "dynamicConfig": true
                            }
                        }
                    }],
                    "openDocuments": [{ "uri": uri }]
                }))
                .await
                .unwrap();
            client
                .overlay()
                .open(VirtualDocument::new(
                    Uri::from_str(&uri).unwrap(),
                    "vue",
                    CLEAN,
                ))
                .unwrap();

            let diagnostics = pull_diagnostics(&client, &uri).await;
            assert_eq!(diagnostics["items"], json!([]), "{diagnostics:#}");

            // A moved assertion expression and its moved type keep hover ranges.
            for (anchor, inner, fragment) in [
                ("<HTMLElement>document.body", "body", "body: HTMLElement"),
                ("<HTMLElement>document.body", "HTMLElement", "HTMLElement"),
                ("T => value;\nconst run", "value", "value: T"),
            ] {
                let (start, end) = at(anchor, inner);
                let response = hover(&client, &uri, &start).await;
                assert_eq!(
                    response["range"],
                    json!({ "start": start, "end": end }),
                    "{response:#}"
                );
                assert!(
                    response["contents"].to_string().contains(fragment),
                    "{response:#}"
                );
            }

            // Renames reach every authored occurrence, never a generated byte.
            for (target, name, occurrences) in [
                (
                    ("const element", "element"),
                    "node",
                    [
                        ("const element", "element"),
                        ("<any>element", "element"),
                        ("<HTMLElement>element)", "element"),
                    ],
                ),
                (
                    ("<T>(value: T): T", "T"),
                    "Item",
                    [
                        ("<T>(value: T): T", "T"),
                        ("(value: T): T", "T"),
                        ("): T =>", "T"),
                    ],
                ),
            ] {
                let renamed = rename(&client, &uri, &at(target.0, target.1).0, name).await;
                let expected = occurrences.map(|(anchor, inner)| at(anchor, inner));
                for (start, end) in &expected {
                    assert!(
                        contains_text_edit(&renamed, &uri, start, end, name),
                        "{renamed:#}"
                    );
                }
                for range in edit_ranges(&renamed) {
                    assert!(
                        expected
                            .iter()
                            .any(|(start, end)| range == json!({ "start": start, "end": end })),
                        "{range}: {renamed:#}"
                    );
                }
            }

            stop.store(true, Ordering::Relaxed);
            client.graceful_close().await.unwrap();
            responder.join().unwrap();
        });
    });
}

fn project(prefix: &str) -> tempfile::TempDir {
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(cases_root)
        .unwrap()
}

fn write_sources(root: &Path, newline: &str) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    for (name, source) in [
        ("Clean.vue", CLEAN),
        ("Plain.vue", PLAIN),
        ("Generic.vue", GENERIC),
        ("Errors.vue", ERRORS),
    ] {
        std::fs::write(root.join("src").join(name), source.replace('\n', newline)).unwrap();
    }
    std::fs::write(
        root.join("tsconfig.json"),
        serde_json::to_vec_pretty(&json!({
            "compilerOptions": {
                "strict": true, "noEmit": true, "skipLibCheck": true,
                "target": "ES2022", "module": "ESNext",
                "moduleResolution": "bundler", "types": []
            },
            "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
            "include": ["src"]
        }))
        .unwrap(),
    )
    .unwrap();
}

/// Every edit range in a workspace edit, whichever shape the host returns.
fn edit_ranges(edit: &Value) -> Vec<Value> {
    let changes = edit["changes"]
        .as_object()
        .into_iter()
        .flat_map(|changes| changes.values());
    let documents = edit["documentChanges"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|change| &change["edits"]);
    changes
        .chain(documents)
        .filter_map(Value::as_array)
        .flatten()
        .map(|edit| edit["range"].clone())
        .collect()
}

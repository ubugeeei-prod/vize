//! New references for #9516; these are not the unrecovered Misskey 415 findings.
#![expect(clippy::unwrap_used, reason = "complete native diagnostic assertions")]

use super::{native, packets};
use crate::{
    BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait,
    corsa_client::CorsaProjectClient, file_uri::path_to_file_uri,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
use vize_carton::{String, cstr, line_index::LineBreaks};

fn original(fixture: &str, issue: u32) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/content-mapper")
        .join(fixture);
    let fixture: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    fixture["originals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["issue"] == issue)
        .unwrap()
        .clone()
}

#[test]
fn generated_tuple_and_property_references_keep_raw_and_authored_diagnostics_distinct() {
    let Some(native) = native() else { return };
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let provider = repo.join("tests/node_modules").canonicalize().unwrap();
    let native_bytes = fs::read(&native).unwrap();
    let native_sha: String = Sha256::digest(&native_bytes)
        .iter()
        .map(|byte| cstr!("{byte:02x}"))
        .collect();
    packets::retain(
        "generated-diagnostics-provider",
        &json!({
            "nativePath":native,"nativeBytes":native_bytes.len(),"nativeSha256":native_sha,
            "vueCanonicalTarget":provider.join("vue").canonicalize().unwrap(),
            "wholeVueManifest":fs::read_to_string(provider.join("vue/package.json")).unwrap(),
            "sourceScope":"current source-built Rust test and its selected native backend; not historical Misskey/public package acceptance",
        }),
    );
    let event = original("escaton-component-events.json", 9427);
    let props = original("escaton-component-props.json", 9428);
    let mut failures = Vec::new();
    for (case, original, addition, expected) in [
        ("empty-event", &event, "", None),
        ("inferred-data-attribute", &props, "", None),
        (
            "authored-tuple",
            &event,
            "const empty = [] as const;\nvoid empty[0];\n",
            Some((
                2493,
                "Tuple type 'readonly []' of length '0' has no element at index '0'.",
            )),
        ),
        (
            "authored-property",
            &event,
            "const closed: { count: number } = { extra: 1 };\nvoid closed;\n",
            Some((
                2353,
                "Object literal may only specify known properties, and 'extra' does not exist in type '{ count: number; }'.",
            )),
        ),
    ] {
        let observed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let temporary = tempfile::tempdir().unwrap();
            let root = temporary.path().canonicalize().unwrap();
            std::os::unix::fs::symlink(&provider, root.join("node_modules")).unwrap();
            let original_files = original["files"].as_object().unwrap();
            let mut config: Value =
                serde_json::from_str(original_files["tsconfig.json"].as_str().unwrap()).unwrap();
            // This is the existing Batch API, not the external content-mapper
            // plugin. Preserve its original config separately; don't claim that
            // this distinct native project is the old reporter's invocation.
            config.as_object_mut().unwrap().remove("contentMappers");
            fs::write(
                root.join("tsconfig.json"),
                serde_json::to_vec(&config).unwrap(),
            )
            .unwrap();
            let mut paths = Vec::new();
            for (name, content) in original_files {
                if name == "tsconfig.json" {
                    continue;
                }
                let source = if name == "App.vue" {
                    content
                        .as_str()
                        .unwrap()
                        .replace("</script>", &cstr!("{addition}</script>"))
                } else {
                    content.as_str().unwrap().to_owned()
                };
                fs::write(root.join(name), source).unwrap();
                paths.push(root.join(name));
            }
            let mut checker = BatchTypeChecker::with_options_and_corsa_path(
                &root,
                BatchTypeCheckerOptions {
                    tsconfig_path: Some(root.join("tsconfig.json")),
                    ..Default::default()
                },
                Some(&native),
            )
            .unwrap();
            checker.set_server_count(Some(1));
            checker.set_checker_count(Some(1));
            checker.scan_paths(&paths).unwrap();
            let public = checker.check_project().unwrap();
            let (mirror, config) = checker.native_project_paths_for_test();
            let files = checker.virtual_files();
            let (mapped, packet) = packets::diagnose_selected(
                &root,
                &native,
                &files,
                case,
                "current",
                None,
                (&mirror, &config),
                &paths,
            );
            let authority = &packet["nativeAuthority"];
            assert_eq!(authority["nativeBinary"], json!(native), "{authority}");
            assert_eq!(
                authority["normalizedRequestedOptions"]["strict"], true,
                "{authority}"
            );
            assert_eq!(
                authority["sourceFileNamesResponse"]["status"], "ok",
                "{authority}"
            );
            packets::retain(
                &cstr!("{case}-new-reference"),
                &json!({
                    "reference":"Escaton 9427/9428 sources with current Batch API, not original Misskey diagnostics",
                    "original":original,"authoredAddition":addition,
                    "providerVueManifest":fs::read_to_string(provider.join("vue/package.json")).unwrap(),
                    "wholePublicResult":cstr!("{public:#?}"),
                    "rawScope":"decoded pre-assembly LspDiagnostic vector, not untouched RPC frames",
                    "packet":packet,
                }),
            );
            if case == "empty-event" {
                let app = files
                    .iter()
                    .find(|file| file.original_path == root.join("App.vue"))
                    .unwrap();
                let replay = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    replay_prior_runtime_tuple_read(&native, &mirror, &config, app);
                }));
                if replay.is_err() {
                    failures.push("empty-event-prior-runtime-expression");
                }
            }
            let raw: Vec<_> = packet["wholeRawDiagnosticVector"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|row| row.get(1).unwrap().as_array().unwrap())
                .collect();
            let expected_rows: Vec<_> = expected
                .into_iter()
                .map(|(code, message)| (json!(code), json!(1), json!(message)))
                .collect();
            let app = files
                .iter()
                .find(|file| file.original_path == root.join("App.vue"))
                .unwrap();
            let mut expected_raw = Vec::new();
            if let Some((code, message)) = expected {
                let (needle, relative, length) = if code == 2493 {
                    ("empty[0]", "empty[".len(), 1)
                } else {
                    ("extra: 1", 0, "extra".len())
                };
                let start = app.content.find(needle).unwrap() + relative;
                expected_raw.push(native_diagnostic(
                    &app.content,
                    start,
                    start + length,
                    code,
                    1,
                    message,
                ));
            }
            expected_raw.extend(known_hints(
                &app.content,
                match case {
                    "inferred-data-attribute" => PROP_HINTS,
                    "empty-event" | "authored-tuple" | "authored-property" => EVENT_HINTS,
                    _ => panic!("unknown reference case"),
                },
            ));
            let app_uri = json!(path_to_file_uri(&app.virtual_path));
            let expected_response: Vec<_> = packet["requestedUris"]
                .as_array()
                .unwrap()
                .iter()
                .map(|uri| {
                    json!([
                        uri,
                        if uri == &app_uri {
                            json!(expected_raw)
                        } else {
                            json!([])
                        }
                    ])
                })
                .collect();
            assert_eq!(
                packet["wholeRawDiagnosticVector"],
                json!(expected_response),
                "{packet}"
            );
            assert_eq!(public.diagnostics.len(), expected_rows.len(), "{public:#?}");
            assert_eq!(mapped.len(), expected_rows.len(), "{packet}");
            if let Some((code, message)) = expected {
                let (needle, relative, length) = if code == 2493 {
                    ("empty[0]", "empty[".len(), 1)
                } else {
                    ("extra: 1", 0, "extra".len())
                };
                let source = fs::read_to_string(root.join("App.vue")).unwrap();
                let at = source.find(needle).unwrap() + relative;
                let authored = LineBreaks::Lsp.offset_to_position(&source, at);
                let app = files
                    .iter()
                    .find(|file| file.original_path == root.join("App.vue"))
                    .unwrap();
                let generated = app.content.find(needle).unwrap() + relative;
                let start = LineBreaks::Lsp.offset_to_position(&app.content, generated);
                let end = LineBreaks::Lsp.offset_to_position(&app.content, generated + length);
                assert_eq!(
                    raw.first().unwrap()["range"],
                    json!({
                        "start":{"line":start.0,"character":start.1},
                        "end":{"line":end.0,"character":end.1},
                    }),
                    "{packet}"
                );
                assert_eq!(
                    mapped,
                    [(
                        String::from("App.vue"),
                        authored.0,
                        authored.1,
                        Some(code),
                        Some(1),
                        String::from(message)
                    )],
                    "{packet}"
                );
                let diagnostic = public.diagnostics.first().unwrap();
                assert_eq!(
                    (
                        &diagnostic.file,
                        diagnostic.line,
                        diagnostic.column,
                        diagnostic.code,
                        diagnostic.severity,
                        diagnostic.message.as_str(),
                        diagnostic.block_type
                    ),
                    (
                        &root.join("App.vue"),
                        authored.0,
                        authored.1,
                        Some(code),
                        1,
                        message,
                        Some(crate::SfcBlockType::ScriptSetup)
                    ),
                    "{public:#?}"
                );
            }
        }));
        if observed.is_err() {
            failures.push(case);
        }
    }
    assert!(
        failures.is_empty(),
        "all reference gates must pass: {failures:?}"
    );
}

fn replay_prior_runtime_tuple_read(
    native: &Path,
    mirror: &Path,
    config: &Path,
    file: &crate::batch::virtual_project::VirtualFile,
) {
    // Exact runtime expression inverse from 9435's first parent. Only this
    // site changes; the other historical type-alias site stays repaired.
    let current = "(__vize_args as unknown[])[0]";
    let prior = "__vize_args[0]";
    assert_eq!(file.content.matches(current).count(), 1);
    assert!(
        file.source_map
            .get_original_position(u32::try_from(file.content.find(current).unwrap()).unwrap())
            .is_none(),
        "the synthetic runtime extraction must have no authored owner"
    );
    let replay = file.content.replacen(current, prior, 1);
    fs::write(&file.virtual_path, &replay).unwrap();
    let mut client = CorsaProjectClient::new_for_workspace_config(
        Some(native.to_str().unwrap()),
        mirror,
        config,
    )
    .unwrap();
    let uri = path_to_file_uri(&file.virtual_path);
    client.begin_batch_test_receipt(true);
    let raw = client.request_diagnostics_batch(&[uri.clone()]).unwrap();
    let authority = client.batch_test_receipt(&uri);
    packets::retain(
        "empty-event-prior-runtime-expression",
        &json!({
            "scope":"current factory with only the exact pre-9435 runtime expression; not historical factory/Misskey execution",
            "historicalFix":"2e5e976ec6105311fd7a4399c0f6cd8b6dfb8250",
            "currentExpression":current,"priorExpression":prior,
            "wholeReplayProgram":replay,"wholeCurrentProgram":file.content.as_str(),
            "currentMapsNotAppliedToDifferentLengthReplay":cstr!("{:#?}",file.source_map),
            "wholeDecodedPreAssemblyVector":raw,"nativeAuthority":authority,
            "wholeGeneratedConfig":fs::read_to_string(config).unwrap(),
        }),
    );
    client.shutdown().unwrap();
    fs::write(&file.virtual_path, file.content.as_str()).unwrap();
    assert_eq!(authority["selectedConfig"], json!(config));
    assert_eq!(authority["nativeBinary"], json!(native));
    assert_eq!(
        authority["project"]["compilerOptions"],
        authority["normalizedRequestedOptions"]
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
    let diagnostics: Vec<_> = raw
        .iter()
        .flat_map(|(_, diagnostics)| diagnostics)
        .collect();
    assert_eq!(diagnostics.len(), EVENT_HINTS.len() + 1, "{raw:#?}");
    let diagnostic = diagnostics.first().unwrap();
    assert_eq!(diagnostic.code, Some(json!(2493)));
    assert_eq!(diagnostic.severity, Some(1));
    assert_eq!(
        diagnostic.message.as_str(),
        "Tuple type '[]' of length '0' has no element at index '0'."
    );
    let index = replay.find(prior).unwrap() + "__vize_args[".len();
    let start = LineBreaks::Lsp.offset_to_position(&replay, index);
    let end = LineBreaks::Lsp.offset_to_position(&replay, index + 1);
    assert_eq!(
        (
            diagnostic.range.start.line,
            diagnostic.range.start.character
        ),
        start
    );
    assert_eq!(
        (diagnostic.range.end.line, diagnostic.range.end.character),
        end
    );
    let mut expected = vec![native_diagnostic(
        &replay,
        index,
        index + 1,
        2493,
        1,
        "Tuple type '[]' of length '0' has no element at index '0'.",
    )];
    expected.extend(known_hints(&replay, EVENT_HINTS));
    assert_eq!(json!(raw), json!([[uri, expected]]));
}

// Exact suggestion vectors from the authenticated source qualification. These
// are generated declarations, not arbitrary diagnostic-name exemptions.
const EVENT_HINTS: &[&str] = &[
    "__VizeHasRawProps",
    "__VizePropValue",
    "__VizeNavigationSlots",
    "__VizeEmitPropsFactory",
];
const PROP_HINTS: &[&str] = &["__VizeHasRawProps", "__VizeNavigationSlots"];

fn known_hints(generated: &str, names: &[&str]) -> Vec<Value> {
    names
        .iter()
        .map(|name| {
            let declaration = cstr!("type {name}<");
            assert_eq!(generated.matches(declaration.as_str()).count(), 1);
            let start = generated.find(declaration.as_str()).unwrap() + "type ".len();
            native_diagnostic(
                generated,
                start,
                start + name.len(),
                6196,
                4,
                &cstr!("'{name}' is declared but never used."),
            )
        })
        .collect()
}

fn native_diagnostic(
    generated: &str,
    start: usize,
    end: usize,
    code: u32,
    severity: i32,
    message: &str,
) -> Value {
    let start = LineBreaks::Lsp.offset_to_position(generated, start);
    let end = LineBreaks::Lsp.offset_to_position(generated, end);
    json!({"code":code,"severity":severity,"source":"ts","message":message,
        "range":{"start":{"line":start.0,"character":start.1},"end":{"line":end.0,"character":end.1}}})
}

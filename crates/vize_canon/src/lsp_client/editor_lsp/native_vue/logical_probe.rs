//! Diagnostic-only: an API callback must not lend authority to another LSP owner.
#![expect(
    clippy::disallowed_types,
    reason = "the existing SDK filesystem API owns an Arc"
)]

use super::filesystem::ProjectionFileSystem;
use crate::{file_uri::path_to_file_uri, lsp_client::editor_lsp::EditorLspSession};
use corsa::{
    api::{ApiFileSystem, ApiMode, ApiSpawnConfig, ProjectSession, ReadFileResult},
    runtime::block_on,
};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fmt::Display,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};
use vize_l0::cstr;

const BAD: &str = "export const value = 'bad';\n";
const REPAIRED: &str = "export const value = 1;\n";
const FOREIGN: &str = "export const value = 'foreign';\n";
const MAIN_TAIL: &str = "import { value as sibling } from './other/support';\nexport const checked: number = value;\nexport const foreignChecked: 'foreign' = sibling;\n";
const MAX_PACKET_BYTES: usize = 4 * 1024 * 1024;

fn hash(bytes: &[u8]) -> vize_l0::String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| cstr!("{byte:02x}"))
        .collect()
}

fn save(root: &Path, name: &str, value: Value) -> Value {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(cstr!("{name}.json").as_str()))
        .unwrap();
    let bytes = serde_json::to_vec_pretty(&value).unwrap();
    if bytes.len() > MAX_PACKET_BYTES {
        serde_json::to_writer_pretty(file, &json!({"complete":false,"refused":"packet byte cap","originalBytes":bytes.len(),"sha256":hash(&bytes),"limitBytes":MAX_PACKET_BYTES})).unwrap();
        panic!("logical probe observation exceeded the explicit packet cap");
    }
    file.write_all(&bytes).unwrap();
    value
}

fn observed<T: Serialize, E: Display>(root: &Path, name: &str, result: Result<T, E>) -> Value {
    save(
        root,
        name,
        match result {
            Ok(value) => json!({"ok":true,"value":value}),
            Err(error) => json!({"ok":false,"error":cstr!("{error}")}),
        },
    )
}

#[derive(Debug, PartialEq, Eq, Serialize)]
enum Entry {
    Directory,
    File(Vec<u8>),
    Link(PathBuf),
}

fn inventory(root: &Path) -> BTreeMap<PathBuf, Entry> {
    fn visit(root: &Path, current: &Path, rows: &mut BTreeMap<PathBuf, Entry>) {
        for entry in fs::read_dir(current).unwrap().map(Result::unwrap) {
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            let value = if let Ok(target) = fs::read_link(&path) {
                Entry::Link(target)
            } else if metadata.is_dir() {
                visit(root, &path, rows);
                Entry::Directory
            } else {
                assert!(metadata.is_file());
                Entry::File(fs::read(&path).unwrap())
            };
            rows.insert(path.strip_prefix(root).unwrap().to_path_buf(), value);
        }
    }
    let mut rows = BTreeMap::new();
    visit(root, root, &mut rows);
    rows
}

fn api_packet(
    root: &Path,
    native: &Path,
    project: &Path,
    config: &Path,
    logical: &Path,
    source: &str,
    foreign: &Path,
) -> Value {
    let filesystem = Arc::new(ProjectionFileSystem::new(logical, source).unwrap());
    let callbacks = save(
        root,
        "callbacks",
        json!({
            "logical":logical,"foreignSameBasename":foreign,
            "logicalContentExact":matches!(filesystem.read_file(logical.to_str().unwrap()),ReadFileResult::Content(text) if text.as_str()==source),
            "logicalExists":filesystem.file_exists(logical.to_str().unwrap()),
            "logicalRealpath":filesystem.realpath(logical.to_str().unwrap()).map(|path|Value::String(path.into())),
            "foreignFallback":matches!(filesystem.read_file(foreign.to_str().unwrap()),ReadFileResult::Fallback),
            "foreignExistsOverride":filesystem.file_exists(foreign.to_str().unwrap()),
            "foreignRealpathOverride":filesystem.realpath(foreign.to_str().unwrap()).map(|path|Value::String(path.into())),
            "parentEntries":filesystem.get_accessible_entries(logical.parent().unwrap().to_str().unwrap()).map(|entries|json!({"files":Value::Array(entries.files.into_iter().map(|path|Value::String(path.into())).collect()),"directories":Value::Array(entries.directories.into_iter().map(|path|Value::String(path.into())).collect())}))
        }),
    );
    let opened = block_on(ProjectSession::spawn(
        ApiSpawnConfig::new(native)
            .with_cwd(project)
            .with_mode(
                if vize_carton::corsa_api_mode::uses_async_json_rpc_api(native) {
                    ApiMode::AsyncJsonRpcStdio
                } else {
                    ApiMode::SyncMsgpackStdio
                },
            )
            .with_filesystem(filesystem),
        config.to_str().unwrap(),
        Some(crate::lsp_client::session::uri_document_identifier(
            &path_to_file_uri(&project.join("src/main.ts")),
        )),
    ));
    let opened_record = observed(
        root,
        "open",
        opened.as_ref().map(|session| session.project()),
    );
    let session = match opened {
        Ok(session) => session,
        Err(_) => return json!({"callbacks":callbacks,"open":opened_record}),
    };
    let effective = observed(
        root,
        "effective",
        block_on(session.client().parse_config_file(
            crate::lsp_client::session::uri_document_identifier(&path_to_file_uri(config)),
        )),
    );
    let project_record = json!(session.project());
    let closed = observed(root, "close", block_on(session.close()));
    json!({"callbacks":callbacks,"open":opened_record,"effective":effective,"project":project_record,"close":closed})
}

fn lsp_phase(
    root: &Path,
    editor: &mut EditorLspSession,
    main: &str,
    logical: &Path,
    source: &str,
) -> Value {
    let uri = path_to_file_uri(logical);
    let opened = observed(root, "mirror-logical", editor.mirror(&uri, source));
    let before = observed(
        root,
        "configured-before",
        editor.configured_project_receipt(main),
    );
    let diagnostics = observed(root, "diagnostics", editor.diagnostics(main));
    let character = MAIN_TAIL.lines().nth(1).unwrap().find("value").unwrap() as u32;
    let hover = observed(root, "hover", editor.hover(main, 2, character));
    let definition = observed(root, "definition", editor.definition(main, 2, character));
    let after = observed(
        root,
        "configured-after",
        editor.configured_project_receipt(main),
    );
    json!({"mirror":opened,"before":before,"diagnostics":diagnostics,"hover":hover,"definition":definition,"after":after})
}

fn lsp_packets(
    root: &Path,
    native: &Path,
    project: &Path,
    config: &Path,
    logical: &Path,
    main_text: &str,
    repair: bool,
) -> Value {
    let opened = EditorLspSession::spawn_with_startup_configuration(
        native.to_str().unwrap(),
        project,
        project,
        config,
    );
    let status = observed(
        root,
        "open",
        opened
            .as_ref()
            .map(|_| json!({"cwd":project,"selectedConfig":config,"independentStandardLsp":true})),
    );
    let mut editor = match opened {
        Ok(editor) => editor,
        Err(_) => return json!({"open":status}),
    };
    let main = path_to_file_uri(&project.join("src/main.ts"));
    let main_open = observed(root, "mirror-main", editor.mirror(&main, main_text));
    let broken_root = root.join("broken");
    fs::create_dir(&broken_root).unwrap();
    let broken = lsp_phase(&broken_root, &mut editor, &main, logical, BAD);
    let repaired = if repair {
        let repaired_root = root.join("repaired");
        fs::create_dir(&repaired_root).unwrap();
        lsp_phase(&repaired_root, &mut editor, &main, logical, REPAIRED)
    } else {
        Value::Null
    };
    let closed = observed(root, "close", editor.shutdown());
    json!({"open":status,"mainMirror":main_open,"broken":broken,"repaired":repaired,"close":closed})
}

fn value(packet: &Value) -> &Value {
    assert_eq!(packet["ok"], true, "{packet:#}");
    &packet["value"]
}

fn code(diagnostic: &Value) -> i64 {
    diagnostic["code"]
        .as_i64()
        .or_else(|| {
            diagnostic["code"]
                .as_str()
                .and_then(|code| code.strip_prefix("TS").unwrap_or(code).parse().ok())
        })
        .unwrap()
}

fn lsp_codes(packet: &Value) -> Vec<i64> {
    let packet = value(&packet["diagnostics"]);
    assert_eq!(packet["kind"], "full", "{packet:#}");
    packet["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(code)
        .collect()
}

fn assert_api(
    packet: &Value,
    config: &Path,
    project: &Path,
    logical: &Path,
    foreign: &Path,
    inside: bool,
) {
    let callbacks = &packet["callbacks"];
    assert_eq!(callbacks["logicalContentExact"], true);
    assert_eq!(callbacks["logicalExists"], true);
    assert_eq!(
        callbacks["logicalRealpath"],
        logical.to_string_lossy().as_ref()
    );
    assert_eq!(callbacks["foreignFallback"], true);
    assert_eq!(callbacks["foreignExistsOverride"], Value::Null);
    assert_eq!(callbacks["foreignRealpathOverride"], Value::Null);
    value(&packet["open"]);
    value(&packet["close"]);
    let effective = value(&packet["effective"]);
    assert_eq!(
        packet["project"]["configFileName"],
        config.to_string_lossy().as_ref()
    );
    assert_eq!(effective["options"], packet["project"]["compilerOptions"]);
    assert_eq!(
        effective["options"]["rootDir"],
        project.join("src").to_string_lossy().as_ref()
    );
    assert_eq!(effective["options"]["strict"], true);
    assert_eq!(effective["options"]["noEmit"], true);
    let includes = |field: &Value, path: &Path| {
        field
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name.as_str() == path.to_str())
    };
    assert!(includes(
        &effective["fileNames"],
        &project.join("src/main.ts")
    ));
    assert!(includes(&effective["fileNames"], foreign));
    assert_eq!(includes(&effective["fileNames"], logical), inside);
    assert_eq!(includes(&packet["project"]["rootFiles"], logical), inside);
}

fn assert_lsp(
    packet: &Value,
    api: &Value,
    config: &Path,
    main: &Path,
    logical: &Path,
    foreign: &Path,
    value_type: &str,
) {
    value(&packet["mirror"]);
    let expected_options = &value(&api["effective"])["options"];
    for side in ["before", "after"] {
        let receipt = value(&packet[side]);
        assert_eq!(
            receipt["project"]["configFileName"],
            config.to_string_lossy().as_ref()
        );
        assert_eq!(&receipt["project"]["compilerOptions"], expected_options);
        assert_eq!(&receipt["normalizedRequestedOptions"], expected_options);
        assert_eq!(receipt["sourceFileNamesResponse"]["status"], "ok");
        let names = receipt["sourceFileNamesResponse"]["wholeResponse"]
            .as_array()
            .unwrap();
        for expected in [main, logical, foreign] {
            assert!(
                names.iter().any(|name| name.as_str() == expected.to_str()),
                "actual diagnosing LSP omitted {expected:?}: {receipt:#}"
            );
        }
    }
    let character = MAIN_TAIL.lines().nth(1).unwrap().find("value").unwrap() as u32;
    let definition = value(&packet["definition"]);
    assert_eq!(
        definition,
        &json!([{"uri":path_to_file_uri(logical),"range":{"start":{"line":0,"character":13},"end":{"line":0,"character":18}}}])
    );
    assert_eq!(
        value(&packet["hover"]),
        &json!({"contents":{"kind":"markdown","value":cstr!("```typescript\n(alias) const value: {value_type}\n```\n")},"range":{"start":{"line":2,"character":character},"end":{"line":2,"character":character+5}}})
    );
}

#[test]
#[ignore = "diagnostic-only: real locked native provider; API success alone is insufficient"]
fn actual_logical_projection_requires_the_independent_configured_lsp_to_agree() {
    let capture = PathBuf::from(
        std::env::var_os("VIZE_LOGICAL_PROBE_CAPTURE")
            .expect("fresh owned capture directory required"),
    );
    assert!(capture.is_absolute());
    fs::create_dir(&capture).unwrap();
    let runtime_source = std::env::var("EXPECTED_SOURCE").expect("literal source required");
    let compiled_source = option_env!("VIZE_LOGICAL_PROBE_COMPILED_SOURCE");
    save(
        &capture,
        "compiled-source",
        json!({"runtimeSource":runtime_source,"compiledSource":compiled_source}),
    );
    assert_eq!(compiled_source, Some(runtime_source.as_str()));
    let native =
        PathBuf::from(std::env::var_os("CORSA_PATH").expect("locked native provider required"))
            .canonicalize()
            .unwrap();
    let library = std::env::current_exe().unwrap().canonicalize().unwrap();
    let library_bytes = fs::read(&library).unwrap();
    let library_image =
        json!({"path":library,"bytes":library_bytes.len(),"sha256":hash(&library_bytes)});
    save(&capture, "library-image", library_image.clone());
    let version = Command::new(&native).arg("--version").output().unwrap();
    let native_hash = hash(&fs::read(&native).unwrap());
    save(
        &capture,
        "identity",
        json!({"schema":1,"scope":"private logical-path source probe, not public package or product qualification","source":runtime_source,"compiledSource":compiled_source,"native":{"path":native,"sha256":native_hash,"versionStdout":version.stdout,"versionStderr":version.stderr,"versionCode":version.status.code()},"libraryImage":library_image,"representation":"complete SDK-decoded API/LSP values and errors retained before semantic assertions; not raw framed wire","timeouts":"unchanged API/LspSpawnConfig defaults, no environment mutation"}),
    );
    let mut packets = Vec::new();
    for (label, inside) in [("inside-root", true), ("outside-root", false)] {
        let retained = capture.join(label);
        fs::create_dir(&retained).unwrap();
        let work = tempfile::tempdir().unwrap();
        let project = work.path().canonicalize().unwrap();
        fs::create_dir_all(project.join("src/other")).unwrap();
        fs::create_dir(project.join("outside")).unwrap();
        let foreign = project.join("src/other/support.ts");
        fs::write(&foreign, FOREIGN).unwrap();
        let logical = project.join(if inside {
            "src/support.ts"
        } else {
            "outside/support.ts"
        });
        let main = project.join("src/main.ts");
        let main_text = cstr!(
            "import {{ value }} from '{}';\n{MAIN_TAIL}",
            if inside {
                "./support"
            } else {
                "../outside/support"
            }
        );
        fs::write(&main, main_text.as_bytes()).unwrap();
        let config = project.join("tsconfig.json");
        let authored = serde_json::to_vec_pretty(&json!({"compilerOptions":{"strict":true,"target":"ESNext","module":"ESNext","moduleResolution":"Bundler","noEmit":true,"rootDir":"src","types":[]},"include":["src/**/*.ts"]})).unwrap();
        fs::write(&config, &authored).unwrap();
        let before = inventory(&project);
        save(
            &retained,
            "authored-before",
            json!({"project":project,"main":main,"mainText":main_text,"logical":logical,"logicalPhysicallyAbsent":!logical.exists(),"foreign":foreign,"foreignText":FOREIGN,"config":config,"authoredConfigBytes":authored,"inventory":before}),
        );
        let api_bad_root = retained.join("api-broken");
        fs::create_dir(&api_bad_root).unwrap();
        let api_bad = api_packet(
            &api_bad_root,
            &native,
            &project,
            &config,
            &logical,
            BAD,
            &foreign,
        );
        let lsp_root = retained.join("lsp");
        fs::create_dir(&lsp_root).unwrap();
        let lsp = lsp_packets(
            &lsp_root, &native, &project, &config, &logical, &main_text, inside,
        );
        let after = inventory(&project);
        let packet = save(
            &retained,
            "whole",
            json!({"inside":inside,"apiBroken":api_bad,"lsp":lsp,"project":project,"config":config,"main":main,"logical":logical,"foreign":foreign,"logicalStillAbsent":!logical.exists(),"authoredBytesUnchanged":fs::read(&config).unwrap()==authored,"before":before,"after":after,"inventoryEqual":before==after}),
        );
        packets.push(packet);
    }
    let native_after = hash(&fs::read(&native).unwrap());
    save(
        &capture,
        "whole",
        json!({"packets":packets,"nativeAfterSha256":native_after,"nativeImageUnchanged":native_after==native_hash,"qualificationCredit":0}),
    );
    assert_eq!(native_after, native_hash);
    assert_eq!(version.status.code(), Some(0));
    assert_eq!(version.stdout, b"Version 7.0.2\n");
    assert!(version.stderr.is_empty());
    for packet in &packets {
        let inside = packet["inside"].as_bool().unwrap();
        let project = Path::new(packet["project"].as_str().unwrap());
        let config = Path::new(packet["config"].as_str().unwrap());
        let main = Path::new(packet["main"].as_str().unwrap());
        let logical = Path::new(packet["logical"].as_str().unwrap());
        let foreign = Path::new(packet["foreign"].as_str().unwrap());
        assert_eq!(packet["logicalStillAbsent"], true);
        assert_eq!(packet["authoredBytesUnchanged"], true);
        assert_eq!(packet["inventoryEqual"], true);
        let api = &packet["apiBroken"];
        assert_api(api, config, project, logical, foreign, inside);
        let lsp = &packet["lsp"];
        value(&lsp["open"]);
        value(&lsp["mainMirror"]);
        value(&lsp["close"]);
        assert_lsp(
            &lsp["broken"],
            api,
            config,
            main,
            logical,
            foreign,
            "\"bad\"",
        );
        assert_eq!(
            lsp_codes(&lsp["broken"]),
            if inside { vec![2322] } else { vec![6059, 2322] }
        );
        if inside {
            assert_lsp(&lsp["repaired"], api, config, main, logical, foreign, "1");
            assert!(lsp_codes(&lsp["repaired"]).is_empty());
        }
    }
}

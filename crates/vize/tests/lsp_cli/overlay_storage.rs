//! The ordinary source CLI must not materialize into a linked installation.

use crate::support::lsp_process::{LspProcess, file_uri};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Debug, PartialEq, Eq)]
enum Entry {
    Directory,
    File(Vec<u8>),
    Link(PathBuf),
}

fn hash(bytes: &[u8]) -> vize_l0::String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| vize_l0::cstr!("{byte:02x}"))
        .collect()
}

fn inventory(root: &Path) -> BTreeMap<PathBuf, Entry> {
    fn visit(root: &Path, current: &Path, result: &mut BTreeMap<PathBuf, Entry>) {
        for entry in fs::read_dir(current).unwrap().map(Result::unwrap) {
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            let value = if let Ok(target) = fs::read_link(&path) {
                Entry::Link(target)
            } else if metadata.is_dir() {
                visit(root, &path, result);
                Entry::Directory
            } else {
                assert!(metadata.is_file());
                Entry::File(fs::read(&path).unwrap())
            };
            result.insert(path.strip_prefix(root).unwrap().to_path_buf(), value);
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

fn link(source: &Path, target: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target)
        .or_else(|_| junction::create(source, target))
        .unwrap();
}

fn retained_inventory(rows: &BTreeMap<PathBuf, Entry>) -> Vec<Value> {
    rows.iter()
        .map(|(path, entry)| match entry {
            Entry::Directory => json!({"path":path,"kind":"directory"}),
            Entry::Link(target) => json!({"path":path,"kind":"link","target":target}),
            Entry::File(bytes) => {
                json!({"path":path,"kind":"file","bytes":bytes.len(),"sha256":hash(bytes)})
            }
        })
        .collect()
}

#[test]
#[ignore = "actual locked native provider and source CLI; Linux/Windows Actions"]
fn default_native_overlay_preserves_the_whole_linked_installation_and_authored_project() {
    let capture = PathBuf::from(std::env::var_os("VIZE_PRIVATE_OVERLAY_CAPTURE").unwrap());
    let native = PathBuf::from(std::env::var_os("CORSA_PATH").unwrap())
        .canonicalize()
        .unwrap();
    assert!(std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_none());
    let directory = tempfile::tempdir().unwrap();
    let project = directory.path().join("consumer");
    let installed = directory.path().join("installed");
    let cache = directory.path().join("user-cache");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&installed).unwrap();
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let vue = [
        repo.join("node_modules/vue"),
        repo.join("npm/builder/vite/node_modules/vue"),
    ]
    .into_iter()
    .find(|path| path.join("package.json").is_file())
    .unwrap()
    .canonicalize()
    .unwrap();
    link(&vue, &installed.join("vue"));
    link(&installed, &project.join("node_modules"));
    fs::write(installed.join("sentinel"), b"whole installation\0\r\n").unwrap();
    let source = include_str!(
        "../../../../tests/_fixtures/differential/typechecker/native-overlay-input-custody-11145/App.vue.txt"
    );
    let app = project.join("App.vue");
    fs::write(&app, source).unwrap();
    let project_before = inventory(&project);
    let installed_before = inventory(&installed);
    let cli_hash = hash(&fs::read(env!("CARGO_BIN_EXE_vize")).unwrap());
    assert_eq!(
        cli_hash.as_str(),
        std::env::var("VIZE_OVERLAY_EXPECTED_CLI_SHA").unwrap()
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(&project)
        .arg("lsp")
        .env_remove("VIZE_LSP_LOG_FILE")
        // Match the product's 10s default, preventing the shared test helper's
        // ordinary CI-only 120s override from changing this regression route.
        .env("VIZE_LSP_NATIVE_DIAGNOSTIC_TIMEOUT_MS", "10000")
        .env("CORSA_PATH", &native)
        .env("XDG_CACHE_HOME", &cache)
        .env("LOCALAPPDATA", &cache);
    let mut lsp = LspProcess::spawn_command(command, Stdio::piped());
    lsp.retain_protocol();
    let uri = file_uri(&app);
    lsp.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":file_uri(&project),"capabilities":{},"initializationOptions":{"lint":false,"typecheck":true,"ecosystem":false,"crossFile":true}}}));
    let initialized = lsp.recv_response(1);
    assert_eq!(initialized.get("error"), None);
    assert_eq!(initialized["result"]["serverInfo"]["name"], "vize-maestro");
    lsp.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"vue","version":1,"text":source}}}));
    let publication = lsp.recv_matching(|message| {
        message["method"] == "textDocument/publishDiagnostics"
            && message["params"]["uri"].as_str() == Some(uri.as_str())
            && message["params"]["version"] == 1
    });
    lsp.send(json!({"jsonrpc":"2.0","id":2,"method":"textDocument/foldingRange","params":{"textDocument":{"uri":uri}}}));
    let folding = lsp.recv_response(2);
    lsp.send(json!({"jsonrpc":"2.0","id":3,"method":"shutdown"}));
    let shutdown = lsp.recv_response(3);
    lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
    let status = lsp.wait_for_exit();
    let project_after = inventory(&project);
    let installed_after = inventory(&installed);
    let terminal = lsp.terminal_evidence();
    let record = json!({"source":std::env::var("EXPECTED_SOURCE").unwrap(),"scope":"ordinary source CLI, real native provider, unchanged cwd/rootUri, default log override unset; not public-package acceptance","firstCLI":{"path":env!("CARGO_BIN_EXE_vize"),"sha256":cli_hash},"native":{"path":native,"sha256":hash(&fs::read(&native).unwrap())},"rootUri":file_uri(&project),"cwd":project,"logOverride":null,"cache":{"XDG_CACHE_HOME":cache,"LOCALAPPDATA":cache},"initialized":initialized,"publication":publication,"folding":folding,"shutdown":shutdown,"terminal":terminal,"projectBefore":retained_inventory(&project_before),"projectAfter":retained_inventory(&project_after),"installedBefore":retained_inventory(&installed_before),"installedAfter":retained_inventory(&installed_after)});
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(capture.join("source-cli.json"))
        .unwrap();
    serde_json::to_writer_pretty(file, &record).unwrap();
    assert_eq!(
        publication,
        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"version":1,"diagnostics":[]}})
    );
    let expected: Value = serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/differential/lsp/default-log-installation-custody-11145/response.expected.json"
    ))
    .unwrap();
    assert_eq!(folding, json!({"jsonrpc":"2.0","id":2,"result":expected}));
    assert_eq!(shutdown, json!({"jsonrpc":"2.0","id":3,"result":null}));
    assert_eq!(status.code(), Some(0));
    assert_eq!(
        hash(&fs::read(env!("CARGO_BIN_EXE_vize")).unwrap()),
        cli_hash
    );
    assert_eq!(project_before, project_after);
    assert_eq!(installed_before, installed_after);
}

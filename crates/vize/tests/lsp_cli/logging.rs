//! Real stdio startup keeps the full project and linked installation immutable.
#![cfg(unix)]

use crate::support::lsp_process::{LspProcess, file_uri};
use serde_json::json;
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

fn inventory(root: &Path) -> BTreeMap<PathBuf, Entry> {
    fn walk(root: &Path, current: &Path, result: &mut BTreeMap<PathBuf, Entry>) {
        for entry in fs::read_dir(current).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let kind = entry.file_type().unwrap();
            let value = if kind.is_symlink() {
                Entry::Link(fs::read_link(&path).unwrap())
            } else if kind.is_dir() {
                walk(root, &path, result);
                Entry::Directory
            } else {
                Entry::File(fs::read(&path).unwrap())
            };
            result.insert(path.strip_prefix(root).unwrap().to_path_buf(), value);
        }
    }
    let mut result = BTreeMap::new();
    walk(root, root, &mut result);
    result
}

fn run(command: Command, project: &Path) {
    let mut lsp = LspProcess::spawn_command(command, Stdio::piped());
    lsp.send(
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "processId":null,"rootUri":file_uri(project),"capabilities":{},
            "initializationOptions":{"lint":false,"typecheck":false,"ecosystem":false}
        }}),
    );
    let reply = lsp.recv_response(1);
    assert_eq!(reply.get("error"), None);
    assert_eq!(reply["result"]["serverInfo"]["name"], "vize-maestro");
    lsp.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
    lsp.send(json!({"jsonrpc":"2.0","id":2,"method":"shutdown"}));
    assert_eq!(
        lsp.recv_response(2),
        json!({"jsonrpc":"2.0","id":2,"result":null})
    );
    lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
    assert_eq!(lsp.wait_for_exit().code(), Some(0));
}

#[test]
fn explicit_and_invalid_log_paths_preserve_the_entire_linked_installation() {
    for explicit in [true, false] {
        let directory = tempfile::tempdir().unwrap();
        let project = directory.path().join("project");
        let installed = directory.path().join("installed");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&installed).unwrap();
        fs::write(installed.join("sentinel"), b"whole retained package\0\r\n").unwrap();
        std::os::unix::fs::symlink(&installed, project.join("node_modules")).unwrap();
        let project_before = inventory(&project);
        let installed_before = inventory(&installed);
        let log = directory.path().join("evidence/lsp.log");
        let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
        command.current_dir(&project).arg("lsp").env(
            "VIZE_LSP_LOG_FILE",
            if explicit {
                log.as_os_str()
            } else {
                Path::new("node_modules/relative.log").as_os_str()
            },
        );
        run(command, &project);
        assert_eq!(inventory(&project), project_before);
        assert_eq!(inventory(&installed), installed_before);
        assert_eq!(log.exists(), explicit);
    }
}

#[test]
#[cfg(target_os = "linux")]
fn default_log_uses_user_cache_with_override_unset_and_no_installation_writes() {
    let directory = tempfile::tempdir().unwrap();
    let project = directory.path().join("project");
    let installed = directory.path().join("installed");
    let cache = directory.path().join("user-cache");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&installed).unwrap();
    fs::write(
        installed.join("sentinel"),
        b"whole retained installation\0\r\n",
    )
    .unwrap();
    std::os::unix::fs::symlink(&installed, project.join("node_modules")).unwrap();
    let project_before = inventory(&project);
    let installed_before = inventory(&installed);
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(&project)
        .arg("lsp")
        .env_remove("VIZE_LSP_LOG_FILE")
        .env("XDG_CACHE_HOME", &cache);
    run(command, &project);
    assert_eq!(inventory(&project), project_before);
    assert_eq!(inventory(&installed), installed_before);
    let log = fs::read_to_string(cache.join("vize/logs/lsp.log")).unwrap();
    assert_eq!(
        log.lines().next().unwrap().split_once(" INFO ").unwrap().1,
        "vize_maestro: Starting vize_maestro LSP server"
    );
}

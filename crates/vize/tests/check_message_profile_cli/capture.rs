//! Retain complete packets even when an exact native observation fails.

use crate::support::indexed_access::hash;
use serde_json::json;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

pub(super) fn retain(project: &Path, runtime: &str, label: &str, output: &Output, profiled: bool) {
    let Some(root) = std::env::var_os("VIZE_MESSAGE_PROFILE_CAPTURE_DIR") else {
        return;
    };
    let directory = Path::new(&root).join(label);
    fs::create_dir_all(&directory).unwrap();
    let mut payloads = vec![
        ("stdout.json", output.stdout.clone()),
        ("stderr.txt", output.stderr.clone()),
        ("entry.ts", fs::read(project.join("src/entry.ts")).unwrap()),
        (
            "tsconfig.json",
            fs::read(project.join("tsconfig.json")).unwrap(),
        ),
    ];
    if profiled && let Ok(profile) = fs::read(project.join("profile.json")) {
        payloads.push(("profile.json", profile));
    }
    let mut entries = Vec::new();
    for (name, bytes) in payloads {
        fs::write(directory.join(name), &bytes).unwrap();
        entries.push(json!({"path":name,"bytes":bytes.len(),"sha256":hash(&bytes)}));
    }
    let provider = fs::canonicalize(runtime).unwrap();
    let provider_bytes = fs::read(&provider).unwrap();
    let version = Command::new(&provider).arg("--version").output().unwrap();
    let receipt = json!({
        "status":output.status.code(), "files":entries,
        "provider":{
            "path":provider,"bytes":provider_bytes.len(),"sha256":hash(&provider_bytes),
            "version_status":version.status.code(),"version_stdout":std::str::from_utf8(&version.stdout).unwrap(),
            "version_stderr":std::str::from_utf8(&version.stderr).unwrap()
        },
        "source":std::env::var("GITHUB_SHA").ok(), "run":std::env::var("GITHUB_RUN_ID").ok(),
        "attempt":std::env::var("GITHUB_RUN_ATTEMPT").ok()
    });
    fs::write(
        directory.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt).unwrap(),
    )
    .unwrap();
}

#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
//! `vize lib` through the real binary against a fixture registry (no network).

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use sha2::{Digest, Sha256};

fn hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| vize::carton::cstr!("{byte:02x}"))
        .collect::<Vec<_>>()
        .concat()
}

/// Minimal registry: `button` depends on `primitive`.
fn write_registry(dir: &Path) {
    let files = [
        (
            "primitive",
            "foundations/primitive/primitive.ts",
            "export const as = \"div\";\n",
        ),
        (
            "button",
            "families/button/button.ts",
            "import { as } from \"../../foundations/primitive/primitive.ts\";\nexport const tag = as;\n",
        ),
    ];
    let mut items = Vec::new();
    for (name, path, content) in files {
        let target = dir.join("files").join(path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, content).unwrap();
        let sha = hex(content.as_bytes());
        let content_hash = hex(vize::carton::cstr!("{path}\0{sha}\n").as_bytes());
        let deps: &[&str] = if name == "button" {
            &["primitive"]
        } else {
            &[]
        };
        items.push(serde_json::json!({
            "name": name, "kind": "ui", "title": name, "description": "fixture",
            "aliases": [], "packageSubpath": vize::carton::cstr!("./{name}").as_str(), "entry": path,
            "files": [{ "path": path, "role": "entry", "sha256": sha, "size": content.len() }],
            "registryDependencies": deps, "dependencies": [], "contentHash": content_hash,
        }));
    }
    items.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    let manifest = serde_json::json!({
        "schemaVersion": 1, "registryKind": "vize-lib",
        "package": { "name": "@vizejs/ui", "version": "0.0.1" },
        "kind": "ui", "filesDirectory": "files",
        "defaultTargetDirectory": "src/components/vize", "items": items,
    });
    fs::write(
        dir.join("registry.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

fn vize(root: &Path, registry: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .arg("lib")
        .arg("--root")
        .arg(root)
        .arg("--registry")
        .arg(registry)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn pull_status_and_remove_round_trip_through_the_binary() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("app");
    let registry = temp.path().join("registry");
    fs::create_dir_all(&root).unwrap();
    write_registry(&registry);

    let pulled = vize(&root, &registry, &["pull", "button"]);
    assert!(
        pulled.status.success(),
        "{}",
        String::from_utf8_lossy(&pulled.stderr)
    );
    assert!(
        root.join("src/components/vize/families/button/button.ts")
            .is_file()
    );
    assert!(
        root.join("src/components/vize/foundations/primitive/primitive.ts")
            .is_file()
    );

    let status = vize(&root, &registry, &["--json", "status"]);
    let status: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(status["items"][0]["name"], "button");
    assert_eq!(status["items"][0]["state"], "clean");
    assert_eq!(status["items"][1]["direct"], false);

    let lock: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("vize-lib.lock.json")).unwrap()).unwrap();
    assert_eq!(lock["lockfileVersion"], 1);
    assert_eq!(lock["items"][0]["version"], "0.0.1");

    let failed = vize(&root, &registry, &["pull", "missing"]);
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("error: no registry item matches"));

    let removed = vize(&root, &registry, &["remove", "button"]);
    assert!(
        removed.status.success(),
        "{}",
        String::from_utf8_lossy(&removed.stderr)
    );
    assert!(!root.join("src/components/vize").exists());
    assert!(!root.join("vize-lib.lock.json").exists());
}

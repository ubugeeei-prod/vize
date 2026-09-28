//! Build stage dumps must reflect the same product compile as emitted JS.
#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn temp_project_dir(test_name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vize-build-dump-{}-{}-{}",
        std::process::id(),
        test_name,
        nonce
    ))
}

fn write_batch(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    for name in ["a", "b", "c"] {
        fs::write(
            root.join("src").join(format!("{name}.vue")),
            format!("<template><div>{name}</div></template>\n"),
        )
        .unwrap();
    }
}

fn vize(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

fn sorted_entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("directory exists")
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

fn stderr_lines(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .map(str::to_owned)
        .collect()
}

#[test]
fn build_dump_uses_the_same_product_compile_without_changing_output() {
    let root = temp_project_dir("dump-dir");
    write_batch(&root);
    let plain = vize(&root, &["build", "src", "--output", "plain"]);
    assert_eq!(plain.status.code(), Some(0));
    let output = vize(
        &root,
        &[
            "build",
            "src",
            "--output",
            "dist",
            "--dump-dir",
            "dumps",
            "--dump-after-change",
        ],
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(sorted_entries(&root.join("dist")), ["a.js", "b.js", "c.js"]);
    assert_eq!(
        sorted_entries(&root.join("dumps")),
        ["a.vue", "b.vue", "c.vue"]
    );
    for name in ["a", "b", "c"] {
        assert_eq!(
            fs::read(root.join("dist").join(format!("{name}.js"))).unwrap(),
            fs::read(root.join("plain").join(format!("{name}.js"))).unwrap(),
        );
        let dir = root.join("dumps").join(format!("{name}.vue"));
        let feed: serde_json::Value =
            serde_json::from_slice(&fs::read(dir.join("stages.json")).unwrap()).unwrap();
        assert_eq!(feed["schema_version"], 2);
        assert_eq!(feed["command"], "vize-build");
        assert_eq!(feed["outcome"]["kind"], "accepted");
        assert_eq!(feed["source"]["path"], format!("{name}.vue"));
        assert!(!feed["pages"].as_array().unwrap().is_empty());
        assert_eq!(
            sorted_entries(&dir).len(),
            feed["pages"].as_array().unwrap().len() + 1,
        );
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn build_dump_records_legacy_selection_without_native_pages() {
    let root = temp_project_dir("dump-legacy");
    write_batch(&root);
    let accepted = vize(
        &root,
        &["build", "src", "--output", "native", "--dump-dir", "dumps"],
    );
    assert_eq!(accepted.status.code(), Some(0));
    assert!(sorted_entries(&root.join("dumps/a.vue")).len() > 1);
    let output = vize(
        &root,
        &[
            "build",
            "src",
            "--output",
            "dist",
            "--template-syntax",
            "quirks",
            "--dump-dir",
            "dumps",
            "--dump-after-change",
        ],
    );
    assert_eq!(output.status.code(), Some(0));
    let dir = root.join("dumps/a.vue");
    assert_eq!(sorted_entries(&dir), ["stages.json"]);
    let feed: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("stages.json")).unwrap()).unwrap();
    assert_eq!(feed["outcome"]["kind"], "legacy");
    assert!(feed["pages"].as_array().unwrap().is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn stats_build_dump_observes_each_file_without_writing_modules() {
    let root = temp_project_dir("dump-stats");
    write_batch(&root);
    let output = vize(
        &root,
        &["build", "src", "--format", "stats", "--dump-dir", "dumps"],
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        sorted_entries(&root.join("dumps")),
        ["a.vue", "b.vue", "c.vue"]
    );
    assert!(!root.join("dist").exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn dump_write_failure_does_not_emit_a_fallback_module() {
    let root = temp_project_dir("dump-write-failure");
    write_batch(&root);
    fs::create_dir_all(root.join("dumps/a.vue/stages.json")).unwrap();
    let output = vize(
        &root,
        &[
            "build",
            "src",
            "--output",
            "dist",
            "--dump-dir",
            "dumps",
            "--continue-on-error",
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(sorted_entries(&root.join("dist")), ["b.js", "c.js"]);
    assert!(
        stderr_lines(&output)
            .iter()
            .any(|line| line == "  \x1b[31mDump errors (1):\x1b[0m")
    );
    let _ = fs::remove_dir_all(root);
}

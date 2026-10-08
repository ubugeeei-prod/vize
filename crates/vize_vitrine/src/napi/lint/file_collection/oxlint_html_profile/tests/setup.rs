use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::json;

use super::{CORPUS, Case, custody};

pub(super) fn source_bytes(value: &str) -> Vec<u8> {
    match value {
        "@scripted" => fs::read(Path::new(CORPUS).join("scripted.html")).unwrap(),
        "@scriptless" => fs::read(Path::new(CORPUS).join("scriptless.htm")).unwrap(),
        "@clean" => fs::read(Path::new(CORPUS).join("clean.html")).unwrap(),
        other => other.as_bytes().to_vec(),
    }
}

fn write(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn command(
    root: &Path,
    program: &str,
    args: &[&str],
    journal: &mut Vec<serde_json::Value>,
    receipt: &mut custody::Receipt,
) {
    let output = match Command::new(program).args(args).current_dir(root).output() {
        Ok(output) => output,
        Err(error) => {
            journal.push(json!({"program":program,"args":args,"cwd":root,"spawn_error":vize_l0::cstr!("{error}")}));
            receipt.setup(journal, false);
            panic!("owned setup spawn failed: {program} {args:?}: {error}");
        }
    };
    journal.push(json!({"program": program, "args": args, "cwd": root,
        "status": output.status.code(), "stdout": output.stdout, "stderr": output.stderr}));
    receipt.setup(journal, output.status.success());
    assert!(
        output.status.success(),
        "owned setup failed: {program} {args:?}"
    );
}

pub(super) fn setup(
    case: &Case,
    root: &Path,
    journal: &mut Vec<serde_json::Value>,
    receipt: &mut custody::Receipt,
) -> (PathBuf, PathBuf) {
    fs::create_dir(root).unwrap();
    command(root, "git", &["init", "-q"], journal, receipt);
    // Freeze this active policy source independently of the installed Git
    // template. Cases with info-exclude patterns overwrite these authored bytes.
    write(&root.join(".git/info/exclude"), b"");
    for (name, bytes) in &case.files {
        write(&root.join(name), &source_bytes(bytes));
    }
    let cwd = root.join(&case.cwd);
    fs::create_dir_all(&cwd).unwrap();
    let config = cwd.join(".oxlintrc.json");
    write(&config, case.config.as_deref().unwrap_or("{}\n").as_bytes());
    if let Some(bytes) = &case.global_exclude {
        let source = root.join("configured-global-ignore");
        write(&source, bytes.as_bytes());
        command(
            root,
            "git",
            &[
                "config",
                "--local",
                "core.excludesFile",
                source.to_str().unwrap(),
            ],
            journal,
            receipt,
        );
        // Genuine Git proves this owned configured source actually ignores the
        // control. The producer still selects it with git_global(false).
        command(
            root,
            "git",
            &["check-ignore", "--no-index", "--", "src/Global.html"],
            journal,
            receipt,
        );
    }
    for path in &case.force_track {
        command(root, "git", &["add", "-f", "--", path], journal, receipt);
        command(
            root,
            "git",
            &["ls-files", "--error-unmatch", "--", path],
            journal,
            receipt,
        );
    }
    if case.linked_git {
        fs::remove_dir_all(root.join(".git")).unwrap();
        write(&root.join(".git"), b"gitdir: deliberately-unqualified\n");
    }
    if case.no_git {
        fs::remove_dir_all(root.join(".git")).unwrap();
    }
    #[cfg(unix)]
    for (name, destination) in &case.symlinks {
        let path = root.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        if let Ok(metadata) = fs::symlink_metadata(&path) {
            if metadata.is_dir() {
                fs::remove_dir_all(&path).unwrap();
            } else {
                fs::remove_file(&path).unwrap();
            }
        }
        std::os::unix::fs::symlink(destination, path).unwrap();
    }
    if let Some(name) = &case.fifo {
        command(root, "mkfifo", &[name], journal, receipt);
    }
    (cwd, config)
}

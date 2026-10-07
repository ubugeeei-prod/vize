//! Byte-preserving version integration and registry catalog custody.
use super::super::pr_github as github;
use std::{path::Path, process::Command};

pub(super) fn bytes(args: &[&str], root: &Path) -> Result<Vec<u8>, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(output.stdout)
}

pub(super) fn text(revision: &str, path: &str, root: &Path) -> Result<String, String> {
    String::from_utf8(bytes(&["show", &format!("{revision}:{path}")], root)?)
        .map_err(|e| e.to_string())
}

pub(super) fn paths(revision: &str, root: &Path) -> Result<Vec<String>, String> {
    let raw = bytes(&["ls-tree", "-r", "--name-only", "-z", revision], root)?;
    raw.split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .map(|p| String::from_utf8(p.to_vec()).map_err(|e| e.to_string()))
        .collect()
}

/// Mirrors the official preparation transformations. Everything outside the
/// version fields remains byte-identical, including newline and file modes.
pub(super) fn rewrite(path: &str, content: &str, old: &str, new: &str) -> String {
    let package_json = path.ends_with("/package.json")
        && (path.starts_with("npm/")
            || matches!(
                path,
                "editors/vscode/package.json" | "editors/vscode-art/package.json"
            ));
    let readme = path == "README.md" || (path.starts_with("npm/") && path.ends_with("/README.md"));
    let cargo_lock = path == "Cargo.lock"
        || path == "editors/zed/Cargo.lock"
        || path == "examples/volt-target/Cargo.lock"
        || (path.starts_with("davinci/vize_extension_host/tests/guests/")
            && path.ends_with("/Cargo.lock"));
    let mut section = "";
    let mut package = "";
    let mut replaced = false;
    let mut benchmark = false;
    let mut catalog = false;
    content
        .split('\n')
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                section = trimmed;
            }
            if cargo_lock && trimmed == "[[package]]" {
                package = "";
            }
            if cargo_lock
                && let Some(name) = trimmed
                    .strip_prefix("name = \"")
                    .and_then(|s| s.strip_suffix('"'))
            {
                package = name;
            }
            if readme {
                if line.contains("<!-- benchmark:readme:start -->") {
                    benchmark = true;
                }
                let result = if benchmark {
                    line.into()
                } else {
                    line.replace(old, new)
                };
                if line.contains("<!-- benchmark:readme:end -->") {
                    benchmark = false;
                }
                return result;
            }
            if path == "Cargo.toml"
                && section == "[workspace.package]"
                && trimmed == format!("version = \"{old}\"")
            {
                return line.replacen(old, new, 1);
            }
            if path == "Cargo.toml"
                && section == "[workspace.dependencies]"
                && (trimmed.starts_with("vize_") || line.contains("package = \"vize_"))
                && line.contains(&format!("version = \"={old}\""))
            {
                return line.replace(
                    &format!("version = \"={old}\""),
                    &format!("version = \"={new}\""),
                );
            }
            if package_json && !replaced && trimmed.starts_with("\"version\"") && line.contains(old)
            {
                replaced = true;
                return line.replacen(old, new, 1);
            }
            if matches!(
                path,
                "editors/zed/Cargo.toml" | "editors/zed/extension.toml"
            ) && !replaced
                && trimmed.starts_with("version = \"")
                && trimmed.ends_with('"')
                && line.contains(old)
            {
                replaced = true;
                return line.replacen(old, new, 1);
            }
            if cargo_lock
                && (package == "vize"
                    || package.starts_with("vize_")
                    || matches!(package, "davinci_harness" | "davinci_test_support")
                    || package == "vize-zed-extension")
                && trimmed == format!("version = \"{old}\"")
            {
                return line.replacen(old, new, 1);
            }
            if path == "pnpm-workspace.yaml"
                && line.starts_with("  - \"@vizejs/native-")
                && line.contains(old)
            {
                return if line.contains(new) {
                    line.into()
                } else {
                    line.replacen(old, &format!("{old} || {new}"), 1)
                };
            }
            if matches!(path, "pnpm-workspace.yaml" | "pnpm-lock.yaml") {
                if line == "  native-binaries:" {
                    catalog = true;
                } else if catalog
                    && !line.is_empty()
                    && !line.starts_with("    ")
                    && !(path == "pnpm-workspace.yaml" && line.starts_with("  #"))
                {
                    catalog = false;
                }
                if catalog
                    && line.contains(old)
                    && ((path == "pnpm-workspace.yaml"
                        && line.starts_with("    \"@vizejs/native-"))
                        || (path == "pnpm-lock.yaml"
                            && (line.starts_with("      specifier: ")
                                || line.starts_with("      version: "))))
                {
                    return line.replacen(old, new, 1);
                }
            }
            line.into()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn verify_delta(
    parent: &str,
    head: &str,
    old: &str,
    new: &str,
    root: &Path,
) -> Result<(), String> {
    if github::version_text(&text(parent, "Cargo.toml", root)?)? != old
        || github::version_text(&text(head, "Cargo.toml", root)?)? != new
    {
        return Err(
            "The version integration must bump the original base version exactly once.".into(),
        );
    }
    let raw = bytes(
        &["diff", "--raw", "--no-renames", "--no-abbrev", parent, head],
        root,
    )?;
    let changes = String::from_utf8(raw).map_err(|e| e.to_string())?;
    if changes.is_empty() {
        return Err("An empty version integration is not delivery proof.".into());
    }
    for row in changes.lines() {
        let (metadata, path) = row.split_once('\t').ok_or("Malformed version diff")?;
        let columns: Vec<_> = metadata.split_whitespace().collect();
        if columns.len() != 5
            || columns[0] != ":100644"
            || columns[1] != "100644"
            || columns[4] != "M"
        {
            return Err(format!(
                "Version integration changed a mode, added/deleted a file, or renamed {path}."
            ));
        }
        let before = text(parent, path, root)?;
        let expected = rewrite(path, &before, old, new);
        let observed = text(head, path, root)?;
        if expected == before || observed != expected {
            return Err(format!(
                "{path} differs from the exact generated version transformation."
            ));
        }
    }
    // Check every eligible tracked file, not only the presented diff. Omitting
    // a version pin is just as invalid as smuggling a non-version edit.
    for path in paths(parent, root)? {
        if !(path == "Cargo.toml"
            || path.ends_with("Cargo.lock")
            || path.ends_with("package.json")
            || path.ends_with("README.md")
            || path == "editors/zed/extension.toml"
            || path == "editors/zed/Cargo.toml"
            || path == "pnpm-workspace.yaml"
            || path == "pnpm-lock.yaml")
        {
            continue;
        }
        let before = text(parent, &path, root)?;
        let expected = rewrite(&path, &before, old, new);
        if expected != before && text(head, &path, root)? != expected {
            return Err(format!(
                "Version integration omitted or altered generated metadata in {path}."
            ));
        }
    }
    Ok(())
}

pub(super) use super::catalog::catalog;

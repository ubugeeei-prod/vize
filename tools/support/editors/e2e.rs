#![allow(dead_code)]

use serde_json::json;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[path = "../common.rs"]
mod common;

pub fn vscode_extension_path(repo_root: &Path) -> PathBuf {
    repo_root.join("editors/vscode")
}

pub fn real_vue_fixture_path(repo_root: &Path) -> PathBuf {
    vscode_extension_path(repo_root).join("test-fixtures/extension-host/real-vue")
}

pub fn prepare_real_vue_workspace(
    workspace_path: &Path,
    preserve_existing: bool,
) -> Result<PathBuf, String> {
    let repo_root = common::repo_root()?;
    if !preserve_existing && workspace_path.exists() {
        fs::remove_dir_all(workspace_path)
            .map_err(|error| format!("cannot remove {}: {error}", workspace_path.display()))?;
    }
    if let Some(parent) = workspace_path.parent() {
        common::mkdir(parent)?;
    }
    common::mkdir(workspace_path)?;
    copy_dir_contents(&real_vue_fixture_path(&repo_root), workspace_path)?;
    enable_ts_extension_imports(&workspace_path.join("tsconfig.json"))?;
    common::mkdir(workspace_path.join("node_modules"))?;
    let vue_link = workspace_path.join("node_modules/vue");
    if vue_link.exists() {
        remove_path(&vue_link)?;
    }
    symlink_dir(&resolve_vue_package_path(&repo_root)?, &vue_link)?;
    common::write_json_pretty(
        workspace_path.join("vize.config.json"),
        &json!({ "typeChecker": { "corsaPath": resolve_corsa_path(&repo_root)? } }),
    )?;
    Ok(workspace_path.to_path_buf())
}

pub fn resolve_real_server_path(repo_root: &Path) -> Result<PathBuf, String> {
    let exe = if cfg!(windows) { "vize.exe" } else { "vize" };
    let candidates = if let Ok(configured) = env::var("VIZE_SERVER_PATH") {
        vec![PathBuf::from(configured)]
    } else {
        ["ci", "release", "debug"]
            .into_iter()
            .map(|profile| repo_root.join("target").join(profile).join(exe))
            .collect()
    };
    for candidate in &candidates {
        if candidate.exists() {
            return candidate
                .canonicalize()
                .map_err(|error| format!("cannot canonicalize {}: {error}", candidate.display()));
        }
    }
    Err(format!(
        "missing real vize server binary (checked {}). Build one with `cargo build --profile ci -p vize` or set VIZE_SERVER_PATH.",
        candidates
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

pub fn resolve_corsa_path(repo_root: &Path) -> Result<PathBuf, String> {
    let extension = vscode_extension_path(repo_root);
    let package = format!("@typescript/typescript-{}-{}", node_platform(), node_arch());
    let direct = extension
        .join("node_modules")
        .join(package.replace('/', std::path::MAIN_SEPARATOR_STR))
        .join("lib")
        .join(if cfg!(windows) { "tsc.exe" } else { "tsc" });
    if direct.exists() {
        return direct
            .canonicalize()
            .map_err(|error| format!("cannot canonicalize {}: {error}", direct.display()));
    }
    if let Some(found) = find_file_named(
        &extension.join("node_modules"),
        if cfg!(windows) { "tsc.exe" } else { "tsc" },
    )? {
        if found.components().any(|component| {
            component
                .as_os_str()
                .to_string_lossy()
                .contains("@typescript")
        }) {
            return found
                .canonicalize()
                .map_err(|error| format!("cannot canonicalize {}: {error}", found.display()));
        }
    }
    Err(format!(
        "missing TypeScript 7 runtime (checked {})",
        direct.display()
    ))
}

pub fn resolve_vue_package_path(repo_root: &Path) -> Result<PathBuf, String> {
    for candidate in [
        repo_root.join("tests/node_modules/vue/package.json"),
        repo_root.join("node_modules/vue/package.json"),
    ] {
        if candidate.exists() {
            return candidate
                .parent()
                .unwrap()
                .canonicalize()
                .map_err(|error| format!("cannot canonicalize {}: {error}", candidate.display()));
        }
    }
    let script = "console.log(require.resolve('vue/package.json'))";
    let output = common::run_capture_in("node", &["-e", script], repo_root.join("tests"))?;
    let manifest = PathBuf::from(output.stdout.trim());
    manifest
        .parent()
        .ok_or_else(|| "resolved vue manifest has no parent".to_string())?
        .canonicalize()
        .map_err(|error| format!("cannot canonicalize {}: {error}", manifest.display()))
}

fn enable_ts_extension_imports(tsconfig_path: &Path) -> Result<(), String> {
    let mut value = common::read_json(tsconfig_path)?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| format!("{} must contain a JSON object", tsconfig_path.display()))?;
    let compiler_options = object
        .entry("compilerOptions")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| "compilerOptions must be an object".to_string())?;
    compiler_options.insert("allowImportingTsExtensions".to_string(), json!(true));
    common::write_json_pretty(tsconfig_path, &value)
}

fn copy_dir_contents(from: &Path, to: &Path) -> Result<(), String> {
    let mut entries = fs::read_dir(from)
        .map_err(|error| format!("cannot read {}: {error}", from.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot read {}: {error}", from.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let target = to.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|error| format!("cannot stat {}: {error}", entry.path().display()))?;
        if file_type.is_dir() {
            common::mkdir(&target)?;
            copy_dir_contents(&entry.path(), &target)?;
        } else if file_type.is_file() {
            if let Some(parent) = target.parent() {
                common::mkdir(parent)?;
            }
            fs::copy(entry.path(), &target)
                .map_err(|error| format!("cannot copy {}: {error}", target.display()))?;
        }
    }
    Ok(())
}

fn remove_path(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("cannot stat {}: {error}", path.display()))?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path)
            .map_err(|error| format!("cannot remove {}: {error}", path.display()))
    } else {
        fs::remove_file(path).map_err(|error| format!("cannot remove {}: {error}", path.display()))
    }
}

#[cfg(unix)]
fn symlink_dir(from: &Path, to: &Path) -> Result<(), String> {
    std::os::unix::fs::symlink(from, to).map_err(|error| {
        format!(
            "cannot symlink {} -> {}: {error}",
            to.display(),
            from.display()
        )
    })
}

#[cfg(windows)]
fn symlink_dir(from: &Path, to: &Path) -> Result<(), String> {
    std::os::windows::fs::symlink_dir(from, to).map_err(|error| {
        format!(
            "cannot symlink {} -> {}: {error}",
            to.display(),
            from.display()
        )
    })
}

fn find_file_named(root: &Path, name: &str) -> Result<Option<PathBuf>, String> {
    if !root.exists() {
        return Ok(None);
    }
    let mut entries = fs::read_dir(root)
        .map_err(|error| format!("cannot read {}: {error}", root.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot read {}: {error}", root.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("cannot stat {}: {error}", path.display()))?;
        if file_type.is_file() && entry.file_name() == name {
            return Ok(Some(path));
        }
        if file_type.is_dir() {
            if let Some(found) = find_file_named(&path, name)? {
                return Ok(Some(found));
            }
        }
    }
    Ok(None)
}

fn node_platform() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        "windows" => "win32",
        "linux" => "linux",
        other => other,
    }
}

fn node_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    }
}

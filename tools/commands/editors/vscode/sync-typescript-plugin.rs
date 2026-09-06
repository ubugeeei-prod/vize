#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

#[path = "../../../support/common.rs"]
mod common;

const USAGE: &str = "Usage: rust-script tools/commands/editors/vscode/sync-typescript-plugin.rs <stage|inject> [vsix]";
const PACKAGE_PATH: &str = "node_modules/@vizejs/typescript-vue-plugin";
const PLUGIN_FILES: [&str; 6] = [
    "component-contracts.cjs",
    "import-resolution.cjs",
    "index.cjs",
    "module-resolution.cjs",
    "package.json",
    "virtual-modules.cjs",
];

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let root = common::repo_root()?;
    let extension_dir = root.join("editors/vscode");
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("stage") => stage_plugin(
            &extension_dir.join("typescript-vue-plugin"),
            &extension_dir.join(PACKAGE_PATH),
        ),
        Some("inject") => {
            let vsix = args.get(1).map(PathBuf::from).unwrap_or_else(|| {
                env::current_dir()
                    .unwrap_or_else(|_| root.clone())
                    .join("dist/vize.vsix")
            });
            inject_plugin(&extension_dir.join("typescript-vue-plugin"), &vsix)
        }
        Some("--help" | "-h") => {
            println!("{USAGE}");
            Ok(())
        }
        Some(other) => Err(format!("unknown command {other}\n\n{USAGE}")),
        None => Err(USAGE.to_string()),
    }
}

fn stage_plugin(source_dir: &Path, target_dir: &Path) -> Result<(), String> {
    validate_plugin_files(source_dir)?;
    let parent = target_dir
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", target_dir.display()))?;
    common::mkdir(parent)?;
    let temp_dir = unique_sibling_dir(parent, ".typescript-vue-plugin-stage")?;
    let backup_dir = unique_sibling_dir(parent, ".typescript-vue-plugin-backup")?;
    common::mkdir(&temp_dir)?;

    let result = (|| {
        copy_plugin_files(source_dir, &temp_dir)?;
        replace_staged_plugin(&temp_dir, target_dir, &backup_dir)
    })();

    if result.is_err() {
        let _ = fs::remove_dir_all(&temp_dir);
    }
    result
}

fn validate_plugin_files(source_dir: &Path) -> Result<(), String> {
    if !source_dir.is_dir() {
        return Err(format!(
            "TypeScript Vue plugin source directory does not exist: {}",
            source_dir.display()
        ));
    }
    for file in PLUGIN_FILES {
        let path = source_dir.join(file);
        if !path.is_file() {
            return Err(format!(
                "TypeScript Vue plugin source file is missing: {}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn copy_plugin_files(source_dir: &Path, target_dir: &Path) -> Result<(), String> {
    for file in PLUGIN_FILES {
        fs::copy(source_dir.join(file), target_dir.join(file)).map_err(|error| {
            format!(
                "cannot copy {} to {}: {error}",
                source_dir.join(file).display(),
                target_dir.join(file).display()
            )
        })?;
    }
    Ok(())
}

fn replace_staged_plugin(
    temp_dir: &Path,
    target_dir: &Path,
    backup_dir: &Path,
) -> Result<(), String> {
    if target_dir.exists() {
        fs::rename(target_dir, backup_dir).map_err(|error| {
            format!(
                "cannot move {} to {}: {error}",
                target_dir.display(),
                backup_dir.display()
            )
        })?;
    }

    if let Err(error) = fs::rename(temp_dir, target_dir) {
        if backup_dir.exists() {
            let _ = fs::rename(backup_dir, target_dir);
        }
        return Err(format!(
            "cannot move staged plugin {} to {}: {error}",
            temp_dir.display(),
            target_dir.display()
        ));
    }

    if backup_dir.exists() {
        fs::remove_dir_all(backup_dir)
            .map_err(|error| format!("cannot remove {}: {error}", backup_dir.display()))?;
    }
    Ok(())
}

fn unique_sibling_dir(parent: &Path, prefix: &str) -> Result<PathBuf, String> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    Ok(parent.join(format!("{prefix}-{}-{nanos}", std::process::id())))
}

fn inject_plugin(source_dir: &Path, vsix_path: &Path) -> Result<(), String> {
    let vsix_path = absolute_from_cwd(vsix_path)?;
    if !vsix_path.exists() {
        return Err(format!("VSIX does not exist: {}", vsix_path.display()));
    }
    let temp_dir = unique_temp_dir("vize-vsix-plugin")?;
    let target_dir = temp_dir.join("extension").join(PACKAGE_PATH);
    let result = (|| {
        stage_plugin(source_dir, &target_dir)?;
        let entries = PLUGIN_FILES
            .iter()
            .map(|file| format!("extension/{PACKAGE_PATH}/{file}"))
            .collect::<Vec<_>>();
        let output = Command::new("zip")
            .args(["-X", "-q"])
            .arg(&vsix_path)
            .args(&entries)
            .current_dir(&temp_dir)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| format!("failed to run zip: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "zip failed with status {}\n{}{}",
                output.status.code().unwrap_or(1),
                String::from_utf8_lossy(&output.stderr),
                String::from_utf8_lossy(&output.stdout)
            ));
        }
        Ok(())
    })();
    let _ = fs::remove_dir_all(&temp_dir);
    result
}

fn absolute_from_cwd(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    env::current_dir()
        .map(|cwd| cwd.join(path))
        .map_err(|error| format!("cannot read current dir: {error}"))
}

fn unique_temp_dir(prefix: &str) -> Result<PathBuf, String> {
    let mut path = env::temp_dir();
    path.push(format!(
        "{}-{}-{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos()
    ));
    common::mkdir(&path)?;
    Ok(path)
}

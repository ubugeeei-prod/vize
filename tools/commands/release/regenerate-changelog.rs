#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! tempfile = "3"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../support/common.rs"]
mod common;

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

const GIT_CLIFF_VERSION: &str = "2.6.1";

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let raw_args = env::args().skip(1).collect::<Vec<_>>();
    let (print_args, args) = parse_git_cliff_args(&raw_args)?;
    if print_args {
        println!(
            "{}",
            serde_json::to_string(&args).map_err(|error| error.to_string())?
        );
        return Ok(());
    }

    let root = common::repo_root()?;
    let cliff = ensure_git_cliff()?;
    let status = Command::new(&cliff)
        .args(&args)
        .current_dir(&root)
        .stdin(Stdio::null())
        .status()
        .map_err(|error| format!("failed to run {}: {error}", cliff.display()))?;
    if !status.success() {
        return Err(format!(
            "git-cliff failed with exit {}",
            status.code().unwrap_or(1)
        ));
    }
    let changelog = root.join("CHANGELOG.md");
    if !changelog.is_file() {
        return Err("git-cliff finished but CHANGELOG.md is missing.".to_string());
    }
    Ok(())
}

fn parse_git_cliff_args(argv: &[String]) -> Result<(bool, Vec<String>), String> {
    let mut print_args = false;
    let mut args = vec![
        "--config".to_string(),
        "cliff.toml".to_string(),
        "--output".to_string(),
        "CHANGELOG.md".to_string(),
    ];
    let mut index = 0usize;
    while index < argv.len() {
        match argv[index].as_str() {
            "--print-args" => {
                print_args = true;
                index += 1;
            }
            "--unreleased" | "--latest" => {
                args.push(argv[index].clone());
                index += 1;
            }
            "--tag" => {
                let Some(tag) = argv.get(index + 1) else {
                    return Err("Missing value for --tag".to_string());
                };
                if tag.is_empty() || tag.starts_with("--") {
                    return Err("Missing value for --tag".to_string());
                }
                args.push("--tag".to_string());
                args.push(tag.clone());
                index += 2;
            }
            arg => return Err(format!("Unknown argument: {arg}")),
        }
    }
    Ok((print_args, args))
}

fn ensure_git_cliff() -> Result<PathBuf, String> {
    if Command::new("git-cliff")
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
    {
        return Ok(PathBuf::from("git-cliff"));
    }

    let asset = resolve_asset()?;
    let url = format!(
        "https://github.com/orhun/git-cliff/releases/download/v{GIT_CLIFF_VERSION}/{asset}"
    );
    let scratch = tempfile::Builder::new()
        .prefix("vize-git-cliff-")
        .tempdir()
        .map_err(|error| error.to_string())?
        .keep();
    let archive = scratch.join(&asset);
    download_file(&url, &archive)?;

    let archive_text = archive.to_string_lossy().into_owned();
    let scratch_text = scratch.to_string_lossy().into_owned();
    if asset.ends_with(".tar.gz") {
        common::run_capture(
            "tar",
            &["-xzf", archive_text.as_str(), "-C", scratch_text.as_str()],
        )?;
    } else {
        common::run_capture(
            "unzip",
            &["-q", archive_text.as_str(), "-d", scratch_text.as_str()],
        )?;
    }

    let stem = asset
        .strip_suffix(".tar.gz")
        .or_else(|| asset.strip_suffix(".zip"))
        .unwrap_or(&asset);
    let bin = scratch.join(stem).join(if env::consts::OS == "windows" {
        "git-cliff.exe"
    } else {
        "git-cliff"
    });
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&bin, fs::Permissions::from_mode(0o755))
            .map_err(|error| format!("cannot chmod {}: {error}", bin.display()))?;
    }
    Ok(bin)
}

fn resolve_asset() -> Result<String, String> {
    match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => Ok(format!(
            "git-cliff-{GIT_CLIFF_VERSION}-x86_64-unknown-linux-gnu.tar.gz"
        )),
        ("linux", "aarch64") => Ok(format!(
            "git-cliff-{GIT_CLIFF_VERSION}-aarch64-unknown-linux-gnu.tar.gz"
        )),
        ("macos", "aarch64") => Ok(format!(
            "git-cliff-{GIT_CLIFF_VERSION}-aarch64-apple-darwin.tar.gz"
        )),
        ("macos", "x86_64") => Ok(format!(
            "git-cliff-{GIT_CLIFF_VERSION}-x86_64-apple-darwin.tar.gz"
        )),
        ("windows", "x86_64") => Ok(format!(
            "git-cliff-{GIT_CLIFF_VERSION}-x86_64-pc-windows-msvc.zip"
        )),
        (platform, arch) => Err(format!(
            "Unsupported platform/arch for git-cliff fallback: {platform}/{arch}"
        )),
    }
}

fn download_file(url: &str, dest: &Path) -> Result<(), String> {
    let output = Command::new("curl")
        .args([
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--output",
        ])
        .arg(dest)
        .arg(url)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run curl: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "curl failed for {url}\n{}{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        )
        .trim()
        .to_string());
    }
    Ok(())
}

#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! vize_tool_support = { path = "../../support" }
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! sha2 = "0.11"
//! tempfile = "3"
//!
//! [package]
//! edition = "2024"
//! ```

use sha2::{Digest, Sha512};
use vize_tool_support::common;

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

const GIT_CLIFF_VERSION: &str = "2.6.1";
const MAX_GIT_CLIFF_ARCHIVE_BYTES: u64 = 16 * 1024 * 1024;

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

    let (asset, expected_sha512) = resolve_asset()?;
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
    verify_sha512(&archive, expected_sha512)?;

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

fn resolve_asset() -> Result<(String, &'static str), String> {
    match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => Ok((
            format!("git-cliff-{GIT_CLIFF_VERSION}-x86_64-unknown-linux-gnu.tar.gz"),
            "b1e7a6b70fc794accdccd9717eff5a8bf8b507f0308da44d837d935be31b18b19db3758d1a9bb4975fbca7d01ddaae251229a753ae2cd2ecc24845cfc50d067f",
        )),
        ("linux", "aarch64") => Ok((
            format!("git-cliff-{GIT_CLIFF_VERSION}-aarch64-unknown-linux-gnu.tar.gz"),
            "6e8aafddd1e1d163290561cd595fa86971b034012b0602fd8f554f831158af56d3c8f9d82793b4e79d1300f6620091602d81b49110f96ca87068d5df6127cf57",
        )),
        ("macos", "aarch64") => Ok((
            format!("git-cliff-{GIT_CLIFF_VERSION}-aarch64-apple-darwin.tar.gz"),
            "d507021de298ceba43fd5cc1594c9b868888b5f8a685bc6910d31dc0902eb7b09e25de06a6ef20fb9fc98a505127e695e647747bc97a3697d626d54d6992b394",
        )),
        ("macos", "x86_64") => Ok((
            format!("git-cliff-{GIT_CLIFF_VERSION}-x86_64-apple-darwin.tar.gz"),
            "acc7e28da2af6337fe67b05003d5468cba4fc3014b2d51e7cf9f7f01e8ee11d4bc10845ba282694320648e7627af67bfe62d00a7e7b6f9faec8b2c38fbbd00a3",
        )),
        ("windows", "x86_64") => Ok((
            format!("git-cliff-{GIT_CLIFF_VERSION}-x86_64-pc-windows-msvc.zip"),
            "9da4c3bb4c407ded67e411544c4d4b0c2b748c20db5181b9da72aeb652efc8c939a0e07411886d4132d49581c02e8b7ecae1d127dcc4508159a2f70fea780e76",
        )),
        (platform, arch) => Err(format!(
            "Unsupported platform/arch for git-cliff fallback: {platform}/{arch}"
        )),
    }
}

fn download_file(url: &str, dest: &Path) -> Result<(), String> {
    let maximum_archive_bytes = MAX_GIT_CLIFF_ARCHIVE_BYTES.to_string();
    let output = Command::new("curl")
        .args([
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--max-redirs",
            "5",
            "--max-filesize",
            maximum_archive_bytes.as_str(),
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
    let metadata = fs::metadata(dest).map_err(|error| {
        format!(
            "cannot inspect downloaded archive {}: {error}",
            dest.display()
        )
    })?;
    if !metadata.is_file() || metadata.len() > MAX_GIT_CLIFF_ARCHIVE_BYTES {
        return Err(format!(
            "downloaded archive is not a regular file within the {MAX_GIT_CLIFF_ARCHIVE_BYTES}-byte limit"
        ));
    }
    Ok(())
}

fn verify_sha512(path: &Path, expected: &str) -> Result<(), String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("cannot read downloaded archive {}: {error}", path.display()))?;
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha512::digest(bytes);
    let mut actual = String::with_capacity(digest.len() * 2);
    for byte in digest {
        actual.push(char::from(HEX[usize::from(byte >> 4)]));
        actual.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    if actual != expected {
        return Err(format!("SHA-512 mismatch for {}", path.display()));
    }
    Ok(())
}

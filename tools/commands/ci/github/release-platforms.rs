#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../../rust/common.rs"]
mod common;

use serde::Serialize;
use serde_json::Value;
use std::{collections::BTreeSet, env, fs, path::Path, process::ExitCode};

#[derive(Clone, Debug, Serialize)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

#[derive(Clone, Debug, Serialize)]
struct CliPlatform {
    host: &'static str,
    target: &'static str,
    archive: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct NativePlatform {
    host: &'static str,
    target: &'static str,
    cross_compile: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReleasePlan {
    version: Version,
    include_slow_platforms: bool,
    skipped_targets: Vec<&'static str>,
    cli_matrix: Vec<CliPlatform>,
    native_matrix: Vec<NativePlatform>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CadenceResult {
    changed: bool,
    skipped_targets: Vec<String>,
}

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let [command, ref_name, rest @ ..] = args.as_slice() else {
        return Err(usage());
    };
    match command.as_str() {
        "github-output" => write_github_outputs(ref_name),
        "apply-cadence" => {
            if rest.is_empty() {
                return Err("apply-cadence requires at least one package.json path".to_string());
            }
            for package_json_path in rest {
                let result = apply_release_platform_cadence(ref_name, package_json_path)?;
                let action = if result.changed { "Applied" } else { "Skipped" };
                let skipped = if result.skipped_targets.is_empty() {
                    "none".to_string()
                } else {
                    result.skipped_targets.join(", ")
                };
                println!(
                    "{action} release platform cadence for {package_json_path}; skipped targets: {skipped}"
                );
            }
            Ok(())
        }
        "print" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&release_platform_plan(ref_name)?)
                    .map_err(|error| error.to_string())?
            );
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "Usage: rust-script tools/commands/ci/github/release-platforms.rs <github-output|apply-cadence|print> <ref-name> [...package-json]".to_string()
}

fn cli_release_platforms() -> Vec<CliPlatform> {
    vec![
        CliPlatform {
            host: "blacksmith-12vcpu-macos-15",
            target: "x86_64-apple-darwin",
            archive: "vize-x86_64-apple-darwin.tar.gz",
        },
        CliPlatform {
            host: "blacksmith-12vcpu-macos-15",
            target: "aarch64-apple-darwin",
            archive: "vize-aarch64-apple-darwin.tar.gz",
        },
        CliPlatform {
            host: "windows-2025",
            target: "x86_64-pc-windows-msvc",
            archive: "vize-x86_64-pc-windows-msvc.zip",
        },
        CliPlatform {
            host: "windows-2025",
            target: "aarch64-pc-windows-msvc",
            archive: "vize-aarch64-pc-windows-msvc.zip",
        },
        CliPlatform {
            host: "blacksmith-32vcpu-ubuntu-2404",
            target: "x86_64-unknown-linux-gnu",
            archive: "vize-x86_64-unknown-linux-gnu.tar.gz",
        },
        CliPlatform {
            host: "blacksmith-32vcpu-ubuntu-2404",
            target: "x86_64-unknown-linux-musl",
            archive: "vize-x86_64-unknown-linux-musl.tar.gz",
        },
        CliPlatform {
            host: "blacksmith-32vcpu-ubuntu-2404",
            target: "aarch64-unknown-linux-gnu",
            archive: "vize-aarch64-unknown-linux-gnu.tar.gz",
        },
        CliPlatform {
            host: "blacksmith-32vcpu-ubuntu-2404",
            target: "aarch64-unknown-linux-musl",
            archive: "vize-aarch64-unknown-linux-musl.tar.gz",
        },
    ]
}

fn native_release_platforms() -> Vec<NativePlatform> {
    vec![
        NativePlatform {
            host: "blacksmith-12vcpu-macos-15",
            target: "x86_64-apple-darwin",
            cross_compile: false,
        },
        NativePlatform {
            host: "blacksmith-12vcpu-macos-15",
            target: "aarch64-apple-darwin",
            cross_compile: false,
        },
        NativePlatform {
            host: "windows-2025",
            target: "x86_64-pc-windows-msvc",
            cross_compile: false,
        },
        NativePlatform {
            host: "windows-11-arm",
            target: "aarch64-pc-windows-msvc",
            cross_compile: false,
        },
        NativePlatform {
            host: "ubuntu-22.04",
            target: "x86_64-unknown-linux-gnu",
            cross_compile: false,
        },
        NativePlatform {
            host: "blacksmith-32vcpu-ubuntu-2404",
            target: "x86_64-unknown-linux-musl",
            cross_compile: true,
        },
        NativePlatform {
            host: "ubuntu-22.04-arm",
            target: "aarch64-unknown-linux-gnu",
            cross_compile: false,
        },
        NativePlatform {
            host: "blacksmith-32vcpu-ubuntu-2404-arm",
            target: "aarch64-unknown-linux-musl",
            cross_compile: true,
        },
    ]
}

fn parse_release_version(ref_name: &str) -> Result<Version, String> {
    let name = ref_name.strip_prefix('v').unwrap_or(ref_name);
    let stable = name.split_once('-').map(|(value, _)| value).unwrap_or(name);
    let parts = stable.split('.').collect::<Vec<_>>();
    let [major, minor, patch] = parts.as_slice() else {
        return Err(invalid_version(ref_name));
    };
    Ok(Version {
        major: major.parse().map_err(|_| invalid_version(ref_name))?,
        minor: minor.parse().map_err(|_| invalid_version(ref_name))?,
        patch: patch.parse().map_err(|_| invalid_version(ref_name))?,
    })
}

fn invalid_version(ref_name: &str) -> String {
    format!("Release tag must look like vMAJOR.MINOR.PATCH[-PRERELEASE], got {ref_name}")
}

fn release_platform_plan(ref_name: &str) -> Result<ReleasePlan, String> {
    Ok(ReleasePlan {
        version: parse_release_version(ref_name)?,
        include_slow_platforms: true,
        skipped_targets: Vec::new(),
        cli_matrix: cli_release_platforms(),
        native_matrix: native_release_platforms(),
    })
}

fn write_github_outputs(ref_name: &str) -> Result<(), String> {
    let plan = release_platform_plan(ref_name)?;
    common::append_github_outputs(&[
        (
            "include_slow_platforms",
            plan.include_slow_platforms.to_string(),
        ),
        ("release_minor", plan.version.minor.to_string()),
    ])?;
    common::append_github_multiline_output(
        "cli_matrix",
        &serde_json::to_string(&plan.cli_matrix).map_err(|error| error.to_string())?,
    )?;
    common::append_github_multiline_output(
        "native_matrix",
        &serde_json::to_string(&plan.native_matrix).map_err(|error| error.to_string())?,
    )?;

    if let Some(summary_path) = env::var_os("GITHUB_STEP_SUMMARY") {
        let skipped = if plan.skipped_targets.is_empty() {
            "none".to_string()
        } else {
            plan.skipped_targets.join(", ")
        };
        common::append_text(
            summary_path,
            &[
                "## Release platform cadence",
                "",
                &format!("- Ref: {ref_name}"),
                &format!("- Minor version: {}", plan.version.minor),
                &format!(
                    "- Slow targets enabled: {}",
                    if plan.include_slow_platforms {
                        "yes"
                    } else {
                        "no"
                    }
                ),
                &format!("- Skipped targets: {skipped}"),
                "",
            ]
            .join("\n"),
        )?;
    }
    Ok(())
}

fn apply_release_platform_cadence(
    ref_name: &str,
    package_json_path: &str,
) -> Result<CadenceResult, String> {
    let plan = release_platform_plan(ref_name)?;
    if plan.include_slow_platforms {
        return Ok(CadenceResult {
            changed: false,
            skipped_targets: Vec::new(),
        });
    }

    let path = Path::new(package_json_path);
    let mut package_json: Value = serde_json::from_str(&common::read_text(path)?)
        .map_err(|error| format!("cannot parse {}: {error}", path.display()))?;
    let skipped = plan
        .skipped_targets
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if let Some(optional) = package_json
        .get_mut("optionalDependencies")
        .and_then(Value::as_object_mut)
    {
        for target in &skipped {
            if let Some(package_name) = slow_native_package_name(target) {
                optional.remove(package_name);
            }
        }
    }
    if let Some(targets) = package_json
        .pointer_mut("/napi/targets")
        .and_then(Value::as_array_mut)
    {
        targets.retain(|target| {
            target
                .as_str()
                .map(|target| !skipped.contains(target))
                .unwrap_or(true)
        });
    }
    fs::write(
        path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&package_json).map_err(|error| error.to_string())?
        ),
    )
    .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    let package_dir = path.parent().unwrap_or_else(|| Path::new("."));
    for target in &skipped {
        if let Some(napi_dir) = slow_napi_package_dir(target) {
            let target_dir = package_dir.join("npm").join(napi_dir);
            match fs::remove_dir_all(&target_dir) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(format!("cannot remove {}: {error}", target_dir.display()));
                }
            }
        }
    }
    Ok(CadenceResult {
        changed: true,
        skipped_targets: plan
            .skipped_targets
            .into_iter()
            .map(str::to_string)
            .collect(),
    })
}

fn slow_native_package_name(target: &str) -> Option<&'static str> {
    match target {
        "x86_64-apple-darwin" => Some("@vizejs/native-darwin-x64"),
        "aarch64-pc-windows-msvc" => Some("@vizejs/native-win32-arm64-msvc"),
        _ => None,
    }
}

fn slow_napi_package_dir(target: &str) -> Option<&'static str> {
    match target {
        "x86_64-apple-darwin" => Some("darwin-x64"),
        "aarch64-pc-windows-msvc" => Some("win32-arm64-msvc"),
        _ => None,
    }
}

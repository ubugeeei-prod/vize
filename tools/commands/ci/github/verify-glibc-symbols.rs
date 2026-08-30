#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{
    collections::BTreeSet,
    env,
    path::Path,
    process::{Command, ExitCode, Stdio},
};

const DEFAULT_MAX_GLIBC: &str = "2.36";

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    fn text(&self) -> String {
        if self.patch == 0 {
            format!("{}.{}", self.major, self.minor)
        } else {
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

fn main() -> ExitCode {
    match verify() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn verify() -> Result<(), String> {
    let (max, files) = parse_args(env::args().skip(1))?;
    if files.is_empty() {
        return Err(
            "Usage: rust-script tools/commands/ci/github/verify-glibc-symbols.rs [--max 2.36] <file.node>..."
                .to_string(),
        );
    }

    let mut failed = false;
    for file in files {
        if !Path::new(&file).exists() {
            report_error(&format!("native binary does not exist: {file}"));
            failed = true;
            continue;
        }

        let versions = parse_versions(&readelf_version_info(&file)?);
        if versions.is_empty() {
            report_error(&format!(
                "native binary has no GLIBC_* version records: {file}"
            ));
            failed = true;
            continue;
        }
        if versions.iter().any(|version| version > &max) {
            let highest = versions.last().expect("nonempty versions");
            report_error(&format!(
                "{file} requires GLIBC_{}, above the supported ceiling GLIBC_{}",
                highest.text(),
                max.text()
            ));
            failed = true;
            continue;
        }
        let highest = versions.last().expect("nonempty versions");
        println!("{file}: GLIBC_{} <= GLIBC_{}", highest.text(), max.text());
    }

    if failed {
        Err("glibc symbol verification failed".to_string())
    } else {
        Ok(())
    }
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<(Version, Vec<String>), String> {
    let mut max = DEFAULT_MAX_GLIBC.to_string();
    let mut files = Vec::new();
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        if arg == "--max" {
            max = args
                .next()
                .ok_or_else(|| "--max requires a value".to_string())?;
        } else {
            files.push(arg);
        }
    }
    Ok((parse_version(&max)?, files))
}

fn readelf_version_info(file: &str) -> Result<String, String> {
    let output = Command::new("readelf")
        .args(["--version-info", file])
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run readelf: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "readelf --version-info failed for {file}\n{}{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        )
        .trim()
        .to_string());
    }
    Ok(format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn parse_versions(text: &str) -> BTreeSet<Version> {
    let mut versions = BTreeSet::new();
    let mut rest = text;
    while let Some(index) = rest.find("GLIBC_") {
        rest = &rest[index + "GLIBC_".len()..];
        let end = rest
            .find(|character: char| !character.is_ascii_digit() && character != '.')
            .unwrap_or(rest.len());
        if let Ok(version) = parse_version(&rest[..end]) {
            versions.insert(version);
        }
        rest = &rest[end..];
    }
    versions
}

fn parse_version(value: &str) -> Result<Version, String> {
    let parts = value.split('.').collect::<Vec<_>>();
    let ([major, minor] | [major, minor, _]) = parts.as_slice() else {
        return Err(format!(
            "glibc version must look like MAJOR.MINOR[.PATCH], got {value}"
        ));
    };
    Ok(Version {
        major: major.parse().map_err(|_| {
            format!("glibc version must look like MAJOR.MINOR[.PATCH], got {value}")
        })?,
        minor: minor.parse().map_err(|_| {
            format!("glibc version must look like MAJOR.MINOR[.PATCH], got {value}")
        })?,
        patch: parts
            .get(2)
            .map(|patch| patch.parse())
            .transpose()
            .map_err(|_| format!("glibc version must look like MAJOR.MINOR[.PATCH], got {value}"))?
            .unwrap_or(0),
    })
}

fn report_error(message: &str) {
    eprintln!("::error::{message}");
}

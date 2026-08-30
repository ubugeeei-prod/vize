#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{
    collections::BTreeSet,
    env,
    path::PathBuf,
    process::{Command, ExitCode, Stdio},
};

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
    let args = env::args().skip(1).collect::<Vec<_>>();
    let [target] = args.as_slice() else {
        return Err(
            "Usage: rust-script tools/commands/ci/github/verify-musl-cli-binary.rs <rust-target>"
                .to_string(),
        );
    };
    let binary = PathBuf::from("target")
        .join(target)
        .join("release")
        .join("vize");
    let binary_text = binary
        .to_str()
        .ok_or_else(|| format!("binary path is not valid UTF-8: {}", binary.display()))?;

    let file_status = Command::new("file")
        .arg(&binary)
        .stdin(Stdio::null())
        .status()
        .map_err(|error| format!("failed to run file: {error}"))?;
    if !file_status.success() {
        return Err(format!(
            "file {} failed with exit {}",
            binary.display(),
            file_status.code().unwrap_or(1)
        ));
    }

    let readelf = command_output("readelf", &["-l", binary_text])?;
    if readelf.contains("Requesting program interpreter") {
        eprintln!("::error ::musl CLI binary has a dynamic interpreter");
        print!("{readelf}");
        return Err("musl CLI binary verification failed".to_string());
    }

    let strings = command_output("strings", &[binary_text])?;
    let glibc = glibc_requirement_lines(&strings);
    if !glibc.is_empty() {
        eprintln!("::error ::musl CLI binary contains glibc version requirements");
        for line in glibc {
            println!("{line}");
        }
        return Err("musl CLI binary verification failed".to_string());
    }

    Ok(())
}

fn command_output(command: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run {command}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{command} {} failed with exit {}\n{}{}",
            args.join(" "),
            output.status.code().unwrap_or(1),
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        )
        .trim()
        .to_string());
    }
    Ok(format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn glibc_requirement_lines(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter(|line| {
            line.match_indices("GLIBC_").any(|(index, _)| {
                line.as_bytes()
                    .get(index + "GLIBC_".len())
                    .is_some_and(u8::is_ascii_digit)
            })
        })
        .map(ToOwned::to_owned)
        .collect()
}

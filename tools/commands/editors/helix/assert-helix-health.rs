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
    env,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

#[path = "../../../rust/common.rs"]
mod common;

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let root = common::repo_root()?;
    let helix_binary = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("hx"));
    let server_path = env::var_os("VIZE_SERVER_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/ci/vize"))
        .canonicalize()
        .map_err(|error| format!("Vize server does not exist: {error}"))?;
    let config_home = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "XDG_CONFIG_HOME must point at the isolated Helix config".to_string())?;
    let runtime = env::var_os("HELIX_RUNTIME")
        .map(PathBuf::from)
        .ok_or_else(|| "HELIX_RUNTIME must point at the pinned Helix runtime".to_string())?;
    if !helix_binary.exists() && helix_binary.components().count() > 1 {
        return Err(format!(
            "Helix binary does not exist: {}",
            helix_binary.display()
        ));
    }
    if !runtime.exists() {
        return Err(format!(
            "Helix runtime does not exist: {}",
            runtime.display()
        ));
    }
    let installed = config_home.join("helix/languages.toml");
    let packaged = root.join("editors/helix/languages.toml");
    if common::read_text(&installed)? != common::read_text(&packaged)? {
        return Err("Helix must inspect the exact packaged languages.toml".to_string());
    }
    let expected_server = format!("✓ vize: {}", server_path.display());
    for language in ["vue", "art-vue"] {
        let output = Command::new(&helix_binary)
            .args(["--health", language])
            .env(
                "PATH",
                prepend_path(server_path.parent().unwrap_or(Path::new(".")))?,
            )
            .output()
            .map_err(|error| format!("failed to run hx --health {language}: {error}"))?;
        let text = strip_ansi(&format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
        if !output.status.success() {
            return Err(format!("hx --health {language} failed:\n{text}"));
        }
        let configured = lines_between(
            &text,
            "Configured language servers:",
            "Configured debug adapter:",
        )?;
        if configured != vec![expected_server.clone()] {
            return Err(format!(
                "hx --health {language} did not resolve exactly the packaged Vize server: {configured:?}"
            ));
        }
    }
    println!("helix package health passed for vue and art-vue");
    Ok(())
}

fn prepend_path(dir: &Path) -> Result<String, String> {
    let existing = env::var_os("PATH").unwrap_or_default();
    let mut paths = env::split_paths(&existing).collect::<Vec<_>>();
    paths.insert(0, dir.to_path_buf());
    env::join_paths(paths)
        .map_err(|error| format!("cannot build PATH: {error}"))
        .map(|value| value.to_string_lossy().to_string())
}

fn strip_ansi(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for escaped in chars.by_ref() {
                if ('@'..='~').contains(&escaped) {
                    break;
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn lines_between(value: &str, start: &str, end: &str) -> Result<Vec<String>, String> {
    let lines = value
        .lines()
        .map(str::trim)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let start_at = lines
        .iter()
        .position(|line| line == start)
        .ok_or_else(|| format!("missing {start:?} in Helix health output"))?;
    let end_at = lines
        .iter()
        .position(|line| line.starts_with(end))
        .ok_or_else(|| format!("missing {end:?} in Helix health output"))?;
    if end_at <= start_at {
        return Err(format!(
            "missing {end:?} after {start:?} in Helix health output"
        ));
    }
    Ok(lines[start_at + 1..end_at]
        .iter()
        .filter(|line| !line.is_empty())
        .cloned()
        .collect())
}

#[cfg(test)]
mod tests {
    use super::strip_ansi;

    #[test]
    fn strip_ansi_preserves_utf8_symbols() {
        assert_eq!(
            strip_ansi("\x1b[32m✓\x1b[0m vize: /tmp/vize"),
            "✓ vize: /tmp/vize"
        );
    }
}

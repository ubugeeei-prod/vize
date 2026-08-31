#![allow(dead_code)]

use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[path = "./common.rs"]
mod common;

pub fn workspace_version(repo_root: &Path) -> Result<String, String> {
    let cargo = common::read_text(repo_root.join("Cargo.toml"))?;
    cargo
        .lines()
        .find_map(|line| {
            line.strip_prefix("version = \"")
                .and_then(|rest| rest.strip_suffix('"'))
                .map(ToString::to_string)
        })
        .ok_or_else(|| "workspace version is missing from Cargo.toml".to_string())
}

pub fn default_archive(repo_root: &Path, basename: &str, arg: Option<String>) -> PathBuf {
    arg.map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join(basename))
}

pub fn file_size(path: &Path) -> Result<u64, String> {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .map_err(|error| format!("cannot stat {}: {error}", path.display()))
}

pub fn assert_size(path: &Path, label: &str, min: u64, max: u64) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("{label} does not exist: {}", path.display()));
    }
    let size = file_size(path)?;
    if size <= min {
        return Err(format!("{label} is suspiciously small: {size} bytes"));
    }
    if size >= max {
        return Err(format!("{label} is unexpectedly large: {size} bytes"));
    }
    Ok(())
}

pub fn list_zip(path: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("unzip")
        .args(["-Z1"])
        .arg(path)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run unzip: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "unzip -Z1 failed for {}:\n{}{}",
            path.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(lines(String::from_utf8_lossy(&output.stdout).as_ref()))
}

pub fn read_zip_text(path: &Path, name: &str) -> Result<String, String> {
    let output = Command::new("unzip")
        .arg("-p")
        .arg(path)
        .arg(unzip_member_pattern(name))
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run unzip: {error}"))?;
    if !output.status.success() {
        return Err(format!("missing zip entry: {name}"));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| format!("zip entry {name} is not utf8: {error}"))
}

fn unzip_member_pattern(name: &str) -> String {
    let mut pattern = String::new();
    for character in name.chars() {
        if matches!(character, '[' | ']' | '*' | '?' | '\\') {
            pattern.push('\\');
        }
        pattern.push(character);
    }
    pattern
}

pub fn read_zip_json(path: &Path, name: &str) -> Result<Value, String> {
    serde_json::from_str(&read_zip_text(path, name)?)
        .map_err(|error| format!("zip entry {name} is not JSON: {error}"))
}

pub fn list_tar_gz(path: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("tar")
        .arg("-tzf")
        .arg(path)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run tar: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "tar -tzf failed for {}:\n{}{}",
            path.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(lines(String::from_utf8_lossy(&output.stdout).as_ref()))
}

pub fn read_tar_text(path: &Path, name: &str) -> Result<String, String> {
    let output = Command::new("tar")
        .arg("-xOzf")
        .arg(path)
        .arg(name)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run tar: {error}"))?;
    if !output.status.success() {
        return Err(format!("missing tar entry: {name}"));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| format!("tar entry {name} is not utf8: {error}"))
}

pub fn assert_unique(entries: &[String], label: &str) -> Result<(), String> {
    let unique = entries.iter().collect::<BTreeSet<_>>();
    if unique.len() != entries.len() {
        return Err(format!("{label} has duplicates"));
    }
    Ok(())
}

pub fn assert_safe_entries(entries: &[String], label: &str, root: &str) -> Result<(), String> {
    let root_entry = format!("{root}/");
    for name in entries {
        if name.contains('\\') {
            return Err(format!("{label} entry must use POSIX separators: {name}"));
        }
        if name.contains('\0') {
            return Err(format!("{label} entry contains a NUL byte: {name}"));
        }
        if name.starts_with('/') {
            return Err(format!("{label} entry must be relative: {name}"));
        }
        if name.split('/').any(|part| part == "..") {
            return Err(format!("{label} entry must not traverse: {name}"));
        }
        if !(name == &root_entry || name.starts_with(&root_entry)) {
            return Err(format!("unexpected root: {name}"));
        }
    }
    Ok(())
}

pub fn require_entries<F>(
    archive: &Path,
    entries: &[String],
    required: &[&str],
    read_text: F,
    label: &str,
) -> Result<(), String>
where
    F: Fn(&Path, &str) -> Result<String, String>,
{
    let set = entries.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for name in required {
        if !set.contains(name) {
            return Err(format!("{label} is missing required file: {name}"));
        }
        if read_text(archive, name)?.trim().is_empty() {
            return Err(format!("{label} file is empty: {name}"));
        }
    }
    Ok(())
}

pub fn assert_allowed(entries: &[String], label: &str, allowed: &[&str]) -> Result<(), String> {
    for name in entries {
        if !allowed.iter().any(|pattern| glob_like_match(pattern, name)) {
            return Err(format!("{label} ships an unexpected file: {name}"));
        }
    }
    Ok(())
}

pub fn assert_forbidden(entries: &[String], label: &str, forbidden: &[&str]) -> Result<(), String> {
    for name in entries {
        for pattern in forbidden {
            if glob_like_match(pattern, name)
                || (pattern.starts_with('.') && name.ends_with(pattern))
            {
                return Err(format!("{label} must not ship {name}"));
            }
        }
    }
    Ok(())
}

pub fn expect_contains(source: &str, needle: &str, message: &str) -> Result<(), String> {
    if !source.contains(needle) {
        return Err(message.to_string());
    }
    Ok(())
}

pub fn expect_json_string(value: &Value, path: &[&str], expected: &str) -> Result<(), String> {
    let actual = path
        .iter()
        .try_fold(value, |node, key| node.get(*key))
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing JSON string {}", path.join(".")))?;
    if actual != expected {
        return Err(format!(
            "{} expected {expected:?}, got {actual:?}",
            path.join(".")
        ));
    }
    Ok(())
}

fn lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn glob_like_match(pattern: &str, value: &str) -> bool {
    if pattern == value {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix("/**") {
        return value.starts_with(prefix);
    }
    if let Some(prefix) = pattern.strip_suffix("/*") {
        return value.starts_with(prefix)
            && !value[prefix.len()..].trim_start_matches('/').contains('/');
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return value.starts_with(prefix);
    }
    false
}

#![allow(dead_code)]

use serde::Serialize;
use serde_json::Value;
use std::{
    env,
    ffi::{OsStr, OsString},
    fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

pub struct CommandOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn main_result(result: Result<(), String>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

pub fn repo_root() -> Result<PathBuf, String> {
    if let Some(root) = env::var_os("VIZE_REPO_ROOT") {
        let root = PathBuf::from(root);
        if is_repo_root(&root) {
            return canonicalize_root(&root);
        }
        return Err(format!(
            "VIZE_REPO_ROOT={} is not a Vize repository root",
            root.display()
        ));
    }

    for candidate in script_context_roots() {
        if let Some(root) = candidate
            .ancestors()
            .find(|candidate| is_repo_root(candidate))
            .map(Path::to_path_buf)
        {
            return canonicalize_root(&root);
        }
    }

    let current =
        env::current_dir().map_err(|error| format!("cannot read current dir: {error}"))?;
    current
        .ancestors()
        .find(|candidate| is_repo_root(candidate))
        .map(Path::to_path_buf)
        .map(|root| canonicalize_root(&root))
        .transpose()?
        .ok_or_else(|| {
            format!(
                "cannot find Vize repository root from {}; run inside the repository or set VIZE_REPO_ROOT",
                current.display()
            )
        })
}

fn canonicalize_root(root: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(root).map_err(|error| {
        format!(
            "cannot canonicalize repository root {}: {error}",
            root.display()
        )
    })
}

fn script_context_roots() -> Vec<PathBuf> {
    ["RUST_SCRIPT_PATH", "RUST_SCRIPT_BASE_PATH"]
        .into_iter()
        .filter_map(env::var_os)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|path| {
            if path.is_file() {
                path.parent().unwrap_or(&path).to_path_buf()
            } else {
                path
            }
        })
        .collect()
}

fn is_repo_root(dir: &Path) -> bool {
    dir.join("Cargo.toml").is_file() && dir.join("pnpm-workspace.yaml").is_file()
}

pub fn read_text(path: impl AsRef<Path>) -> Result<String, String> {
    let path = path.as_ref();
    fs::read_to_string(path).map_err(|error| format!("cannot read {}: {error}", path.display()))
}

pub fn read_optional_text(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

pub fn write_text(path: impl AsRef<Path>, value: &str) -> Result<(), String> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    fs::write(path, value).map_err(|error| format!("cannot write {}: {error}", path.display()))
}

pub fn append_text(path: impl AsRef<Path>, value: &str) -> Result<(), String> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("cannot open {}: {error}", path.display()))?;
    file.write_all(value.as_bytes())
        .map_err(|error| format!("cannot append {}: {error}", path.display()))
}

pub fn read_json(path: impl AsRef<Path>) -> Result<Value, String> {
    let path = path.as_ref();
    serde_json::from_str(&read_text(path)?)
        .map_err(|error| format!("cannot parse JSON {}: {error}", path.display()))
}

pub fn write_json_pretty(path: impl AsRef<Path>, value: &impl Serialize) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    write_text(path, &format!("{json}\n"))
}

pub fn write_json_compact(path: impl AsRef<Path>, value: &impl Serialize) -> Result<(), String> {
    let json = serde_json::to_string(value).map_err(|error| error.to_string())?;
    write_text(path, &format!("{json}\n"))
}

pub fn mkdir(path: impl AsRef<Path>) -> Result<(), String> {
    let path = path.as_ref();
    fs::create_dir_all(path).map_err(|error| format!("cannot create {}: {error}", path.display()))
}

pub fn run_capture(command: &str, args: &[impl AsRef<OsStr>]) -> Result<CommandOutput, String> {
    run_capture_in(
        command,
        args,
        env::current_dir().map_err(|error| error.to_string())?,
    )
}

pub fn run_capture_in(
    command: &str,
    args: &[impl AsRef<OsStr>],
    cwd: impl AsRef<Path>,
) -> Result<CommandOutput, String> {
    let output = Command::new(command)
        .args(args.iter().map(AsRef::as_ref))
        .current_dir(cwd.as_ref())
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run {}: {error}", command_line(command, args)))?;
    let result = CommandOutput {
        status: output.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    };
    if result.status != 0 {
        let detail = format!("{}{}", result.stdout, result.stderr);
        return Err(format!(
            "{} failed with exit {}{}",
            command_line(command, args),
            result.status,
            if detail.trim().is_empty() {
                String::new()
            } else {
                format!("\n{}", detail.trim())
            }
        )
        .trim()
        .to_string());
    }
    Ok(result)
}

pub fn run_status(command: &str, args: &[impl AsRef<OsStr>]) -> Result<i32, String> {
    let status = Command::new(command)
        .args(args.iter().map(AsRef::as_ref))
        .status()
        .map_err(|error| format!("failed to run {}: {error}", command_line(command, args)))?;
    Ok(status.code().unwrap_or(1))
}

pub fn command_line(command: &str, args: &[impl AsRef<OsStr>]) -> String {
    let mut parts = vec![command.to_string()];
    parts.extend(
        args.iter()
            .map(|arg| shell_quote(&arg.as_ref().to_string_lossy())),
    );
    parts.join(" ")
}

pub fn shell_quote(value: &str) -> String {
    if !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._/:@%+=,-".contains(&byte))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

pub fn append_github_outputs(outputs: &[(&str, String)]) -> Result<(), String> {
    let path =
        env::var_os("GITHUB_OUTPUT").ok_or_else(|| "GITHUB_OUTPUT is required".to_string())?;
    let mut text = String::new();
    for (name, value) in outputs {
        text.push_str(name);
        text.push('=');
        text.push_str(value);
        text.push('\n');
    }
    append_text(path, &text)
}

pub fn append_github_multiline_output(name: &str, value: &str) -> Result<(), String> {
    let path =
        env::var_os("GITHUB_OUTPUT").ok_or_else(|| "GITHUB_OUTPUT is required".to_string())?;
    append_text(path, &format!("{name}<<JSON\n{value}\nJSON\n"))
}

pub fn args_os() -> Vec<OsString> {
    env::args_os().skip(1).collect()
}

pub fn normalize_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn relative_path(root: &Path, path: &Path) -> String {
    normalize_path(path.strip_prefix(root).unwrap_or(path))
}

pub fn visit_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_files(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum Runtime {
    Bash,
    Node,
}

impl Runtime {
    fn executable(self) -> &'static str {
        match self {
            Self::Bash => "bash",
            Self::Node => "node",
        }
    }
}

pub fn run(runtime: Runtime, legacy_tool: &str) -> ExitCode {
    run_with_args(runtime, legacy_tool, env::args_os().skip(1))
}

fn run_with_args<I, S>(runtime: Runtime, legacy_tool: &str, args: I) -> ExitCode
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    match run_inner(runtime, legacy_tool, args) {
        Ok(code) => ExitCode::from(code),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

fn run_inner<I, S>(runtime: Runtime, legacy_tool: &str, args: I) -> Result<u8, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let root = repo_root()?;
    let tool_path = validate_legacy_tool(&root, legacy_tool)?;
    let status = Command::new(runtime.executable())
        .arg(tool_path)
        .args(args)
        .status()
        .map_err(|error| {
            format!(
                "failed to run {} for {legacy_tool}: {error}",
                runtime.executable()
            )
        })?;

    Ok(status.code().unwrap_or(1).clamp(0, 255) as u8)
}

fn repo_root() -> Result<PathBuf, String> {
    if let Some(root) = env::var_os("VIZE_REPO_ROOT") {
        let root = PathBuf::from(root);
        if is_repo_root(&root) {
            return Ok(root);
        }
        return Err(format!(
            "VIZE_REPO_ROOT={} is not a Vize repository root",
            root.display()
        ));
    }

    let current =
        env::current_dir().map_err(|error| format!("cannot read current dir: {error}"))?;
    for dir in current.ancestors() {
        if is_repo_root(dir) {
            return Ok(dir.to_path_buf());
        }
    }

    Err(format!(
        "cannot find Vize repository root from {}; run the command inside the repository or set VIZE_REPO_ROOT",
        current.display()
    ))
}

fn is_repo_root(dir: &Path) -> bool {
    dir.join("Cargo.toml").is_file() && dir.join("pnpm-workspace.yaml").is_file()
}

fn validate_legacy_tool(root: &Path, legacy_tool: &str) -> Result<PathBuf, String> {
    let relative = Path::new(legacy_tool);
    if relative.is_absolute() {
        return Err(format!(
            "legacy tool path must be repository-relative: {legacy_tool}"
        ));
    }
    if !relative.starts_with("tools") {
        return Err(format!(
            "legacy tool path must live under tools/: {legacy_tool}"
        ));
    }
    if relative
        .components()
        .any(|component| component.as_os_str() == "..")
    {
        return Err(format!(
            "legacy tool path cannot contain '..': {legacy_tool}"
        ));
    }
    if legacy_tool.contains("tools/moon/.mooncakes/") {
        return Err(format!(
            "legacy tool path cannot target vendored MoonBit cache: {legacy_tool}"
        ));
    }

    let tool_path = root.join(relative);
    if !tool_path.is_file() {
        return Err(format!(
            "legacy tool does not exist: {}",
            tool_path.display()
        ));
    }
    Ok(tool_path)
}

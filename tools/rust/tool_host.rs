use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Runtime {
    Node,
}

impl Runtime {
    fn executable(self) -> &'static str {
        match self {
            Self::Node => "node",
        }
    }
}

pub fn run(runtime: Runtime, module: &str) -> ExitCode {
    run_with_args(runtime, module, env::args_os().skip(1))
}

fn run_with_args<I, S>(runtime: Runtime, module: &str, args: I) -> ExitCode
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    match run_inner(runtime, module, args) {
        Ok(code) => ExitCode::from(code),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

fn run_inner<I, S>(runtime: Runtime, module: &str, args: I) -> Result<u8, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let root = repo_root()?;
    let module_path = validate_tool_module(&root, module)?;
    let status = Command::new(runtime.executable())
        .arg(module_path)
        .args(args)
        .status()
        .map_err(|error| {
            format!(
                "failed to run {} for {module}: {error}",
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
    if let Some(root) = repo_root_from(&current) {
        return Ok(root);
    }
    if let Some(root) = rust_script_source_repo_root()? {
        return Ok(root);
    }

    Err(format!(
        "cannot find Vize repository root from {}; run the command inside the repository or set VIZE_REPO_ROOT",
        current.display()
    ))
}

fn is_repo_root(dir: &Path) -> bool {
    dir.join("Cargo.toml").is_file() && dir.join("pnpm-workspace.yaml").is_file()
}

fn repo_root_from(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|candidate| is_repo_root(candidate))
        .map(Path::to_path_buf)
}

fn rust_script_source_repo_root() -> Result<Option<PathBuf>, String> {
    let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") else {
        return Ok(None);
    };
    let manifest_path = Path::new(manifest_dir).join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path).map_err(|error| {
        format!(
            "cannot read rust-script manifest {}: {error}",
            manifest_path.display()
        )
    })?;
    for line in manifest.lines() {
        let Some(raw_path) = line.trim_start().strip_prefix("path = ") else {
            continue;
        };
        let Some(source_path) = parse_toml_string(raw_path) else {
            continue;
        };
        if let Some(root) = repo_root_from(Path::new(&source_path)) {
            return Ok(Some(root));
        }
    }
    Ok(None)
}

fn parse_toml_string(value: &str) -> Option<String> {
    let mut chars = value.trim_start().chars();
    if chars.next()? != '"' {
        return None;
    }
    let mut output = String::new();
    let mut escaped = false;
    for char in chars {
        if escaped {
            output.push(char);
            escaped = false;
        } else if char == '\\' {
            escaped = true;
        } else if char == '"' {
            return Some(output);
        } else {
            output.push(char);
        }
    }
    None
}

fn validate_tool_module(root: &Path, module: &str) -> Result<PathBuf, String> {
    let relative = Path::new(module);
    if relative.is_absolute() {
        return Err(format!(
            "tool module path must be repository-relative: {module}"
        ));
    }
    if !relative.starts_with("tools") {
        return Err(format!("tool module path must live under tools/: {module}"));
    }
    if relative
        .components()
        .any(|component| component.as_os_str() == "..")
    {
        return Err(format!("tool module path cannot contain '..': {module}"));
    }
    if module.contains("tools/moon/.mooncakes/") {
        return Err(format!(
            "tool module path cannot target vendored MoonBit cache: {module}"
        ));
    }

    let module_path = root.join(relative);
    if !module_path.is_file() {
        return Err(format!(
            "tool module does not exist: {}",
            module_path.display()
        ));
    }
    Ok(module_path)
}

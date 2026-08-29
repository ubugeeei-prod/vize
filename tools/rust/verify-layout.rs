#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

const EXPLICIT_ENTRYPOINTS: &[&str] = &[
    "tools/fixtures/glyph-corpus-waiver-audit.mjs",
    "tools/fixtures/patina-rule-map.mjs",
    "tools/fixtures/real-project-surface-verdict.mjs",
    "tools/github/release-platforms.mjs",
    "tools/github/require-needs-success.mjs",
    "tools/github/semver-change-marker.mjs",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Runtime {
    Bash,
    Node,
}

impl Runtime {
    fn wrapper_token(self) -> &'static str {
        match self {
            Self::Bash => "Runtime::Bash",
            Self::Node => "Runtime::Node",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Entrypoint {
    command: String,
    legacy: String,
    runtime: Runtime,
}

fn main() -> ExitCode {
    match verify() {
        Ok(count) => {
            println!("rust-script tools: verified {count} command wrappers");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for error in errors {
                eprintln!("rust-script tools: {error}");
            }
            ExitCode::from(1)
        }
    }
}

fn verify() -> Result<usize, Vec<String>> {
    let root = repo_root().map_err(|error| vec![error])?;
    let tools = root.join("tools");
    let entrypoints =
        collect_entrypoints(&root, &tools).map_err(|error| vec![error.to_string()])?;
    let mut errors = Vec::new();
    let mut expected_commands = BTreeSet::new();

    for entrypoint in &entrypoints {
        expected_commands.insert(entrypoint.command.clone());
        let path = root.join(&entrypoint.command);
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) => {
                errors.push(format!("missing wrapper {}: {error}", entrypoint.command));
                continue;
            }
        };
        if !source.starts_with("#!/usr/bin/env rust-script\n") {
            errors.push(format!(
                "{} must start with a rust-script shebang",
                entrypoint.command
            ));
        }
        if !source.contains("legacy_command::run(") {
            errors.push(format!(
                "{} must call the shared legacy runner",
                entrypoint.command
            ));
        }
        if !source.contains(entrypoint.runtime.wrapper_token()) {
            errors.push(format!(
                "{} must use {}",
                entrypoint.command,
                entrypoint.runtime.wrapper_token()
            ));
        }
        if !source.contains(&format!("\"{}\"", entrypoint.legacy)) {
            errors.push(format!(
                "{} must point at {}",
                entrypoint.command, entrypoint.legacy
            ));
        }
    }

    for command in collect_command_wrappers(&root, &root.join("tools/commands"))
        .map_err(|error| vec![error.to_string()])?
    {
        if !expected_commands.contains(&command) {
            errors.push(format!("{command} has no matching legacy entrypoint"));
        }
    }

    if errors.is_empty() {
        Ok(entrypoints.len())
    } else {
        Err(errors)
    }
}

fn collect_entrypoints(root: &Path, tools: &Path) -> io::Result<Vec<Entrypoint>> {
    let mut files = Vec::new();
    visit_files(tools, &mut files)?;
    let mut entrypoints = Vec::new();

    for file in files {
        let legacy = normalize_relative(root, &file);
        if skip_path(&legacy) {
            continue;
        }
        let Some(runtime) = runtime_for(&file) else {
            continue;
        };
        if !has_shebang(&file)? && !EXPLICIT_ENTRYPOINTS.contains(&legacy.as_str()) {
            continue;
        }
        let command = command_path(&legacy)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        entrypoints.push(Entrypoint {
            command,
            legacy,
            runtime,
        });
    }

    entrypoints.sort();
    Ok(entrypoints)
}

fn collect_command_wrappers(root: &Path, commands: &Path) -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    visit_files(commands, &mut files)?;
    let mut wrappers = files
        .into_iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .map(|path| normalize_relative(root, &path))
        .collect::<Vec<_>>();
    wrappers.sort();
    Ok(wrappers)
}

fn visit_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
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

fn skip_path(path: &str) -> bool {
    path.starts_with("tools/commands/")
        || path.starts_with("tools/rust/")
        || path.starts_with("tools/moon/.mooncakes/")
}

fn runtime_for(path: &Path) -> Option<Runtime> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("js" | "mjs" | "ts") => Some(Runtime::Node),
        Some("sh") => Some(Runtime::Bash),
        _ => None,
    }
}

fn has_shebang(path: &Path) -> io::Result<bool> {
    let source = fs::read_to_string(path)?;
    Ok(source.starts_with("#!"))
}

fn command_path(legacy: &str) -> Result<String, String> {
    let legacy = legacy.strip_prefix("tools/").expect("tool path");
    let mut parts = legacy.split('/').collect::<Vec<_>>();
    let file = parts.pop().expect("file name");
    let stem = file
        .strip_suffix(".mjs")
        .or_else(|| file.strip_suffix(".js"))
        .or_else(|| file.strip_suffix(".ts"))
        .or_else(|| file.strip_suffix(".sh"))
        .unwrap_or(file);

    let mut buckets = BTreeMap::from([
        ("ai-fix-agent.mjs", vec!["agents"]),
        ("davinci", vec!["davinci"]),
        ("editor-e2e", vec!["editors", "e2e"]),
        ("emacs-vize", vec!["editors", "emacs"]),
        ("fixtures", vec!["fixtures"]),
        ("fuzz", vec!["ci", "fuzz"]),
        ("github", vec!["ci", "github"]),
        ("helix-vize", vec!["editors", "helix"]),
        ("npm", vec!["release", "npm"]),
        ("nvim-vize", vec!["editors", "neovim"]),
        ("release", vec!["release"]),
        ("vim-vize", vec!["editors", "vim"]),
        ("vscode-vize", vec!["editors", "vscode"]),
        ("zed-vize", vec!["editors", "zed"]),
    ]);
    let first = parts.first().copied().unwrap_or(file);
    let mut command_parts = buckets
        .remove(first)
        .ok_or_else(|| format!("missing canonical command bucket for tools/{legacy}"))?;
    command_parts.push(stem);
    Ok(format!("tools/commands/{}.rs", command_parts.join("/")))
}

fn repo_root() -> Result<PathBuf, String> {
    let current =
        env::current_dir().map_err(|error| format!("cannot read current dir: {error}"))?;
    for dir in current.ancestors() {
        if dir.join("Cargo.toml").is_file() && dir.join("pnpm-workspace.yaml").is_file() {
            return Ok(dir.to_path_buf());
        }
    }
    Err(format!(
        "cannot find repository root from {}",
        current.display()
    ))
}

fn normalize(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn normalize_relative(root: &Path, path: &Path) -> String {
    normalize(path.strip_prefix(root).unwrap_or(path))
}

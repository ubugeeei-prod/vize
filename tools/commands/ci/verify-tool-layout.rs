#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

const LEGACY_COMMAND_MARKER: &str = concat!("legacy", "_command");
const TOOL_HOST_MARKER: &str = "tool_host";

fn main() -> ExitCode {
    match verify() {
        Ok(count) => {
            println!("rust-script tools: verified {count} Rust Script commands");
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
    let commands = collect_command_scripts(&root).map_err(|error| vec![error.to_string()])?;
    let mut errors = Vec::new();

    if root.join("tools/rust").exists() {
        errors.push("tools/rust language bucket must not come back".to_string());
    }
    if root
        .join("tools/support")
        .join(format!("{LEGACY_COMMAND_MARKER}.rs"))
        .exists()
    {
        errors.push("legacy command runner source must not come back".to_string());
    }
    if root
        .join("tools/support")
        .join(format!("{TOOL_HOST_MARKER}.rs"))
        .exists()
    {
        errors.push("tool host runner source must not come back".to_string());
    }

    for command in &commands {
        verify_command(&root, command, &mut errors);
    }

    for script in collect_javascript_tools(&root).map_err(|error| vec![error.to_string()])? {
        errors.push(format!("{script} must be ported to Rust Script"));
    }

    if errors.is_empty() {
        Ok(commands.len())
    } else {
        Err(errors)
    }
}

fn verify_command(root: &Path, command: &str, errors: &mut Vec<String>) {
    let path = root.join(command);
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            errors.push(format!("cannot read {command}: {error}"));
            return;
        }
    };
    if !source.starts_with("#!/usr/bin/env rust-script\n") {
        errors.push(format!("{command} must start with a rust-script shebang"));
    }
    if source.contains(LEGACY_COMMAND_MARKER) {
        errors.push(format!(
            "{command} must not reference the legacy command runner"
        ));
    }
    if command.contains("-vize/") {
        errors.push(format!("{command} must use product-neutral editor buckets"));
    }
    if source.contains(&format!("{TOOL_HOST_MARKER}::run(")) {
        errors.push(format!("{command} must not proxy to Node tooling"));
    }
    if source.contains(&format!("{TOOL_HOST_MARKER}::Runtime::Node")) {
        errors.push(format!("{command} must not mention the Node runtime"));
    }
}

fn collect_command_scripts(root: &Path) -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    visit_files(&root.join("tools/commands"), &mut files)?;
    let mut commands = files
        .into_iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .map(|path| normalize_relative(root, &path))
        .collect::<Vec<_>>();
    commands.sort();
    Ok(commands)
}

fn collect_javascript_tools(root: &Path) -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    visit_files(&root.join("tools"), &mut files)?;
    let mut scripts = Vec::new();
    for path in files {
        let relative = normalize_relative(root, &path);
        if skip_path(&relative) || !is_javascript_tool(&path) {
            continue;
        }
        scripts.push(relative);
    }
    scripts.sort();
    Ok(scripts)
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
        || path.starts_with("tools/benchmarks/scripts/")
        || path.starts_with("tools/config/vite-plus/")
        || path.starts_with("tools/support/")
        || path.starts_with("tools/moon/.mooncakes/")
}

fn is_javascript_tool(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("js" | "mjs" | "ts")
    )
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

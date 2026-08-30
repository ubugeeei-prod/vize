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
    let host_hash = tool_host_hash(&root).map_err(|error| vec![error.to_string()])?;
    let mut errors = Vec::new();

    if root.join("tools/rust/legacy_command.rs").exists() {
        errors.push("tools/rust/legacy_command.rs must not come back".to_string());
    }

    for command in &commands {
        verify_command(&root, command, &host_hash, &mut errors);
    }

    for script in collect_legacy_executables(&root).map_err(|error| vec![error.to_string()])? {
        errors.push(format!(
            "{script} is executable legacy tooling; move the command to tools/commands"
        ));
    }

    if errors.is_empty() {
        Ok(commands.len())
    } else {
        Err(errors)
    }
}

fn verify_command(root: &Path, command: &str, host_hash: &str, errors: &mut Vec<String>) {
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
    if source.contains("legacy_command") {
        errors.push(format!(
            "{command} must not reference the legacy command runner"
        ));
    }
    if command.contains("-vize/") {
        errors.push(format!("{command} must use product-neutral editor buckets"));
    }
    if source.contains("tool_host::run(") {
        let required_salt = format!("tool-host: {host_hash}");
        if !source.contains(&required_salt) {
            errors.push(format!(
                "{command} must include {required_salt} so rust-script cache invalidates with the shared host"
            ));
        }
        verify_hosted_module(root, command, &source, errors);
    }
}

fn verify_hosted_module(root: &Path, command: &str, source: &str, errors: &mut Vec<String>) {
    if !source.contains("tool_host::Runtime::Node") {
        errors.push(format!(
            "{command} compatibility modules must run through Node"
        ));
    }
    let Some(module) = first_tool_string(source) else {
        errors.push(format!(
            "{command} must name the compatibility module it hosts"
        ));
        return;
    };
    if !(module.ends_with(".mjs") || module.ends_with(".js") || module.ends_with(".ts")) {
        errors.push(format!("{command} hosts unsupported module {module}"));
    }
    if module.contains("tools/moon/.mooncakes/") || module.contains("..") {
        errors.push(format!("{command} hosts invalid module path {module}"));
        return;
    }
    let module_path = root.join(module);
    let module_source = match fs::read_to_string(&module_path) {
        Ok(source) => source,
        Err(error) => {
            errors.push(format!("{command} hosts missing module {module}: {error}"));
            return;
        }
    };
    if module_source.starts_with("#!") {
        errors.push(format!(
            "{module} must not remain an executable command entrypoint"
        ));
    }
}

fn first_tool_string(source: &str) -> Option<&str> {
    source
        .split('"')
        .skip(1)
        .step_by(2)
        .find(|value| value.starts_with("tools/"))
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

fn collect_legacy_executables(root: &Path) -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    visit_files(&root.join("tools"), &mut files)?;
    let mut scripts = Vec::new();
    for path in files {
        let relative = normalize_relative(root, &path);
        if skip_path(&relative) || !is_legacy_script(&path) {
            continue;
        }
        if is_command_entrypoint(&path)? {
            scripts.push(relative);
        }
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
        || path.starts_with("tools/rust/")
        || path.starts_with("tools/moon/.mooncakes/")
}

fn is_legacy_script(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("js" | "mjs" | "ts" | "sh")
    )
}

fn is_command_entrypoint(path: &Path) -> io::Result<bool> {
    Ok(fs::read_to_string(path)?.starts_with("#!") || is_executable(path)?)
}

fn tool_host_hash(root: &Path) -> io::Result<String> {
    let bytes = fs::read(root.join("tools/rust/tool_host.rs"))?;
    Ok(format!("{:016x}", fnv1a64(&bytes)))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(unix)]
fn is_executable(path: &Path) -> io::Result<bool> {
    use std::os::unix::fs::PermissionsExt;

    Ok(fs::metadata(path)?.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(_path: &Path) -> io::Result<bool> {
    Ok(false)
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

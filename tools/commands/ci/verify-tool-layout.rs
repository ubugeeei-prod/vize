#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! [dependencies]
//! syn = { version = "=2.0.117", features = ["full"] }
//! serde_json = "1"
//! ```

use std::{
    collections::BTreeSet,
    env, fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};
use syn::{Item, ext::IdentExt};

const LEGACY_COMMAND_MARKER: &str = concat!("legacy", "_command");
const TOOL_HOST_MARKER: &str = "tool_host";

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|arg| arg == "--module-closure") {
        return print_module_closure(&args[1..]);
    }
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

fn print_module_closure(paths: &[String]) -> ExitCode {
    let root = match repo_root() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let mut modules = BTreeSet::new();
    let mut errors = Vec::new();
    for path in paths {
        if Path::new(path).is_absolute()
            || Path::new(path)
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            errors.push(format!("expected repository-relative module root: {path}"));
            continue;
        }
        visit_module(&root.join(path), true, &mut modules, &mut errors);
    }
    if !errors.is_empty() {
        for error in errors {
            eprintln!("{error}");
        }
        return ExitCode::FAILURE;
    }
    let mut paths = modules
        .iter()
        .map(|path| normalize_relative(&root, path))
        .collect::<Vec<_>>();
    paths.sort();
    match serde_json::to_string(&paths) {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
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

    let mut modules = BTreeSet::new();
    let mut entrypoints = 0;
    for command in &commands {
        let path = root.join(command);
        if fs::read_to_string(&path)
            .is_ok_and(|source| source.starts_with("#!/usr/bin/env rust-script\n"))
        {
            entrypoints += 1;
            visit_module(&path, true, &mut modules, &mut errors);
        }
    }
    for command in &commands {
        verify_command(&root, command, &modules, &mut errors);
    }

    for script in collect_javascript_tools(&root).map_err(|error| vec![error.to_string()])? {
        errors.push(format!("{script} must be ported to Rust Script"));
    }

    if errors.is_empty() {
        Ok(entrypoints)
    } else {
        Err(errors)
    }
}

fn verify_command(
    root: &Path,
    command: &str,
    modules: &BTreeSet<PathBuf>,
    errors: &mut Vec<String>,
) {
    let path = root.join(command);
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            errors.push(format!("cannot read {command}: {error}"));
            return;
        }
    };
    if !source.starts_with("#!/usr/bin/env rust-script\n") && !modules.contains(&path) {
        errors.push(format!(
            "{command} must start with a rust-script shebang or be a reachable ordinary module"
        ));
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

fn visit_module(
    path: &Path,
    root: bool,
    modules: &mut BTreeSet<PathBuf>,
    errors: &mut Vec<String>,
) {
    if !modules.insert(path.to_path_buf()) {
        return;
    }
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            errors.push(format!("cannot read {}: {error}", path.display()));
            return;
        }
    };
    let parsed = match syn::parse_file(&source) {
        Ok(parsed) => parsed,
        Err(error) => {
            errors.push(format!("cannot parse {}: {error}", path.display()));
            return;
        }
    };
    let base = if root || path.file_name().is_some_and(|name| name == "mod.rs") {
        path.parent().unwrap_or(path).to_path_buf()
    } else {
        path.with_extension("")
    };
    visit_items(&parsed.items, &base, modules, errors);
}

fn visit_items(
    items: &[Item],
    base: &Path,
    modules: &mut BTreeSet<PathBuf>,
    errors: &mut Vec<String>,
) {
    for item in items {
        let Item::Mod(module) = item else {
            continue;
        };
        let name = module.ident.unraw().to_string();
        if module
            .attrs
            .iter()
            .any(|attribute| attribute.path().is_ident("path"))
        {
            errors.push(format!(
                "{}: {name} must use ordinary module discovery",
                base.display()
            ));
            continue;
        }
        if let Some((_, items)) = &module.content {
            visit_items(items, &base.join(&name), modules, errors);
            continue;
        }
        let flat = base.join(format!("{name}.rs"));
        let directory = base.join(&name).join("mod.rs");
        match (flat.is_file(), directory.is_file()) {
            (true, false) => visit_module(&flat, false, modules, errors),
            (false, true) => visit_module(&directory, false, modules, errors),
            (true, true) => errors.push(format!(
                "ambiguous ordinary module: {} and {}",
                flat.display(),
                directory.display()
            )),
            (false, false) => errors.push(format!(
                "missing ordinary module: {} or {}",
                flat.display(),
                directory.display()
            )),
        }
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

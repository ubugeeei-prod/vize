//! Check command execution logic.
//!
//! The direct runner delegates to `vize_canon`'s project-backed Corsa type
//! checker so Vue SFCs, TypeScript sources, ambient declarations, and emitted
//! `.d.ts` output all share the same virtual project.

#![allow(clippy::disallowed_macros)]

use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use ignore::WalkBuilder;
use vize_canon::{
    batch::TypeChecker as BatchTypeCheckerTrait, BatchTypeChecker, BatchTypeCheckerOptions,
    DeclarationEmitOptions,
};
use vize_carton::{cstr, profiler::global_profiler, FxHashSet, String};

use crate::commands::profile::{
    print_profile_report, ProfilePhase, ProfilePhaseKind, ProfileReport,
};

use super::{
    reporting::{JsonFileResult, JsonOutput},
    tsconfig_inputs::collect_default_check_files,
    CheckArgs,
};

/// Run type checking via Unix socket connection to check-server.
#[cfg(unix)]
pub(crate) fn run_with_socket(args: &CheckArgs, socket_path: &str) {
    use std::{
        io::{BufRead, BufReader, Write},
        os::unix::net::UnixStream,
    };

    use super::{JsonRpcResponse, ServerCheckResult};

    let start = Instant::now();
    let collect_start = Instant::now();
    #[allow(clippy::disallowed_types)]
    let default_patterns = vec![std::string::String::from(".")];
    let files = if args.patterns.is_empty() {
        collect_vue_files(&default_patterns)
    } else {
        collect_vue_files(&args.patterns)
    };
    let collect_time = collect_start.elapsed();

    if files.is_empty() {
        eprintln!("No .vue files found matching inputs: {:?}", args.patterns);
        return;
    }

    let connect_start = Instant::now();
    let mut stream = match UnixStream::connect(socket_path) {
        Ok(stream) => stream,
        Err(error) => {
            eprintln!(
                "\x1b[31mError:\x1b[0m Failed to connect to check-server: {}",
                error
            );
            eprintln!();
            eprintln!("\x1b[33mHint:\x1b[0m Start the server first:");
            eprintln!("  vize check-server --socket {}", socket_path);
            std::process::exit(1);
        }
    };
    let connect_time = connect_start.elapsed();

    if !args.quiet {
        eprintln!("Connected to check-server at {}", socket_path);
        eprintln!("Type checking {} Vue files...", files.len());
    }

    let mut total_errors = 0usize;
    #[allow(clippy::disallowed_types, clippy::disallowed_methods)]
    let mut results: Vec<(std::string::String, ServerCheckResult)> = Vec::new();

    let request_start = Instant::now();
    for path in &files {
        #[allow(clippy::disallowed_types)]
        let source = match fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("Failed to read {}: {}", path.display(), error);
                continue;
            }
        };

        #[allow(clippy::disallowed_methods)]
        let filename = path.to_string_lossy().to_string();

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "check",
            "params": {
                "uri": filename,
                "content": source,
            }
        });

        if writeln!(stream, "{}", request).is_err() || stream.flush().is_err() {
            eprintln!("Failed to send request");
            break;
        }

        let mut reader = BufReader::new(&stream);
        #[allow(clippy::disallowed_types)]
        let mut response_line = std::string::String::new();
        if reader.read_line(&mut response_line).is_err() {
            eprintln!("Failed to read response");
            break;
        }

        let response: JsonRpcResponse = match serde_json::from_str(&response_line) {
            Ok(response) => response,
            Err(error) => {
                eprintln!("Failed to parse response: {}", error);
                continue;
            }
        };

        if let Some(error) = response.error {
            eprintln!("Server error: {}", error.message);
            continue;
        }

        if let Some(result) = response.result {
            total_errors += result.error_count;
            if args.show_virtual_ts {
                eprintln!("\n=== {} ===", filename);
                eprintln!("{}", result.virtual_ts);
            }
            results.push((filename, result));
        }
    }
    let request_time = request_start.elapsed();

    let render_start = Instant::now();
    if !args.quiet {
        for (filename, result) in &results {
            if result.diagnostics.is_empty() {
                continue;
            }
            println!("\n\x1b[4m{}\x1b[0m", filename);
            for diagnostic in &result.diagnostics {
                let color = if diagnostic.severity == "error" {
                    "\x1b[31m"
                } else {
                    "\x1b[33m"
                };
                let code = diagnostic
                    .code
                    .as_ref()
                    .map(|code| cstr!(" [{}]", code))
                    .unwrap_or_default();
                println!(
                    "  {}{}:{}:{}\x1b[0m{} {}",
                    color,
                    diagnostic.severity,
                    diagnostic.line,
                    diagnostic.column,
                    code,
                    diagnostic.message
                );
            }
        }
    }
    let render_time = render_start.elapsed();
    let total_time = start.elapsed();

    let status = if total_errors > 0 {
        "\x1b[31m\u{2717}\x1b[0m"
    } else {
        "\x1b[32m\u{2713}\x1b[0m"
    };
    println!(
        "\n{} Type checked {} Vue files in {:.2?} (via socket)",
        status,
        files.len(),
        total_time
    );
    if args.profile {
        let phases = [
            ProfilePhase {
                name: "collect files",
                duration: collect_time,
                kind: ProfilePhaseKind::Wall,
                note: "Vue input discovery",
            },
            ProfilePhase {
                name: "connect socket",
                duration: connect_time,
                kind: ProfilePhaseKind::Wall,
                note: "Unix socket handshake",
            },
            ProfilePhase {
                name: "request checks",
                duration: request_time,
                kind: ProfilePhaseKind::Wall,
                note: "read, send, receive",
            },
            ProfilePhase {
                name: "render diagnostics",
                duration: render_time,
                kind: ProfilePhaseKind::Wall,
                note: "terminal output",
            },
        ];
        let mut recommendations: Vec<String> = Vec::new();
        if request_time > connect_time * 4 {
            recommendations.push(
                "Socket request time dominates; profile the running check-server process next."
                    .into(),
            );
        }
        let summary = cstr!(
            "{} Vue file(s), {} error(s), socket {}",
            files.len(),
            total_errors,
            socket_path
        );
        let report = ProfileReport {
            title: "check --socket",
            summary: summary.as_str(),
            total: total_time,
            phases: &phases,
            files: &[],
            slow_threshold: Duration::from_millis(0),
            throughput_bytes: None,
            operations: None,
            recommendations: &recommendations,
        };
        print_profile_report(&report);
    }
    if total_errors > 0 {
        println!("  \x1b[31m{} error(s)\x1b[0m", total_errors);
        std::process::exit(1);
    }
    println!("  \x1b[32mNo type errors found!\x1b[0m");
}

/// Run type checking directly with a materialized Corsa project.
pub(crate) fn run_direct(args: &CheckArgs) {
    use super::nuxt;

    let start = Instant::now();
    if args.profile {
        let profiler = global_profiler();
        profiler.clear();
        profiler.enable();
    }

    let config = crate::config::load_config(None);
    crate::config::write_schema(None);

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_root = resolve_project_root(args.tsconfig.as_deref(), &cwd, &[]);
    let tsconfig_path = resolve_tsconfig_path(args.tsconfig.as_deref(), &cwd, &project_root, &[]);
    let collect_start = Instant::now();
    let files = if args.patterns.is_empty() {
        collect_default_check_files(&project_root, tsconfig_path.as_deref())
    } else {
        collect_check_files(&args.patterns)
    };
    let collect_time = collect_start.elapsed();

    if files.is_empty() {
        eprintln!(
            "No Vue or TypeScript files found matching inputs: {:?}",
            args.patterns
        );
        return;
    }

    let project_root = resolve_project_root(args.tsconfig.as_deref(), &cwd, &files);
    let tsconfig_path =
        resolve_tsconfig_path(args.tsconfig.as_deref(), &cwd, &project_root, &files);

    let mut virtual_ts_options = build_virtual_ts_options(&config, &cwd);
    nuxt::detect_nuxt_auto_imports(&mut virtual_ts_options, &cwd);

    if !args.quiet {
        eprintln!(
            "Building Corsa virtual project for {} files under {}...",
            files.len(),
            project_root.display()
        );
    }

    let gen_start = Instant::now();
    let mut checker = match BatchTypeChecker::with_options(
        &project_root,
        BatchTypeCheckerOptions {
            tsconfig_path,
            virtual_ts_options,
        },
    ) {
        Ok(checker) => checker,
        Err(error) => {
            eprintln!("\x1b[31mError:\x1b[0m {}", error);
            std::process::exit(1);
        }
    };

    if let Err(error) = checker.scan_paths(&files) {
        eprintln!("\x1b[31mError:\x1b[0m {}", error);
        std::process::exit(1);
    }
    let gen_time = gen_start.elapsed();

    let virtual_files = checker.virtual_files();
    if virtual_files.is_empty() {
        eprintln!("No files were registered for type checking");
        return;
    }

    if args.show_virtual_ts {
        for file in &virtual_files {
            eprintln!("\n=== {} ===", file.original_path.display());
            eprintln!("{}", file.content);
        }
    }

    let profile_artifact_start = Instant::now();
    if args.profile {
        write_profile_virtual_ts(&virtual_files);
    }
    let profile_artifact_time = profile_artifact_start.elapsed();

    if !args.quiet {
        eprintln!(
            "Running Corsa diagnostics for {} files...",
            virtual_files.len()
        );
    }

    let check_start = Instant::now();
    let result = match checker.check_project() {
        Ok(result) => result,
        Err(error) => {
            eprintln!("\x1b[31mError:\x1b[0m {}", error);
            std::process::exit(1);
        }
    };
    let check_time = check_start.elapsed();

    let emit_start = Instant::now();
    let emitted_declarations = if args.declaration {
        let declaration_dir =
            resolve_declaration_dir(args.declaration_dir.as_deref(), &project_root);
        match checker.emit_declarations(&DeclarationEmitOptions::new(declaration_dir.clone())) {
            Ok(result) => Some((declaration_dir, result)),
            Err(error) => {
                eprintln!("\x1b[31mError:\x1b[0m {}", error);
                std::process::exit(1);
            }
        }
    } else {
        None
    };
    let emit_time = emit_start.elapsed();
    let diagnostics_render_start = Instant::now();
    let diagnostics = render_diagnostics(&result.diagnostics);
    let diagnostics_render_time = diagnostics_render_start.elapsed();
    let total_time = start.elapsed();
    let total_errors = result.error_count();

    if args.profile {
        let profiler = global_profiler();
        let operation_summary = profiler.summary();
        profiler.disable();
        let mut phases = vec![
            ProfilePhase {
                name: "collect inputs",
                duration: collect_time,
                kind: ProfilePhaseKind::Wall,
                note: "tsconfig or explicit patterns",
            },
            ProfilePhase {
                name: "virtual project",
                duration: gen_time,
                kind: ProfilePhaseKind::Wall,
                note: "scan paths and generate Virtual TS",
            },
            ProfilePhase {
                name: "profile artifacts",
                duration: profile_artifact_time,
                kind: ProfilePhaseKind::Wall,
                note: "write node_modules/.vize/check-profile",
            },
            ProfilePhase {
                name: "corsa diagnostics",
                duration: check_time,
                kind: ProfilePhaseKind::Wall,
                note: "project-session diagnostics",
            },
            ProfilePhase {
                name: "render diagnostics",
                duration: diagnostics_render_time,
                kind: ProfilePhaseKind::Wall,
                note: "group diagnostics by file",
            },
        ];
        if args.declaration {
            phases.push(ProfilePhase {
                name: "declaration emit",
                duration: emit_time,
                kind: ProfilePhaseKind::Wall,
                note: "materialized Corsa project",
            });
        }

        let virtual_bytes = virtual_files
            .iter()
            .fold(0usize, |acc, file| acc + file.content.len());
        let mut recommendations: Vec<String> = Vec::new();
        if check_time > gen_time * 2 {
            recommendations.push(
                "Corsa diagnostics dominate; keep the generated Virtual TS directory and inspect the largest generated files."
                    .into(),
            );
        } else if gen_time > check_time {
            recommendations.push(
                "Virtual TS generation dominates; inspect SFCs with large templates, macros, or cross-file imports."
                    .into(),
            );
        }
        if let Some(largest) = virtual_files.iter().max_by_key(|file| file.content.len()) {
            recommendations.push(cstr!(
                "Largest Virtual TS: {} ({} bytes).",
                largest.original_path.display(),
                largest.content.len()
            ));
        }

        let summary = cstr!(
            "{} virtual file(s), {} error(s), project {}",
            virtual_files.len(),
            total_errors,
            project_root.display()
        );
        let report = ProfileReport {
            title: "check",
            summary: summary.as_str(),
            total: total_time,
            phases: &phases,
            files: &[],
            slow_threshold: Duration::from_millis(0),
            throughput_bytes: Some(virtual_bytes),
            operations: Some(&operation_summary),
            recommendations: &recommendations,
        };
        print_profile_report(&report);
    }

    if args.format == "json" {
        let mut files_json: Vec<JsonFileResult> = virtual_files
            .iter()
            .map(|file| {
                let key = file.original_path.to_string_lossy().into_owned();
                JsonFileResult {
                    file: display_path(&cwd, &file.original_path).into(),
                    virtual_ts: file.content.clone().into(),
                    diagnostics: diagnostics.get(key.as_str()).cloned().unwrap_or_default(),
                }
            })
            .collect();
        files_json.sort_by(|left, right| left.file.cmp(&right.file));

        let declarations = emitted_declarations.as_ref().map(|(_, result)| {
            result
                .files
                .iter()
                .map(|file| display_path(&cwd, &file.path).into())
                .collect()
        });

        let json_output = JsonOutput {
            files: files_json,
            error_count: total_errors,
            file_count: virtual_files.len(),
            declarations,
        };
        println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
        if total_errors > 0 {
            std::process::exit(1);
        }
        return;
    }

    if !args.quiet {
        for file in checker.virtual_files() {
            let key = file.original_path.to_string_lossy();
            let Some(file_diagnostics) = diagnostics.get(key.as_ref()) else {
                continue;
            };
            if file_diagnostics.is_empty() {
                continue;
            }

            println!("\n\x1b[4m{}\x1b[0m", key);
            for diagnostic in file_diagnostics {
                let color = if diagnostic.starts_with("error") {
                    "\x1b[31m"
                } else {
                    "\x1b[33m"
                };
                println!("  {}{}\x1b[0m", color, diagnostic);
            }
        }
    }

    let status = if total_errors > 0 {
        "\x1b[31m\u{2717}\x1b[0m"
    } else {
        "\x1b[32m\u{2713}\x1b[0m"
    };
    if emitted_declarations.is_some() {
        println!(
            "\n{} Type checked {} files in {:.2?} (collect: {:.2?}, gen: {:.2?}, corsa: {:.2?}, dts: {:.2?})",
            status,
            virtual_files.len(),
            total_time,
            collect_time,
            gen_time,
            check_time,
            emit_time
        );
    } else {
        println!(
            "\n{} Type checked {} files in {:.2?} (collect: {:.2?}, gen: {:.2?}, corsa: {:.2?})",
            status,
            virtual_files.len(),
            total_time,
            collect_time,
            gen_time,
            check_time
        );
    }

    if total_errors > 0 {
        println!("  \x1b[31m{} error(s)\x1b[0m", total_errors);
    } else {
        println!("  \x1b[32mNo type errors found!\x1b[0m");
    }

    if let Some((declaration_dir, emit_result)) = emitted_declarations {
        println!(
            "  \x1b[32mEmitted {} declaration file(s)\x1b[0m to {}",
            emit_result.files.len(),
            declaration_dir.display()
        );
    }

    if total_errors > 0 {
        std::process::exit(1);
    }
}

#[allow(clippy::disallowed_types)]
fn render_diagnostics(
    diagnostics: &[vize_canon::BatchDiagnostic],
) -> std::collections::BTreeMap<std::string::String, Vec<std::string::String>> {
    let mut grouped = std::collections::BTreeMap::<
        std::string::String,
        Vec<(u32, u32, std::string::String)>,
    >::new();

    for diagnostic in diagnostics {
        let severity = match diagnostic.severity {
            1 => "error",
            2 => "warning",
            3 => "info",
            _ => "hint",
        };
        let code = diagnostic
            .code
            .map(|code| cstr!(" [TS{}]", code))
            .unwrap_or_default();
        let rendered = cstr!(
            "{}:{}:{}{} {}",
            severity,
            diagnostic.line + 1,
            diagnostic.column + 1,
            code,
            diagnostic.message
        )
        .into();
        grouped
            .entry(diagnostic.file.to_string_lossy().into_owned())
            .or_default()
            .push((diagnostic.line, diagnostic.column, rendered));
    }

    grouped
        .into_iter()
        .map(|(file, mut diagnostics)| {
            diagnostics.sort_by(|left, right| {
                left.0
                    .cmp(&right.0)
                    .then_with(|| left.1.cmp(&right.1))
                    .then_with(|| left.2.cmp(&right.2))
            });
            let rendered = diagnostics
                .into_iter()
                .map(|(_, _, rendered)| rendered)
                .collect();
            (file, rendered)
        })
        .collect()
}

fn write_profile_virtual_ts(files: &[&vize_canon::VirtualFile]) {
    let profile_dir = PathBuf::from("node_modules/.vize/check-profile");
    if let Err(error) = fs::create_dir_all(&profile_dir) {
        eprintln!("Failed to create profile directory: {}", error);
        return;
    }

    for file in files {
        let file_name = file
            .original_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| cstr!("{name}.ts"))
            .unwrap_or_else(|| "unknown.ts".into());
        let target = profile_dir.join(file_name.as_str());
        if let Err(error) = fs::write(&target, &file.content) {
            eprintln!("Failed to write {}: {}", target.display(), error);
        }
    }

    eprintln!(
        "\x1b[33mProfile:\x1b[0m Virtual TS files written to {}",
        profile_dir.display()
    );
}

fn build_virtual_ts_options(
    config: &crate::config::VizeConfig,
    cwd: &Path,
) -> vize_canon::virtual_ts::VirtualTsOptions {
    let globals_path = config
        .check
        .globals
        .as_deref()
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                cwd.join(path)
            }
        });

    if let Some(ref globals_path) = globals_path {
        match parse_dts_globals(globals_path) {
            Ok(template_globals) => {
                return vize_canon::virtual_ts::VirtualTsOptions {
                    template_globals,
                    ..Default::default()
                };
            }
            Err(error) => {
                eprintln!(
                    "\x1b[33mWarning:\x1b[0m Failed to parse globals from {}: {}",
                    globals_path.display(),
                    error
                );
            }
        }
    }

    vize_canon::virtual_ts::VirtualTsOptions::default()
}

fn resolve_declaration_dir(declaration_dir: Option<&Path>, project_root: &Path) -> PathBuf {
    declaration_dir
        .map(|path| {
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                project_root.join(path)
            }
        })
        .unwrap_or_else(|| project_root.join("dist").join("types"))
}

fn resolve_project_root(
    explicit_tsconfig: Option<&Path>,
    cwd: &Path,
    files: &[PathBuf],
) -> PathBuf {
    if let Some(tsconfig) = explicit_tsconfig {
        let tsconfig_path = if tsconfig.is_absolute() {
            tsconfig.to_path_buf()
        } else {
            cwd.join(tsconfig)
        };
        return tsconfig_path
            .parent()
            .map(|parent| parent.to_path_buf())
            .unwrap_or_else(|| cwd.to_path_buf());
    }

    for file in files {
        if let Some(root) = find_nearest_tsconfig_dir(file) {
            return root;
        }
    }

    if let Some(root) = find_nearest_tsconfig_dir(cwd) {
        return root;
    }

    cwd.to_path_buf()
}

fn resolve_tsconfig_path(
    explicit_tsconfig: Option<&Path>,
    cwd: &Path,
    project_root: &Path,
    files: &[PathBuf],
) -> Option<PathBuf> {
    if let Some(tsconfig) = explicit_tsconfig {
        let tsconfig_path = if tsconfig.is_absolute() {
            tsconfig.to_path_buf()
        } else {
            cwd.join(tsconfig)
        };
        return Some(tsconfig_path.canonicalize().unwrap_or(tsconfig_path));
    }

    for file in files {
        let Some(root) = find_nearest_tsconfig_dir(file) else {
            continue;
        };
        let candidate = root.join("tsconfig.json");
        if candidate.exists() {
            return Some(candidate);
        }
    }

    let candidate = project_root.join("tsconfig.json");
    candidate.exists().then_some(candidate)
}

fn find_nearest_tsconfig_dir(path: &Path) -> Option<PathBuf> {
    let mut current = if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    };

    while let Some(dir) = current {
        if dir.join("tsconfig.json").exists() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }

    None
}

#[allow(clippy::disallowed_types)]
pub(crate) fn collect_check_files(patterns: &[std::string::String]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut seen = FxHashSet::default();

    for pattern in patterns {
        let candidate = PathBuf::from(pattern);
        if candidate.exists() {
            if candidate.is_file() {
                let candidate = normalize_input_path(&candidate);
                if is_supported_check_file(&candidate) && seen.insert(candidate.clone()) {
                    files.push(candidate);
                }
                continue;
            }
            if candidate.is_dir() {
                collect_from_dir(&candidate, &mut files, &mut seen);
                continue;
            }
        }

        let base_dir = base_dir_from_pattern(pattern);
        collect_from_dir(base_dir.as_path(), &mut files, &mut seen);
    }

    files.sort();
    files
}

#[allow(clippy::disallowed_types)]
pub(crate) fn collect_vue_files(patterns: &[std::string::String]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut seen = FxHashSet::default();

    for pattern in patterns {
        let candidate = PathBuf::from(pattern);
        if candidate.exists() {
            if candidate.is_file() {
                let candidate = normalize_input_path(&candidate);
                if candidate
                    .extension()
                    .and_then(|extension| extension.to_str())
                    == Some("vue")
                    && seen.insert(candidate.clone())
                {
                    files.push(candidate);
                }
                continue;
            }
            if candidate.is_dir() {
                collect_from_dir_filtered(&candidate, &mut files, &mut seen, true);
                continue;
            }
        }

        let base_dir = base_dir_from_pattern(pattern);
        collect_from_dir_filtered(&base_dir, &mut files, &mut seen, true);
    }

    files.sort();
    files
}

fn collect_from_dir(dir: &Path, files: &mut Vec<PathBuf>, seen: &mut FxHashSet<PathBuf>) {
    collect_from_dir_filtered(dir, files, seen, false);
}

fn collect_from_dir_filtered(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
    vue_only: bool,
) {
    let skip_generated = should_skip_generated_for_root(dir);
    let walker = WalkBuilder::new(dir)
        .standard_filters(true)
        .hidden(true)
        .build_parallel();

    let collected = std::sync::Mutex::new(Vec::<PathBuf>::new());
    walker.run(|| {
        let collected = &collected;
        Box::new(move |entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file()
                    && is_supported_collect_file(path, vue_only)
                    && (!skip_generated || !is_generated_path(path))
                {
                    if let Ok(mut collected) = collected.lock() {
                        collected.push(path.to_path_buf());
                    }
                }
            }
            ignore::WalkState::Continue
        })
    });

    let Ok(collected) = collected.into_inner() else {
        return;
    };
    for path in collected {
        let path = normalize_input_path(&path);
        if seen.insert(path.clone()) {
            files.push(path);
        }
    }
}

fn base_dir_from_pattern(pattern: &str) -> PathBuf {
    let glob_start = pattern.find(['*', '?', '[', '{']).unwrap_or(pattern.len());
    let prefix = &pattern[..glob_start];
    let base = if prefix.is_empty() {
        "."
    } else if let Some(index) = prefix.rfind('/') {
        &prefix[..index]
    } else {
        prefix
    };
    if base.is_empty() {
        PathBuf::from(".")
    } else {
        PathBuf::from(base)
    }
}

fn normalize_input_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn should_skip_generated_for_root(root: &Path) -> bool {
    !root
        .components()
        .any(|component| component.as_os_str().to_str() == Some("__agent_only"))
}

fn display_path(base: &Path, path: &Path) -> vize_carton::String {
    path.strip_prefix(base)
        .map(|relative| cstr!("{}", relative.display()))
        .unwrap_or_else(|_| cstr!("{}", path.display()))
}

fn is_generated_path(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|name| matches!(name, "__agent_only" | "target"))
    })
}

fn is_supported_collect_file(path: &Path, vue_only: bool) -> bool {
    if vue_only {
        return path.extension().and_then(|extension| extension.to_str()) == Some("vue");
    }
    is_supported_check_file(path)
}

fn is_supported_check_file(path: &Path) -> bool {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
    {
        return true;
    }

    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "vue" | "ts" | "tsx" | "mts" | "cts"))
}

/// Parse a `.d.ts` file containing `ComponentCustomProperties` augmentation.
fn parse_dts_globals(
    path: &Path,
) -> Result<Vec<vize_canon::virtual_ts::TemplateGlobal>, std::io::Error> {
    use super::dts::parse_interface_members;
    use vize_canon::virtual_ts::TemplateGlobal;

    Ok(
        parse_interface_members(path, "interface ComponentCustomProperties")?
            .into_iter()
            .map(|(name, type_annotation)| TemplateGlobal {
                name,
                type_annotation,
                default_value: "{} as any".into(),
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        base_dir_from_pattern, collect_check_files, collect_vue_files, resolve_declaration_dir,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use vize_carton::cstr;

    fn unique_case_dir(name: &str) -> PathBuf {
        static NEXT_CASE_ID: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("__agent_only")
            .join("tests")
            .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
    }

    #[test]
    fn base_dir_from_glob_patterns() {
        assert_eq!(
            base_dir_from_pattern("./src/**/*.vue"),
            PathBuf::from("./src")
        );
        assert_eq!(base_dir_from_pattern("."), PathBuf::from("."));
    }

    #[test]
    fn collect_check_files_includes_ts_and_vue_and_dts() {
        let case_dir = unique_case_dir("collect-check");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(case_dir.join("src")).unwrap();
        fs::write(case_dir.join("src/App.vue"), "").unwrap();
        fs::write(case_dir.join("src/main.ts"), "").unwrap();
        fs::write(case_dir.join("src/env.d.ts"), "").unwrap();
        fs::write(case_dir.join("src/skip.js"), "").unwrap();

        let files = collect_check_files(&vec![case_dir.display().to_string()]);

        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|path| path.ends_with("App.vue")));
        assert!(files.iter().any(|path| path.ends_with("main.ts")));
        assert!(files.iter().any(|path| path.ends_with("env.d.ts")));

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn collect_vue_files_stays_vue_only() {
        let case_dir = unique_case_dir("collect-vue");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(case_dir.join("src")).unwrap();
        fs::write(case_dir.join("src/App.vue"), "").unwrap();
        fs::write(case_dir.join("src/main.ts"), "").unwrap();

        let files = collect_vue_files(&vec![case_dir.display().to_string()]);

        assert_eq!(files, vec![case_dir.join("src/App.vue")]);

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn resolve_declaration_dir_defaults_to_dist_types() {
        let project_root = PathBuf::from("/workspace/project");
        assert_eq!(
            resolve_declaration_dir(None, &project_root),
            project_root.join("dist").join("types")
        );
        assert_eq!(
            resolve_declaration_dir(Some(Path::new("types")), &project_root),
            project_root.join("types")
        );
    }
}

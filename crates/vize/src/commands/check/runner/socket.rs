#![allow(clippy::disallowed_macros)]

use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    time::{Duration, Instant},
};

use vize_carton::{String, cstr, profile, profiler::global_profiler};

use crate::{
    commands::check::{
        CheckArgs, JsonRpcResponse, ServerCheckResult,
        reporting::{JsonFileResult, JsonOutput},
    },
    profile_support,
};
use super::{
    collect::collect_vue_files, display_path, output::save_virtual_ts_targets_or_exit,
    text_style::TextStyle,
};
use vize_curator::profile::{ProfilePhase, ProfilePhaseKind, ProfileReport, print_profile_report};

pub(crate) fn run_with_socket(args: &CheckArgs, socket_path: &str) {
    let start = Instant::now();
    args.profile_export.begin(args.profile);

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
            let style = TextStyle::stderr();
            eprintln!(
                "{} Failed to connect to check-server: {}",
                style.red("Error:"),
                error
            );
            eprintln!();
            eprintln!("{} Start the server first:", style.yellow("Hint:"));
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
    let mut total_warnings = 0usize;
    let mut shown_shared_helpers = false;
    #[allow(clippy::disallowed_types, clippy::disallowed_methods)]
    let mut results: Vec<(std::string::String, ServerCheckResult)> = Vec::new();

    let request_start = Instant::now();
    for path in &files {
        #[allow(clippy::disallowed_types)]
        let source = match profile!("cli.check.socket.file.read", fs::read_to_string(path)) {
            Ok(source) => {
                global_profiler().record_fs_read_to_string(source.len());
                source
            }
            Err(error) => {
                global_profiler().record_fs_read_to_string_failure();
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

        let request_payload = match serde_json::to_string(&request) {
            Ok(payload) => payload,
            Err(error) => {
                eprintln!("Failed to encode request: {}", error);
                continue;
            }
        };
        let request_bytes = request_payload.len() + 1;
        if writeln!(stream, "{request_payload}").is_err() || stream.flush().is_err() {
            global_profiler().record_counter("io.socket.write.calls", 1);
            global_profiler()
                .record_counter("io.socket.write.attempted_bytes", request_bytes as u64);
            global_profiler().record_counter("io.socket.write.failures", 1);
            global_profiler().record_counter("syscall.socket.write.calls", 1);
            global_profiler().record_counter("syscall.socket.write.failures", 1);
            eprintln!("Failed to send request");
            break;
        }
        global_profiler().record_counter("io.socket.write.calls", 1);
        global_profiler().record_counter("io.socket.write.attempted_bytes", request_bytes as u64);
        global_profiler().record_counter("io.socket.write.bytes", request_bytes as u64);
        global_profiler().record_counter("syscall.socket.write.calls", 1);
        global_profiler().record_counter("syscall.socket.flush.calls", 1);

        let mut reader = BufReader::new(&stream);
        #[allow(clippy::disallowed_types)]
        let mut response_line = std::string::String::new();
        if reader.read_line(&mut response_line).is_err() {
            global_profiler().record_counter("io.socket.read.calls", 1);
            global_profiler().record_counter("io.socket.read.failures", 1);
            global_profiler().record_counter("syscall.socket.read.calls", 1);
            global_profiler().record_counter("syscall.socket.read.failures", 1);
            eprintln!("Failed to read response");
            break;
        }
        global_profiler().record_counter("io.socket.read.calls", 1);
        global_profiler().record_counter("io.socket.read.bytes", response_line.len() as u64);
        global_profiler().record_counter("syscall.socket.read.calls", 1);

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
            total_warnings += result
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.severity == "warning")
                .count();
            if args.show_virtual_ts && !shown_shared_helpers {
                eprintln!(
                    "\n=== {} ===",
                    vize_canon::virtual_ts::SHARED_PREAMBLE_FILE_NAME
                );
                eprintln!("{}", vize_canon::virtual_ts::SHARED_PREAMBLE_DTS);
                shown_shared_helpers = true;
            }
            if args.show_virtual_ts {
                eprintln!("\n=== {} ===", filename);
                eprintln!("{}", result.virtual_ts);
            }
            results.push((filename, result));
        }
    }
    let request_time = request_start.elapsed();

    if !args.save_virtual_ts_for.is_empty() {
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        save_virtual_ts_targets_or_exit(
            &args.save_virtual_ts_for,
            &cwd,
            || {
                results.iter().map(|(filename, result)| {
                    (std::path::Path::new(filename), result.virtual_ts.as_str())
                })
            },
            args.quiet,
        );
    }
    let render_start = Instant::now();
    if args.format == "json" {
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let mut files_json: Vec<JsonFileResult> = results
            .iter()
            .map(|(filename, result)| {
                let path = std::path::Path::new(filename);
                JsonFileResult {
                    file: display_path(&cwd, path).into(),
                    virtual_ts: args.show_virtual_ts.then(|| result.virtual_ts.clone()),
                    diagnostics: render_socket_diagnostics(result),
                }
            })
            .collect();
        files_json.sort_by(|left, right| left.file.cmp(&right.file));

        let json_output = JsonOutput {
            files: files_json,
            programs: Vec::new(),
            error_count: total_errors,
            warning_count: total_warnings,
            file_count: results.len(),
            declarations: None,
        };
        match serde_json::to_string_pretty(&json_output) {
            Ok(output) => println!("{output}"),
            Err(error) => {
                eprintln!("Failed to serialize check output: {error}");
                std::process::exit(1);
            }
        }
        if total_errors > 0 {
            std::process::exit(1);
        }
        return;
    }

    if !args.quiet {
        let style = TextStyle::stdout();
        for (filename, result) in &results {
            if result.diagnostics.is_empty() {
                continue;
            }
            println!("\n{}", style.underline(filename));
            for diagnostic in &result.diagnostics {
                let location = cstr!(
                    "{}:{}:{}",
                    diagnostic.severity,
                    diagnostic.line,
                    diagnostic.column
                );
                let location = if diagnostic.severity == "error" {
                    style.red(location)
                } else {
                    style.yellow(location)
                };
                let code = diagnostic
                    .code
                    .as_ref()
                    .map(|code| cstr!(" [{}]", code))
                    .unwrap_or_default();
                println!("  {}{} {}", location, code, diagnostic.message);
            }
        }
    }
    let render_time = render_start.elapsed();
    let total_time = start.elapsed();

    let style = TextStyle::stdout();
    let status = if total_errors > 0 {
        style.red("\u{2717}")
    } else {
        style.green("\u{2713}")
    };
    println!(
        "\n{} Type checked {} Vue files in {:.2?} (via socket)",
        status,
        files.len(),
        total_time
    );
    args.profile_export.finish("check", args.profile);
    if args.profile {
        let profiler = global_profiler();
        let allocation_summary = profile_support::allocation_snapshot();
        let counter_summary = profiler.counter_summary();
        let operation_summary = profiler.summary();
        profiler.disable();
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
            operations: Some(&operation_summary),
            counters: Some(&counter_summary),
            allocations: allocation_summary,
            recommendations: &recommendations,
        };
        print_profile_report(&report);
    }
    if total_errors > 0 {
        println!("  {}", style.red(cstr!("{} error(s)", total_errors)));
        std::process::exit(1);
    }
    println!("  {}", style.green("No type errors found!"));
}

#[allow(clippy::disallowed_types)]
fn render_socket_diagnostics(result: &ServerCheckResult) -> Vec<std::string::String> {
    let mut diagnostics = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let code = diagnostic
                .code
                .as_ref()
                .map(|code| {
                    if code.chars().all(|char| char.is_ascii_digit()) {
                        cstr!(" [TS{}]", code)
                    } else {
                        cstr!(" [{}]", code)
                    }
                })
                .unwrap_or_default();
            cstr!(
                "{}:{}:{}{} {}",
                diagnostic.severity,
                diagnostic.line,
                diagnostic.column,
                code,
                diagnostic.message
            )
            .into()
        })
        .collect::<Vec<_>>();
    diagnostics.sort();
    diagnostics
}

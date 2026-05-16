//! Unix socket client path for `vize check`.
//!
//! This mode keeps repeated checks fast by sending file contents to an already
//! running check server and rendering its diagnostics locally.

#![allow(clippy::disallowed_macros)]

use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    time::{Duration, Instant},
};

use vize_carton::{cstr, String};

use crate::commands::{
    check::{CheckArgs, JsonRpcResponse, ServerCheckResult},
    profile::{print_profile_report, ProfilePhase, ProfilePhaseKind, ProfileReport},
};

use super::collect::collect_vue_files;

/// Run type checking via Unix socket connection to check-server.
pub(crate) fn run_with_socket(args: &CheckArgs, socket_path: &str) {
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

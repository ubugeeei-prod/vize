use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

use serde::Serialize;
use vize_l0::{cstr, String};

use super::collect::collect_check_files_with_ignores;
use super::{display_path, resolve_project_root, resolve_tsconfig_path, text_style::TextStyle};
use crate::commands::check::{
    patterns::CheckFileOptions,
    reporting::{JsonFileResult, JsonOutput},
    tsconfig_inputs::{collect_default_check_files, tsconfig_allows_js, TsconfigInputCache},
    CheckArgs, ProjectJsonRpcResponse, ServerProjectCheckResult,
};

#[derive(Serialize)]
#[expect(clippy::disallowed_types, reason = "JSON protocol uses std String")]
struct ProjectFileRequest {
    uri: std::string::String,
    len: u64,
    #[serde(rename = "modifiedNanos")]
    modified_nanos: u128,
}

#[expect(clippy::disallowed_types, reason = "JSON protocol uses std String")]
pub(crate) fn run_with_socket(args: &CheckArgs, socket_path: &str) {
    // Virtual-source and declaration output need the direct runner's richer
    // artifact contract. The normal diagnostic path uses one project RPC.
    if args.show_virtual_ts || !args.save_virtual_ts_for.is_empty() {
        super::socket_legacy::run_legacy_with_socket(args, socket_path);
        return;
    }
    if args.declaration {
        super::run_direct(args);
        return;
    }

    args.profile_export.begin(args.profile);
    let started = Instant::now();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_root = resolve_project_root(args.tsconfig.as_deref(), &cwd, &[]);
    let tsconfig = resolve_tsconfig_path(args.tsconfig.as_deref(), &cwd, &project_root, &[]);
    let mut tsconfig_cache = TsconfigInputCache::default();
    let include_js = tsconfig
        .as_deref()
        .is_some_and(|path| tsconfig_allows_js(path, &mut tsconfig_cache));
    let files = if args.patterns.is_empty() {
        collect_default_check_files(
            &project_root,
            tsconfig.as_deref(),
            false,
            &mut tsconfig_cache,
        )
    } else {
        collect_check_files_with_ignores(
            &args.patterns,
            CheckFileOptions {
                include_js,
                include_jsx: false,
            },
            None,
        )
    };
    if files.is_empty() {
        super::report_no_inputs(args, tsconfig.as_deref());
        return;
    }

    let request_files = files
        .iter()
        .map(|path| file_request(path))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|error| exit_error(error));
    let tsconfig_request = tsconfig
        .as_deref()
        .map(file_request)
        .transpose()
        .unwrap_or_else(|error| exit_error(error));
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "check-project",
        "params": {
            "files": request_files,
            "tsconfig": tsconfig_request,
        }
    });
    let payload = serde_json::to_string(&request)
        .unwrap_or_else(|error| exit_error(cstr!("Failed to encode request: {error}")));

    let mut stream = UnixStream::connect(socket_path).unwrap_or_else(|error| {
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
    });
    if !args.quiet {
        eprintln!("Connected to check-server at {socket_path}");
        eprintln!("Type checking {} files...", files.len());
    }
    writeln!(stream, "{payload}")
        .and_then(|()| stream.flush())
        .unwrap_or_else(|error| exit_error(cstr!("Failed to send request: {error}")));
    let mut response_line = std::string::String::new();
    BufReader::new(&stream)
        .read_line(&mut response_line)
        .unwrap_or_else(|error| exit_error(cstr!("Failed to read response: {error}")));
    let response: ProjectJsonRpcResponse = serde_json::from_str(&response_line)
        .unwrap_or_else(|error| exit_error(cstr!("Failed to parse response: {error}")));
    if let Some(error) = response.error {
        exit_error(cstr!("Server error: {}", error.message));
    }
    let result = response
        .result
        .unwrap_or_else(|| exit_error("Server returned no result".into()));
    render_result(args, &cwd, &files, result, started);
}

fn file_request(path: &Path) -> Result<ProjectFileRequest, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| cstr!("Failed to read metadata for {}: {error}", path.display()))?;
    let modified_nanos = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map_or(0, |duration| duration.as_nanos());
    Ok(ProjectFileRequest {
        uri: path.to_string_lossy().into_owned(),
        len: metadata.len(),
        modified_nanos,
    })
}

#[expect(clippy::disallowed_types, reason = "JSON diagnostics use std String")]
fn render_result(
    args: &CheckArgs,
    cwd: &Path,
    files: &[PathBuf],
    result: ServerProjectCheckResult,
    started: Instant,
) {
    let mut grouped = BTreeMap::<std::string::String, Vec<std::string::String>>::new();
    for diagnostic in result.diagnostics {
        let code = diagnostic
            .code
            .as_deref()
            .map(|code| cstr!(" [{code}]"))
            .unwrap_or_default();
        grouped.entry(diagnostic.file).or_default().push(
            cstr!(
                "{}:{}:{}{} {}",
                diagnostic.severity,
                diagnostic.line,
                diagnostic.column,
                code,
                diagnostic.message
            )
            .into(),
        );
    }
    for diagnostics in grouped.values_mut() {
        diagnostics.sort();
    }

    if args.format == "json" {
        let files_json = files
            .iter()
            .map(|path| JsonFileResult {
                file: display_path(cwd, path).into(),
                virtual_ts: None,
                diagnostics: grouped
                    .remove(
                        vize_carton::path::canonicalize_non_verbatim(path)
                            .to_string_lossy()
                            .as_ref(),
                    )
                    .unwrap_or_default(),
            })
            .collect();
        let output = JsonOutput {
            files: files_json,
            programs: Vec::new(),
            error_count: result.error_count,
            warning_count: result.warning_count,
            file_count: result.file_count,
            declarations: None,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&output)
                .unwrap_or_else(|error| exit_error(cstr!("Failed to serialize output: {error}")))
        );
    } else {
        if !args.quiet {
            let style = TextStyle::stdout();
            for (filename, diagnostics) in grouped {
                if diagnostics.is_empty() {
                    continue;
                }
                println!(
                    "\n{}",
                    style.underline(display_path(cwd, Path::new(&filename)))
                );
                for diagnostic in diagnostics {
                    println!("  {diagnostic}");
                }
            }
        }
        let style = TextStyle::stdout();
        let status = if result.error_count == 0 {
            style.green("\u{2713}")
        } else {
            style.red("\u{2717}")
        };
        println!(
            "\n{} Type checked {} files in {:.2?} (via socket)",
            status,
            result.file_count,
            started.elapsed()
        );
        if result.error_count == 0 {
            println!("  {}", style.green("No type errors found!"));
        } else {
            println!("  {}", style.red(cstr!("{} error(s)", result.error_count)));
        }
    }

    args.profile_export.finish("check", args.profile);
    let warnings_exceeded = args
        .max_warnings
        .is_some_and(|maximum| result.warning_count > maximum);
    if result.error_count > 0 || warnings_exceeded {
        std::process::exit(1);
    }
}

fn exit_error(message: String) -> ! {
    eprintln!("{message}");
    std::process::exit(1)
}

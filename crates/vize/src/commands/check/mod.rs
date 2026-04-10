//! Check command - Type check Vue SFC files
//!
//! Generates Virtual TypeScript from Vue SFCs and uses Corsa project sessions for type checking.
//! Can connect to a running check-server via Unix socket for faster repeated checks.

mod dts;
mod nuxt;
mod reporting;
mod runner;
mod tsconfig_inputs;

use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
#[allow(clippy::disallowed_types)]
pub struct CheckArgs {
    /// Files or directories to type-check (`.vue`, `.ts`, `.tsx`, `.d.ts`).
    /// When omitted, `tsconfig.json` include/exclude/files are used if available.
    pub patterns: Vec<String>,

    /// Connect to check-server via Unix socket (faster for repeated checks, Unix only)
    #[cfg(unix)]
    #[arg(long, short)]
    pub socket: Option<String>,

    /// tsconfig.json path
    #[arg(long)]
    pub tsconfig: Option<PathBuf>,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Show generated virtual TypeScript
    #[arg(long)]
    pub show_virtual_ts: bool,

    /// Quiet mode - only show summary
    #[arg(short, long)]
    pub quiet: bool,

    /// Profile mode - output Virtual TS and timing to node_modules/.vize directory
    #[arg(long)]
    pub profile: bool,

    /// Path to the Corsa executable (can also use `CORSA_PATH`)
    #[arg(long)]
    pub corsa_path: Option<String>,

    /// Number of parallel Corsa servers to use (defaults to an auto-tuned value)
    #[arg(long)]
    pub servers: Option<usize>,

    /// Emit `.d.ts` files using the materialized Corsa project
    #[arg(long)]
    pub declaration: bool,

    /// Output directory for emitted `.d.ts` files
    #[arg(long)]
    pub declaration_dir: Option<PathBuf>,
}

/// Serde types for check-server JSON-RPC communication (Unix only).
#[cfg(unix)]
#[allow(clippy::disallowed_types)]
pub(crate) mod unix_types {
    use serde::Deserialize;

    /// Server response for check method.
    #[derive(Deserialize)]
    pub(crate) struct ServerCheckResult {
        pub diagnostics: Vec<ServerDiagnostic>,
        #[serde(rename = "virtualTs")]
        pub virtual_ts: String,
        #[serde(rename = "errorCount")]
        pub error_count: usize,
    }

    #[derive(Deserialize)]
    pub(crate) struct ServerDiagnostic {
        pub message: String,
        pub severity: String,
        pub line: u32,
        pub column: u32,
        pub code: Option<String>,
    }

    #[derive(Deserialize)]
    pub(crate) struct JsonRpcResponse {
        pub result: Option<ServerCheckResult>,
        pub error: Option<JsonRpcError>,
    }

    #[derive(Deserialize)]
    pub(crate) struct JsonRpcError {
        #[allow(dead_code)]
        pub code: i64,
        pub message: String,
    }
}

#[cfg(unix)]
pub(crate) use unix_types::*;

pub fn run(args: CheckArgs) {
    // If socket is specified, use socket client mode (Unix only)
    #[cfg(unix)]
    if let Some(ref socket_path) = args.socket {
        runner::run_with_socket(&args, socket_path);
        return;
    }

    // Otherwise, fall back to direct Corsa execution.
    runner::run_direct(&args);
}

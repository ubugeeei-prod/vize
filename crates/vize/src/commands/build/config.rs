//! Build configuration types, compile statistics, and profiling structures.
//!
//! Contains internal data types used during the compilation pipeline:
//! statistics tracking, per-file profiling, error collection, and output formats.

#![allow(clippy::disallowed_macros)]

use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    time::Duration,
};

use super::ScriptExtension;
use vize_carton::cstr;
use vize_carton::String;

/// Aggregate compile statistics shared across worker threads.
#[derive(Debug)]
pub(crate) struct CompileStats {
    pub total_files: usize,
    pub success: AtomicUsize,
    pub failed: AtomicUsize,
    pub total_bytes: AtomicUsize,
    pub output_bytes: AtomicUsize,
    pub total_parse_nanos: AtomicU64,
    pub total_compile_nanos: AtomicU64,
}

impl CompileStats {
    pub fn new(total_files: usize) -> Self {
        Self {
            total_files,
            success: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
            total_bytes: AtomicUsize::new(0),
            output_bytes: AtomicUsize::new(0),
            total_parse_nanos: AtomicU64::new(0),
            total_compile_nanos: AtomicU64::new(0),
        }
    }

    pub fn add_parse_time(&self, duration: Duration) {
        self.total_parse_nanos.fetch_add(
            duration.as_nanos().try_into().unwrap_or(u64::MAX),
            Ordering::Relaxed,
        );
    }

    pub fn add_compile_time(&self, duration: Duration) {
        self.total_compile_nanos.fetch_add(
            duration.as_nanos().try_into().unwrap_or(u64::MAX),
            Ordering::Relaxed,
        );
    }

    pub fn total_parse_time(&self) -> Duration {
        Duration::from_nanos(self.total_parse_nanos.load(Ordering::Relaxed))
    }

    pub fn total_compile_time(&self) -> Duration {
        Duration::from_nanos(self.total_compile_nanos.load(Ordering::Relaxed))
    }
}

/// Serializable output for a single compiled file.
#[derive(Debug, serde::Serialize)]
pub(crate) struct CompileOutput {
    pub filename: String,
    pub code: String,
    pub css: Option<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub script_lang: String,
}

/// Detailed timing information for a single file.
#[derive(Debug, Clone)]
pub(crate) struct FileProfile {
    pub path: PathBuf,
    pub file_size: usize,
    pub parse_time: Duration,
    pub compile_time: Duration,
    pub total_time: Duration,
    pub template_size: usize,
    pub script_size: usize,
    pub style_count: usize,
}

impl FileProfile {
    pub fn is_slow(&self, threshold: Duration) -> bool {
        self.total_time > threshold
    }

    pub fn suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        if self.template_size > 10000 {
            suggestions.push(cstr!(
                "Large template ({} bytes) - consider splitting into smaller components",
                self.template_size
            ));
        }

        if self.script_size > 20000 {
            suggestions.push(cstr!(
                "Large script ({} bytes) - consider extracting logic into composables",
                self.script_size
            ));
        }

        if self.style_count > 3 {
            suggestions.push(cstr!(
                "Multiple style blocks ({}) - consider consolidating styles",
                self.style_count
            ));
        }

        if self.parse_time > self.compile_time * 2 {
            suggestions.push(
                "Parsing is slow - check for complex template expressions or deeply nested elements"
                    .into(),
            );
        }

        suggestions
    }
}

/// Collected error information from a failed compilation.
#[derive(Debug, Clone)]
pub(crate) struct CompileError {
    pub path: PathBuf,
    pub error: String,
    pub phase: ErrorPhase,
}

/// Which phase of the compile pipeline an error occurred in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErrorPhase {
    Read,
    Parse,
    Compile,
}

impl std::fmt::Display for ErrorPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorPhase::Read => write!(f, "read"),
            ErrorPhase::Parse => write!(f, "parse"),
            ErrorPhase::Compile => write!(f, "compile"),
        }
    }
}

/// Determine the output file extension based on script language and extension mode.
pub(crate) fn get_output_extension(script_lang: &str, script_ext: ScriptExtension) -> &'static str {
    match script_ext {
        ScriptExtension::Downcompile => "js",
        ScriptExtension::Preserve => match script_lang {
            "ts" => "ts",
            "tsx" => "tsx",
            "jsx" => "jsx",
            _ => "js",
        },
    }
}

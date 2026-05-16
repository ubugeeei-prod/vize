//! Build command - Compile Vue SFC files
//!
//! Parses and compiles `.vue` Single File Components into JavaScript (or JSON),
//! with parallel processing, profiling, and error collection.

mod config;
mod runner;

use clap::{Args, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum OutputFormat {
    /// Output compiled JavaScript
    #[default]
    Js,
    /// Output JSON with code and metadata
    Json,
    /// Only show statistics (no output)
    Stats,
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum ScriptExtension {
    /// Preserve original script language extension (.ts -> .ts, .tsx -> .tsx, .jsx -> .jsx)
    Preserve,
    /// Downcompile all scripts to JavaScript (.ts -> .js, .tsx -> .js, .jsx -> .js)
    #[default]
    Downcompile,
}

#[derive(Args, Default)]
#[allow(clippy::disallowed_types)]
pub struct BuildArgs {
    /// Glob pattern(s) to match .vue files (default: ./**/*.vue)
    #[arg(default_value = "./**/*.vue")]
    pub patterns: Vec<String>,

    /// Output directory (default: ./dist)
    #[arg(short, long, default_value = "./dist")]
    pub output: PathBuf,

    /// Output format
    #[arg(short, long, value_enum, default_value = "js")]
    pub format: OutputFormat,

    /// Enable SSR mode
    #[arg(long)]
    pub ssr: bool,

    /// Script extension handling: 'preserve' keeps original extension (.ts/.tsx/.jsx), 'downcompile' converts to .js
    #[arg(long, value_enum, default_value = "downcompile")]
    pub script_ext: ScriptExtension,

    /// Number of threads (default: number of CPUs)
    #[arg(short = 'j', long)]
    pub threads: Option<usize>,

    /// Show timing profile breakdown
    #[arg(long)]
    pub profile: bool,

    /// Slow file threshold in milliseconds (default: 100)
    #[arg(long, default_value = "100")]
    pub slow_threshold: u64,

    /// Continue on errors (collect all errors and show at end)
    #[arg(long)]
    pub continue_on_error: bool,
}

pub fn run(args: BuildArgs) {
    runner::run(args);
}

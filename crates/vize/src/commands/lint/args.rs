//! Command-line arguments for the lint command.

use clap::Args;
use std::path::PathBuf;
use vize_s0::String;

use super::patterns::LINT_DEFAULT_PATTERNS;

#[derive(Args)]
#[allow(clippy::disallowed_types)]
pub struct LintArgs {
    /// Glob pattern(s) to match files supported by vize lint
    #[arg(default_values = LINT_DEFAULT_PATTERNS)]
    pub patterns: Vec<String>,

    /// Automatically fix problems when diagnostics provide safe text edits
    #[arg(long)]
    pub fix: bool,

    /// Config file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Do not load a config file
    #[arg(long)]
    pub no_config: bool,

    /// Output format (text, ansi, plain, json, stylish, markdown, html, agent)
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Maximum number of warnings before failing
    #[arg(long)]
    pub max_warnings: Option<usize>,

    /// Quiet mode - only show summary
    #[arg(short, long)]
    pub quiet: bool,

    /// Help display level: full (default), short, none
    #[arg(long, default_value = "full")]
    pub help_level: String,

    /// Override the configured lint preset: happy-path, ecosystem, opinionated, essential, incremental, nuxt
    #[arg(long)]
    pub preset: Option<String>,

    /// Enable opt-in cross-file lint checks for provide/inject, reactivity flow, and race risks.
    #[arg(long)]
    pub cross_file: bool,

    /// Print the provide/inject tree when cross-file lint is enabled.
    #[arg(long)]
    pub cross_file_tree: bool,

    /// Print cross-file complexity score and top hotspots when cross-file lint is enabled.
    #[arg(long)]
    pub cross_file_complexity: bool,

    /// Enable native type-aware lint rules from the active lint configuration.
    #[arg(long)]
    pub type_aware: bool,

    /// Enable opt-in type-aware reactivity-loss linting through the native checker.
    #[arg(long)]
    pub strict_reactivity: bool,

    /// Show detailed timing profile
    #[arg(long)]
    pub profile: bool,

    /// Slow file threshold in milliseconds for profile output
    #[arg(long, default_value = "100")]
    pub slow_threshold: u64,

    #[command(flatten)]
    pub profile_export: crate::commands::profile_export::ProfileExportArgs,
}

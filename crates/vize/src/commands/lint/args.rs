//! Command-line arguments for the lint command.

use clap::{Args, ValueEnum};
use std::path::PathBuf;
use vize_l0::String;

#[derive(Clone, Copy, ValueEnum)]
pub enum LintPresetArg {
    HappyPath,
    GeneralRecommended,
    Default,
    Recommended,
    Ecosystem,
    Opinionated,
    Strict,
    All,
    Essential,
    Incremental,
    Nuxt,
}

impl LintPresetArg {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::HappyPath => "happy-path",
            Self::GeneralRecommended => "general-recommended",
            Self::Default => "default",
            Self::Recommended => "recommended",
            Self::Ecosystem => "ecosystem",
            Self::Opinionated => "opinionated",
            Self::Strict => "strict",
            Self::All => "all",
            Self::Essential => "essential",
            Self::Incremental => "incremental",
            Self::Nuxt => "nuxt",
        }
    }
}

#[derive(Args)]
pub struct LintArgs {
    /// Glob pattern(s) to match files supported by vize lint
    #[arg(
        help = "Glob pattern(s) to match files supported by vize lint\n\n[default: ./**/*.vue ./**/*.html ./**/*.htm ./**/*.js ./**/*.mjs ./**/*.cjs ./**/*.ts ./**/*.mts ./**/*.cts ./**/*.jsx ./**/*.tsx]"
    )]
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

    /// Output format (text, rich, ansi, plain, json, stylish, markdown, html, agent)
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Language of diagnostic messages and the rich renderer: en, ja, zh
    #[arg(long, default_value = "en")]
    pub locale: String,

    /// Maximum number of warnings before failing
    #[arg(long)]
    pub max_warnings: Option<usize>,

    /// Report `eslint-disable` template comments that suppressed nothing
    #[arg(long)]
    pub report_unused_disable_directives: bool,

    /// Quiet mode - only show summary
    #[arg(short, long)]
    pub quiet: bool,

    /// Help display level: full (default), short, none
    #[arg(long, default_value = "full")]
    pub help_level: String,

    /// Override the configured lint preset
    #[arg(long, value_enum)]
    pub preset: Option<LintPresetArg>,

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

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

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TemplateSyntaxArg {
    /// Warn and rewrite recoverable invalid template syntax.
    Standard,
    /// Report recoverable invalid template syntax as errors.
    Strict,
    /// Preserve template syntax compatibility quirks without warnings.
    Quirks,
}

impl From<TemplateSyntaxArg> for vize_atelier_core::TemplateSyntaxMode {
    fn from(value: TemplateSyntaxArg) -> Self {
        match value {
            TemplateSyntaxArg::Standard => Self::Standard,
            TemplateSyntaxArg::Strict => Self::Strict,
            TemplateSyntaxArg::Quirks => Self::Quirks,
        }
    }
}

#[derive(Args, Default)]
#[allow(clippy::disallowed_types)]
pub struct BuildArgs {
    /// File, directory, or glob inputs (output paths stay relative to their common root)
    ///
    /// Existing paths are literal. Globs support *, ?, [...], and recursive **;
    /// quote patterns in the shell.
    /// Backslashes are separators, not escapes. Use [*], [?], [[], or []] for
    /// literal *, ?, [, or ].
    #[arg(default_value = "./**/*.vue")]
    pub patterns: Vec<String>,

    /// Output directory (default: ./dist)
    #[arg(short, long, default_value = "./dist")]
    pub output: PathBuf,

    /// Config file path (accepted for npm CLI compatibility)
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Do not load a config file
    #[arg(long)]
    pub no_config: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value = "js")]
    pub format: OutputFormat,

    /// Enable SSR mode
    #[arg(long)]
    pub ssr: bool,

    /// Enable Vapor mode
    #[arg(long)]
    pub vapor: bool,

    /// Target a custom renderer instead of DOM runtime helpers
    #[arg(long)]
    pub custom_renderer: bool,

    /// Tag patterns compiled as custom elements instead of Vue components
    ///
    /// Repeat the flag for multiple patterns. `Tres*` matches PascalCase
    /// renderer tags such as `<TresMesh>` without treating imported
    /// components as elements.
    #[arg(long, value_name = "PATTERN")]
    pub custom_elements: Vec<String>,

    /// Template syntax compatibility mode
    #[arg(long, value_enum)]
    pub template_syntax: Option<TemplateSyntaxArg>,

    /// Script extension handling: 'preserve' keeps original extension (.ts/.tsx/.jsx), 'downcompile' converts to .js
    #[arg(long, value_enum, default_value = "downcompile")]
    pub script_ext: ScriptExtension,

    /// Emit `.d.ts` declaration files for the built SFCs
    ///
    /// Runs the same Corsa-backed emit as `vize check --declaration` over the
    /// build inputs. Declarations mirror TypeScript's rootDir layout, so a
    /// src-rooted library lands next to its compiled JavaScript.
    #[arg(long, alias = "dts")]
    pub declaration: bool,

    /// Output directory for emitted `.d.ts` files (default: the build output directory)
    #[arg(long, value_name = "DIR", requires = "declaration")]
    pub declaration_dir: Option<PathBuf>,

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

    /// Write per-pass Davinci folio dumps for davinci-driven compiles into DIR
    ///
    /// The compile path has no folio-printable stage artifact until the S2
    /// build path lands (davinci-road P2-12b), so today a build writes the
    /// directory and no pages; `davinci-opt --folio-dir` dumps real pages.
    #[arg(long, value_name = "DIR")]
    pub folio_dir: Option<PathBuf>,

    /// Only dump a pass's folio when the artifact hash changed across it
    #[arg(long, requires = "folio_dir")]
    pub folio_after_change: bool,

    /// Inject a panic for the crash-repro machinery (TS-23): '<file-stem>:<pass>'
    #[arg(long, hide = true, value_name = "SPEC")]
    pub davinci_inject_panic: Option<String>,

    #[command(flatten)]
    pub profile_export: super::profile_export::ProfileExportArgs,
}

pub fn run(args: BuildArgs) {
    runner::run(args);
}

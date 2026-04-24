//! Ready command - Run the standard pre-publish local checks.

use clap::Args;
use std::path::PathBuf;

use crate::commands::{
    build::{BuildArgs, OutputFormat, ScriptExtension},
    check::CheckArgs,
    lint::LintArgs,
};
use vize_carton::ToCompactString;

#[cfg(feature = "glyph")]
use crate::commands::fmt::FmtArgs;

#[derive(Args)]
#[allow(clippy::disallowed_types)]
pub struct ReadyArgs {
    /// Files or directories to process.
    #[arg(default_value = "./**/*.vue")]
    pub patterns: Vec<String>,

    /// Output directory for the build step.
    #[arg(short, long, default_value = "./dist")]
    pub output: PathBuf,

    /// Enable SSR mode for the build step.
    #[arg(long)]
    pub ssr: bool,

    /// Script extension handling for the build step.
    #[arg(long, value_enum, default_value = "downcompile")]
    pub script_ext: ScriptExtension,
}

pub fn run(args: ReadyArgs) {
    #[cfg(feature = "glyph")]
    {
        eprintln!("vize ready: fmt");
        crate::commands::fmt::run(FmtArgs {
            patterns: args.patterns.clone(),
            check: false,
            write: true,
            config: None,
            single_quote: None,
            print_width: None,
            tab_width: None,
            use_tabs: None,
            no_semi: false,
            sort_attributes: None,
            single_attribute_per_line: None,
            max_attributes_per_line: None,
            normalize_directive_shorthands: None,
            profile: false,
            slow_threshold: 100,
        });
    }

    eprintln!("vize ready: lint");
    crate::commands::lint::run(LintArgs {
        patterns: args
            .patterns
            .iter()
            .map(|pattern| pattern.to_compact_string())
            .collect(),
        fix: false,
        config: None,
        format: "text".into(),
        max_warnings: None,
        quiet: false,
        help_level: "full".into(),
        preset: "happy-path".into(),
        profile: false,
        slow_threshold: 100,
    });

    eprintln!("vize ready: check");
    crate::commands::check::run(CheckArgs {
        patterns: args.patterns.clone(),
        #[cfg(unix)]
        socket: None,
        tsconfig: None,
        format: "text".into(),
        show_virtual_ts: false,
        quiet: false,
        profile: false,
        corsa_path: None,
        servers: None,
        declaration: false,
        declaration_dir: None,
    });

    eprintln!("vize ready: build");
    crate::commands::build::run(BuildArgs {
        patterns: args.patterns,
        output: args.output,
        format: OutputFormat::Js,
        ssr: args.ssr,
        script_ext: args.script_ext,
        threads: None,
        profile: false,
        slow_threshold: 100,
        continue_on_error: false,
    });
}

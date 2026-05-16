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
            no_config: false,
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
        no_config: false,
        format: "text".into(),
        max_warnings: None,
        quiet: false,
        help_level: "full".into(),
        preset: "happy-path".into(),
        cross_file: false,
        cross_file_tree: false,
        strict_reactivity: false,
        profile: false,
        slow_threshold: 100,
    });

    eprintln!("vize ready: check");
    crate::commands::check::run(CheckArgs {
        patterns: args.patterns.clone(),
        config: None,
        no_config: false,
        #[cfg(unix)]
        socket: None,
        tsconfig: None,
        format: "text".into(),
        show_virtual_ts: false,
        strict: false,
        no_strict: false,
        max_warnings: None,
        no_check_props: false,
        no_check_emits: false,
        no_check_template_bindings: false,
        no_check_reactivity: false,
        no_check_setup_context: false,
        no_check_invalid_exports: false,
        no_check_fallthrough_attrs: false,
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
        config: None,
        no_config: false,
        format: OutputFormat::Js,
        ssr: args.ssr,
        vapor: false,
        custom_renderer: false,
        script_ext: args.script_ext,
        threads: None,
        profile: false,
        slow_threshold: 100,
        continue_on_error: false,
    });
}

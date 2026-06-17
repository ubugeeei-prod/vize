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
    pub patterns: Vec<String>,

    /// Output directory for the build step.
    #[arg(short, long, default_value = "./dist")]
    pub output: PathBuf,

    /// Config file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Do not load a config file
    #[arg(long)]
    pub no_config: bool,

    /// Enable SSR mode for the build step.
    #[arg(long)]
    pub ssr: bool,

    /// Script extension handling for the build step.
    #[arg(long, value_enum, default_value = "downcompile")]
    pub script_ext: ScriptExtension,

    /// tsconfig.json path for the check step
    #[arg(long)]
    pub tsconfig: Option<PathBuf>,

    /// Disable prop type checks where supported
    #[arg(long)]
    pub no_check_props: bool,

    /// Disable emit type checks where supported
    #[arg(long)]
    pub no_check_emits: bool,

    /// Disable template binding checks where supported
    #[arg(long)]
    pub no_check_template_bindings: bool,
}

pub fn run(args: ReadyArgs) {
    if let Some(path) = args.config.as_deref()
        && !args.no_config
        && let Err(error) = crate::config::validate_explicit_config_path(path)
    {
        eprintln!("\x1b[31mError:\x1b[0m {}", error);
        std::process::exit(2);
    }

    let patterns = ready_patterns(&args.patterns);

    #[cfg(feature = "glyph")]
    {
        eprintln!("vize ready: fmt");
        crate::commands::fmt::run(FmtArgs {
            patterns: patterns.clone(),
            check: false,
            write: true,
            config: args.config.clone(),
            no_config: args.no_config,
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
        patterns: patterns
            .iter()
            .map(|pattern| pattern.to_compact_string())
            .collect(),
        fix: false,
        config: args.config.clone(),
        no_config: args.no_config,
        format: "text".into(),
        max_warnings: None,
        quiet: false,
        help_level: "full".into(),
        preset: "ecosystem".into(),
        cross_file: false,
        cross_file_tree: false,
        type_aware: false,
        strict_reactivity: false,
        profile: false,
        slow_threshold: 100,
    });

    eprintln!("vize ready: check");
    crate::commands::check::run(check_args(&args));

    eprintln!("vize ready: build");
    crate::commands::build::run(BuildArgs {
        patterns,
        output: args.output,
        config: args.config,
        no_config: args.no_config,
        format: OutputFormat::Js,
        ssr: args.ssr,
        vapor: false,
        custom_renderer: false,
        template_syntax: None,
        script_ext: args.script_ext,
        threads: None,
        profile: false,
        slow_threshold: 100,
        continue_on_error: false,
    });
}

#[allow(clippy::disallowed_types)]
fn ready_patterns(patterns: &[String]) -> Vec<String> {
    if patterns.is_empty() {
        vec!["./**/*.vue".into()]
    } else {
        patterns.to_vec()
    }
}

fn check_args(args: &ReadyArgs) -> CheckArgs {
    CheckArgs {
        patterns: check_patterns(args),
        config: args.config.clone(),
        no_config: args.no_config,
        #[cfg(unix)]
        socket: None,
        tsconfig: args.tsconfig.clone(),
        format: "text".into(),
        show_virtual_ts: false,
        save_virtual_ts_for: Vec::new(),
        max_warnings: None,
        no_check_props: args.no_check_props,
        no_check_emits: args.no_check_emits,
        no_check_template_bindings: args.no_check_template_bindings,
        quiet: false,
        profile: false,
        corsa_path: None,
        servers: None,
        declaration: false,
        declaration_dir: None,
    }
}

#[allow(clippy::disallowed_types)]
fn check_patterns(args: &ReadyArgs) -> Vec<String> {
    if args.patterns.is_empty()
        || args.tsconfig.is_some()
        || config_declares_type_checker_tsconfig(args)
    {
        Vec::new()
    } else {
        args.patterns.clone()
    }
}

fn config_declares_type_checker_tsconfig(args: &ReadyArgs) -> bool {
    if args.no_config {
        return false;
    }

    crate::config::load_config_with_features_and_source(args.config.as_deref())
        .config
        .type_checker
        .tsconfig
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready_args() -> ReadyArgs {
        ReadyArgs {
            patterns: vec!["src".into()],
            output: PathBuf::from("./dist"),
            config: None,
            no_config: false,
            ssr: false,
            script_ext: ScriptExtension::Downcompile,
            tsconfig: None,
            no_check_props: false,
            no_check_emits: false,
            no_check_template_bindings: false,
        }
    }

    #[test]
    fn check_args_forward_check_options() {
        let mut args = ready_args();
        args.tsconfig = Some(PathBuf::from("tsconfig.vize.json"));
        args.no_check_props = true;
        args.no_check_emits = true;
        args.no_check_template_bindings = true;

        let check = check_args(&args);

        assert!(check.patterns.is_empty());
        assert_eq!(check.tsconfig, Some(PathBuf::from("tsconfig.vize.json")));
        assert!(check.no_check_props);
        assert!(check.no_check_emits);
        assert!(check.no_check_template_bindings);
    }

    #[test]
    fn check_uses_project_inputs_when_ready_patterns_are_defaulted() {
        let mut args = ready_args();
        args.patterns.clear();

        assert_eq!(ready_patterns(&args.patterns), vec!["./**/*.vue"]);
        assert!(check_args(&args).patterns.is_empty());
    }

    #[test]
    fn check_keeps_explicit_patterns_without_tsconfig() {
        let mut args = ready_args();
        args.no_config = true;

        assert_eq!(check_args(&args).patterns, vec!["src"]);
    }

    #[test]
    fn check_uses_project_inputs_when_config_declares_tsconfig() {
        let project = tempfile::tempdir().unwrap();
        let config_path = project.path().join("vize.config.json");
        std::fs::write(
            &config_path,
            r#"{
  "typeChecker": {
    "tsconfig": "tsconfig.vize.json"
  }
}"#,
        )
        .unwrap();

        let mut args = ready_args();
        args.config = Some(config_path);

        assert!(check_args(&args).patterns.is_empty());
    }
}

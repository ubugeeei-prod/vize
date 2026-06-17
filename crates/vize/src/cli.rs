//! Shared CLI entrypoint for the native binary and Node binding.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vize")]
#[command(about = "High-performance Vue.js toolchain in Rust", long_about = None)]
#[command(version, disable_version_flag = true)]
struct Cli {
    /// Print version
    #[arg(short = 'v', short_alias = 'V', long, action = clap::ArgAction::Version)]
    version: (),

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile Vue SFC files (default command)
    #[command(visible_alias = "atelier")]
    Build(crate::commands::build::BuildArgs),

    /// Format Vue, JSX, and TSX files
    #[cfg(feature = "glyph")]
    #[command(visible_alias = "glyph")]
    Fmt(crate::commands::fmt::FmtArgs),

    /// Lint Vue, HTML, JSX, and TSX files
    #[command(visible_alias = "patina")]
    Lint(crate::commands::lint::LintArgs),

    /// Type check Vue, JSX, and TSX files
    Check(crate::commands::check::CheckArgs),

    /// Create playground compiler inspector payloads and agent reports
    Inspector(crate::commands::inspector::InspectorArgs),

    /// Curator utilities for diagnostics and reports
    Curator(crate::commands::curator::CuratorArgs),

    /// Remove Vize-generated cache artifacts
    Clean(crate::commands::clean::CleanArgs),

    /// Start type check JSON-RPC server (Unix only)
    #[cfg(unix)]
    CheckServer(crate::commands::check_server::CheckServerArgs),

    /// Start component gallery server
    Musea(crate::commands::musea::MuseaArgs),

    /// Start Language Server Protocol server
    #[cfg(feature = "maestro")]
    #[command(visible_alias = "maestro")]
    Lsp(crate::commands::lsp::LspArgs),

    /// IDE integration - LSP server and editor extension management
    #[cfg(feature = "maestro")]
    Ide(crate::commands::ide::IdeArgs),

    /// Update the installed Vize package
    #[command(visible_alias = "self-update")]
    Upgrade(crate::commands::upgrade::UpgradeArgs),

    /// Run fmt, lint, check, and build
    #[cfg(feature = "glyph")]
    Ready(crate::commands::ready::ReadyArgs),
}

pub fn run_from_env() {
    run(Cli::parse());
}

#[allow(clippy::disallowed_types)]
pub fn run_from_args(args: Vec<String>) {
    let args = std::iter::once("vize").chain(args.iter().map(String::as_str));
    run(Cli::parse_from(args));
}

fn run(cli: Cli) {
    match cli.command {
        Some(Commands::Build(args)) => crate::commands::build::run(args),
        #[cfg(feature = "glyph")]
        Some(Commands::Fmt(args)) => crate::commands::fmt::run(args),
        Some(Commands::Lint(args)) => crate::commands::lint::run(args),
        Some(Commands::Check(args)) => crate::commands::check::run(args),
        Some(Commands::Inspector(args)) => crate::commands::inspector::run(args),
        Some(Commands::Curator(args)) => crate::commands::curator::run(args),
        Some(Commands::Clean(args)) => crate::commands::clean::run(args),
        #[cfg(unix)]
        Some(Commands::CheckServer(args)) => crate::commands::check_server::run(args),
        Some(Commands::Musea(args)) => crate::commands::musea::run(args),
        #[cfg(feature = "maestro")]
        Some(Commands::Lsp(args)) => crate::commands::lsp::run(args),
        #[cfg(feature = "maestro")]
        Some(Commands::Ide(args)) => crate::commands::ide::run(args),
        Some(Commands::Upgrade(args)) => crate::commands::upgrade::run(args),
        #[cfg(feature = "glyph")]
        Some(Commands::Ready(args)) => crate::commands::ready::run(args),
        None => {
            crate::commands::build::run(crate::commands::build::BuildArgs::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::CommandFactory;

    #[test]
    fn long_help_snapshot() {
        let mut help = Vec::new();
        Cli::command().write_long_help(&mut help).unwrap();
        insta::assert_snapshot!("cli_long_help", normalize_help(help));
    }

    #[test]
    fn build_help_snapshot() {
        insta::assert_snapshot!("cli_build_help", command_help("build"));
    }

    #[cfg(feature = "glyph")]
    #[test]
    fn fmt_help_snapshot() {
        insta::assert_snapshot!("cli_fmt_help", command_help("fmt"));
    }

    #[test]
    fn check_help_snapshot() {
        insta::assert_snapshot!("cli_check_help", command_help("check"));
    }

    #[cfg(feature = "glyph")]
    #[test]
    fn ready_help_snapshot() {
        insta::assert_snapshot!("cli_ready_help", command_help("ready"));
    }

    #[test]
    fn clean_help_snapshot() {
        insta::assert_snapshot!("cli_clean_help", command_help("clean"));
    }

    #[test]
    fn inspector_help_snapshot() {
        insta::assert_snapshot!("cli_inspector_help", command_help("inspector"));
    }

    #[test]
    fn curator_help_snapshot() {
        insta::assert_snapshot!("cli_curator_help", command_help("curator"));
    }

    fn command_help(command_name: &str) -> String {
        let mut command = Cli::command();
        let subcommand = command.find_subcommand_mut(command_name).unwrap();
        let mut help = Vec::new();
        subcommand.write_long_help(&mut help).unwrap();
        normalize_help(help)
    }

    fn normalize_help(help: Vec<u8>) -> String {
        let help = String::from_utf8(help).unwrap();
        let mut normalized = String::with_capacity(help.len());
        for line in help.lines() {
            normalized.push_str(line.trim_end());
            normalized.push('\n');
        }
        normalized
    }
}

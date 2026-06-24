mod migrate;
mod new;
mod serve_plan;
mod setup;

use clap::{Args, Subcommand};
use migrate::MigrateArgs;
use new::NewArgs;
use serve_plan::create_serve_plan;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use vize_carton::{String, cstr};

#[derive(Args)]
pub struct MuseaArgs {
    #[command(subcommand)]
    pub command: Option<MuseaCommand>,

    #[command(flatten)]
    pub serve: ServeArgs,
}

#[derive(Subcommand)]
pub enum MuseaCommand {
    /// Start the component gallery server (default)
    Serve(ServeArgs),

    /// Create a new Musea art project
    New(NewArgs),

    /// Migrate Storybook CSF stories into Musea `.art.vue` files
    Migrate(MigrateArgs),
}

#[derive(Args, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::disallowed_types)]
pub struct ServeArgs {
    /// Shared Vize config file path
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Port to run the server on
    #[arg(short, long, default_value = "6006")]
    pub port: u16,

    /// Host to bind to
    #[arg(long, default_value = "localhost")]
    pub host: String,

    #[arg(short, long, hide = true)]
    pub stories: Option<PathBuf>,

    /// Open browser automatically
    #[arg(long)]
    pub open: bool,

    /// Fail instead of trying another port when the selected port is unavailable
    #[arg(long, visible_alias = "strictPort")]
    pub strict_port: bool,

    /// Run `vite build` instead of `vite dev`
    #[arg(long)]
    pub build: bool,
}

impl Default for ServeArgs {
    fn default() -> Self {
        Self {
            port: 6006,
            config: None,
            host: cstr!("localhost"),
            stories: None,
            open: false,
            strict_port: false,
            build: false,
        }
    }
}

pub fn run(args: MuseaArgs) {
    match args.command {
        Some(MuseaCommand::Serve(serve_args)) => run_serve(serve_args),
        Some(MuseaCommand::New(new_args)) => new::run(new_args),
        Some(MuseaCommand::Migrate(migrate_args)) => migrate::run(migrate_args),
        None => run_serve(args.serve),
    }
}

fn run_serve(args: ServeArgs) {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            eprintln!("vize musea: failed to read current directory: {}", error);
            std::process::exit(1);
        }
    };
    let plan = match create_serve_plan(&args, &cwd) {
        Ok(plan) => plan,
        Err(message) => {
            eprintln!("{}", message);
            std::process::exit(1);
        }
    };

    let action = if args.build { " build" } else { "" };
    eprintln!("vize musea: starting Vite-backed gallery{}...", action);
    eprintln!(
        "  command: {} {}",
        plan.program.display(),
        plan.args
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    );
    if args.build {
        eprintln!("  output: Musea static gallery entry is emitted under /__musea__/");
    } else {
        eprintln!("  route: configure @vizejs/vite-plugin-musea in Vite and open /__musea__");
    }

    let status = Command::new(&plan.program)
        .args(plan.args.iter().map(|arg| arg.as_str()))
        .envs(
            plan.env
                .iter()
                .map(|item| (item.0.as_str(), item.1.as_str())),
        )
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    match status {
        Ok(status) => {
            std::process::exit(status.code().unwrap_or(1));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "vize musea: could not find Vite. Install vite and @vizejs/vite-plugin-musea, then run from your project root."
            );
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("vize musea: failed to start Vite: {}", error);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests;

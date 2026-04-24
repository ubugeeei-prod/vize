//! Upgrade command - Update the installed Vize CLI.

use clap::{Args, ValueEnum};
use std::process::Command;

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum UpgradeSource {
    /// Update the Rust CLI through Cargo.
    #[default]
    Cargo,
}

#[derive(Args)]
#[allow(clippy::disallowed_types)]
pub struct UpgradeArgs {
    /// Update source.
    #[arg(long, value_enum, default_value = "cargo")]
    pub source: UpgradeSource,

    /// Package to install.
    #[arg(long, default_value = "vize")]
    pub package: String,

    /// Skip `--locked` when running Cargo.
    #[arg(long)]
    pub no_locked: bool,

    /// Print the command without running it.
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: UpgradeArgs) {
    match args.source {
        UpgradeSource::Cargo => run_cargo_upgrade(args),
    }
}

fn run_cargo_upgrade(args: UpgradeArgs) {
    let mut command_args = vec!["install", args.package.as_str(), "--force"];
    if !args.no_locked {
        command_args.push("--locked");
    }

    if args.dry_run {
        eprintln!("cargo {}", command_args.join(" "));
        return;
    }

    let status = Command::new("cargo")
        .args(&command_args)
        .status()
        .unwrap_or_else(|error| {
            eprintln!("Failed to start cargo: {}", error);
            std::process::exit(1);
        });

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}

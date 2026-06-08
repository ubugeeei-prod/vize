//! Curator command - diagnostics and reporting utilities.

use clap::{Args, Subcommand};

#[derive(Args)]
pub struct CuratorArgs {
    #[command(subcommand)]
    pub command: CuratorCommands,
}

#[derive(Subcommand)]
pub enum CuratorCommands {
    /// Print environment information for bug reports
    Env,
}

pub fn run(args: CuratorArgs) {
    match args.command {
        CuratorCommands::Env => crate::commands::env_info::run(),
    }
}

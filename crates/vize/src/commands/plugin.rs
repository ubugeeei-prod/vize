//! Plugin command - inspect the experimental plugin API v2 plan.

use std::path::PathBuf;

#[derive(clap::Args, Debug, Clone)]
pub struct PluginArgs {
    #[command(subcommand)]
    pub command: PluginCommand,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum PluginCommand {
    /// Print the canonical plugin plan resolved from vite.config.* as JSON
    Inspect(InspectArgs),
}

#[derive(clap::Args, Debug, Clone)]
pub struct InspectArgs {
    /// Vite config file or project directory (default: discovered from the current directory)
    #[arg(long)]
    pub config: Option<PathBuf>,
}

pub fn run(args: PluginArgs) {
    match args.command {
        PluginCommand::Inspect(args) => {
            match vize_carton::config::inspect_config_plugins(args.config.as_deref()) {
                Ok(inspection) => match serde_json::to_string_pretty(&inspection) {
                    Ok(json) => println!("{json}"),
                    Err(error) => fail(&vize_carton::cstr!("{error}")),
                },
                Err(error) => fail(&error),
            }
        }
    }
}

fn fail(message: &str) {
    eprintln!("{message}");
    std::process::exit(1);
}

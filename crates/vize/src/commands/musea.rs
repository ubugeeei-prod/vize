mod serve_plan;
mod setup;

use clap::{Args, Subcommand};
use serve_plan::create_serve_plan;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use vize_carton::{String, ToCompactString, cstr};

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

#[derive(Args)]
#[allow(clippy::disallowed_types)]
pub struct NewArgs {
    /// Name of the Musea project (defaults to current directory name)
    pub name: Option<String>,

    /// Directory to create the project in (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<PathBuf>,
}

pub fn run(args: MuseaArgs) {
    match args.command {
        Some(MuseaCommand::Serve(serve_args)) => run_serve(serve_args),
        Some(MuseaCommand::New(new_args)) => run_new(new_args),
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

fn run_new(args: NewArgs) {
    let target_dir = args.path.unwrap_or_else(|| PathBuf::from("."));
    #[allow(clippy::disallowed_types, clippy::disallowed_methods)]
    let project_name = args.name.unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| {
                p.file_name()
                    .map(|name| name.to_string_lossy().as_ref().to_compact_string())
            })
            .unwrap_or_else(|| cstr!("stories"))
    });

    eprintln!(
        "vize musea new: Creating Musea project '{}'...",
        project_name
    );

    let stories_dir = target_dir.join("stories");
    if let Err(e) = fs::create_dir_all(&stories_dir) {
        eprintln!("vize musea new: failed to create stories directory: {}", e);
        std::process::exit(1);
    }

    let example_story = stories_dir.join("Button.art.vue");
    let example_content = r#"<script setup lang="ts">
defineArt("../src/Button.vue", {
  title: "Button",
  category: "Components",
  tags: ["button", "ui"],
});
</script>

<art>
  <variant name="Primary" default>
    <Button variant="primary">Click me</Button>
  </variant>

  <variant name="Secondary">
    <Button variant="secondary">Click me</Button>
  </variant>

  <variant name="Disabled">
    <Button variant="primary" disabled>Disabled</Button>
  </variant>
</art>

<style scoped>
.art-preview {
  padding: 0.5rem 1rem;
  display: flex;
  gap: 0.75rem;
  align-items: center;
}
</style>
"#;

    if let Err(e) = fs::write(&example_story, example_content) {
        eprintln!("vize musea new: failed to create example story: {}", e);
        std::process::exit(1);
    }

    let config_path = target_dir.join("vize.config.ts");
    if !config_path.exists() {
        let config_content = r#"import { defineConfig } from "vize";

export default defineConfig({
  musea: {
    include: ["./stories/**/*.art.vue"],
  },
});
"#;
        if let Err(e) = fs::write(&config_path, config_content) {
            eprintln!("vize musea new: failed to create vize.config.ts: {}", e);
            std::process::exit(1);
        }
        eprintln!("  Created vize.config.ts");
    }

    eprintln!("  Created stories/Button.art.vue");
    eprintln!();
    eprintln!("Musea project '{}' created successfully!", project_name);
    eprintln!();
    eprintln!("Next steps:");
    eprintln!("  1. Add more art files in the 'stories' directory");
    eprintln!("  2. Enable @vizejs/vite-plugin-musea in your Vite or Nuxt project");
}

#[cfg(test)]
mod tests;

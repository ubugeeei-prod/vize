//! Musea command - Component gallery server

use clap::{Args, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Args)]
pub struct MuseaArgs {
    #[command(subcommand)]
    pub command: Option<MuseaCommand>,
}

#[derive(Subcommand)]
pub enum MuseaCommand {
    /// Start the component gallery server (default)
    Serve(ServeArgs),

    /// Create a new Musea art project
    New(NewArgs),
}

#[derive(Args, Default)]
#[allow(clippy::disallowed_types)]
pub struct ServeArgs {
    /// Port to run the server on
    #[arg(short, long, default_value = "6006")]
    pub port: u16,

    /// Host to bind to
    #[arg(long, default_value = "localhost")]
    pub host: String,

    /// Stories directory
    #[arg(short, long)]
    pub stories: Option<PathBuf>,

    /// Open browser automatically
    #[arg(long)]
    pub open: bool,
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
        None => {
            // Default to serve
            run_serve(ServeArgs::default());
        }
    }
}

fn run_serve(args: ServeArgs) {
    eprintln!("vize musea: Starting component gallery...");
    eprintln!("  host: {}", args.host);
    eprintln!("  port: {}", args.port);
    eprintln!("  open: {}", args.open);

    vize_musea::serve();
}

fn run_new(args: NewArgs) {
    let target_dir = args.path.unwrap_or_else(|| PathBuf::from("."));
    #[allow(clippy::disallowed_types, clippy::disallowed_methods)]
    let project_name = args.name.unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "stories".to_string())
    });

    eprintln!(
        "vize musea new: Creating Musea project '{}'...",
        project_name
    );

    // Create art directory structure
    let stories_dir = target_dir.join("stories");
    if let Err(e) = fs::create_dir_all(&stories_dir) {
        eprintln!("Error creating stories directory: {}", e);
        std::process::exit(1);
    }

    // Create example art file
    let example_story = stories_dir.join("Button.art.vue");
    let example_content = r#"<script setup lang="ts">
import Button from '../src/Button.vue'
</script>

<art title="Button" component="../src/Button.vue" category="Components" tags="button, ui">
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
        eprintln!("Error creating example story: {}", e);
        std::process::exit(1);
    }

    // Create vize.config.ts
    let config_path = target_dir.join("vize.config.ts");
    if !config_path.exists() {
        let config_content = r#"import { defineConfig } from 'vize'

export default defineConfig({
  musea: {
    include: ['./stories/**/*.art.vue'],
  },
})
"#;
        if let Err(e) = fs::write(&config_path, config_content) {
            eprintln!("Error creating vize.config.ts: {}", e);
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

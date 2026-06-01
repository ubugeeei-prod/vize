//! Musea command - Component gallery server

use clap::{Args, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
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

    /// Run `vite build` instead of `vite dev`
    #[arg(long)]
    pub build: bool,
}

impl Default for ServeArgs {
    fn default() -> Self {
        Self {
            port: 6006,
            host: cstr!("localhost"),
            stories: None,
            open: false,
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

    eprintln!("vize musea: starting Vite-backed component gallery...");
    eprintln!(
        "  command: {} {}",
        plan.program.display(),
        plan.args
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    );
    eprintln!("  route: configure @vizejs/vite-plugin-musea in Vite and open /__musea__");

    let status = Command::new(&plan.program)
        .args(plan.args.iter().map(|arg| arg.as_str()))
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

#[derive(Debug, PartialEq, Eq)]
struct ServePlan {
    program: PathBuf,
    args: Vec<String>,
}

fn create_serve_plan(args: &ServeArgs, cwd: &Path) -> Result<ServePlan, String> {
    if let Some(stories) = &args.stories {
        return Err(cstr!(
            "vize musea: --stories is not supported by the Vite-backed serve entrypoint yet (got {}). Configure Musea include patterns in vize.config.ts instead.",
            stories.display()
        ));
    }

    let program = resolve_vite_binary(cwd).unwrap_or_else(|| PathBuf::from("vite"));
    let mut vite_args = if args.build {
        vec![cstr!("build")]
    } else {
        vec![
            cstr!("dev"),
            cstr!("--host"),
            args.host.clone(),
            cstr!("--port"),
            args.port.to_compact_string(),
        ]
    };
    if args.open && !args.build {
        vite_args.push(cstr!("--open"));
        vite_args.push(cstr!("/__musea__"));
    }

    Ok(ServePlan {
        program,
        args: vite_args,
    })
}

fn resolve_vite_binary(cwd: &Path) -> Option<PathBuf> {
    for ancestor in cwd.ancestors() {
        let bin_dir = ancestor.join("node_modules").join(".bin");
        for name in vite_bin_names() {
            let candidate = bin_dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

#[cfg(windows)]
fn vite_bin_names() -> &'static [&'static str] {
    &["vite.cmd", "vite.ps1", "vite"]
}

#[cfg(not(windows))]
fn vite_bin_names() -> &'static [&'static str] {
    &["vite"]
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

    // Create art directory structure
    let stories_dir = target_dir.join("stories");
    if let Err(e) = fs::create_dir_all(&stories_dir) {
        eprintln!("vize musea new: failed to create stories directory: {}", e);
        std::process::exit(1);
    }

    // Create example art file
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

    // Create vize.config.ts
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
mod tests {
    use super::{ServeArgs, create_serve_plan, resolve_vite_binary, vite_bin_names};
    use std::fs;
    use std::path::{Path, PathBuf};

    fn write_vite_bin(root: &Path) -> PathBuf {
        let bin_dir = root.join("node_modules").join(".bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let vite_bin = bin_dir.join(vite_bin_names()[0]);
        fs::write(&vite_bin, "").unwrap();
        vite_bin
    }

    #[test]
    fn resolves_vite_binary_from_project_ancestors() {
        let temp = tempfile::tempdir().unwrap();
        let vite_bin = write_vite_bin(temp.path());
        let nested = temp.path().join("packages").join("app");
        fs::create_dir_all(&nested).unwrap();

        assert_eq!(resolve_vite_binary(&nested), Some(vite_bin));
    }

    #[test]
    fn serve_plan_defaults_to_vite_dev_with_gallery_route() {
        let temp = tempfile::tempdir().unwrap();
        let vite_bin = write_vite_bin(temp.path());

        let plan = create_serve_plan(
            &ServeArgs {
                open: true,
                ..ServeArgs::default()
            },
            temp.path(),
        )
        .unwrap();

        assert_eq!(plan.program, vite_bin);
        assert_eq!(
            plan.args,
            [
                "dev",
                "--host",
                "localhost",
                "--port",
                "6006",
                "--open",
                "/__musea__"
            ]
        );
    }

    #[test]
    fn serve_plan_supports_vite_build() {
        let temp = tempfile::tempdir().unwrap();
        let vite_bin = write_vite_bin(temp.path());

        let plan = create_serve_plan(
            &ServeArgs {
                build: true,
                open: true,
                ..ServeArgs::default()
            },
            temp.path(),
        )
        .unwrap();

        assert_eq!(plan.program, vite_bin);
        assert_eq!(plan.args, ["build"]);
    }

    #[test]
    fn serve_plan_rejects_silent_stories_option() {
        let temp = tempfile::tempdir().unwrap();
        write_vite_bin(temp.path());

        let error = create_serve_plan(
            &ServeArgs {
                stories: Some(PathBuf::from("stories")),
                ..ServeArgs::default()
            },
            temp.path(),
        )
        .unwrap_err();

        assert!(error.contains("--stories is not supported"));
    }
}

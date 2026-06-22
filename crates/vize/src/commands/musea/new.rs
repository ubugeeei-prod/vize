use clap::Args;
use std::fs;
use std::path::PathBuf;
use vize_carton::{String, ToCompactString, cstr};

#[derive(Args)]
#[allow(clippy::disallowed_types)]
pub struct NewArgs {
    /// Name of the Musea project (defaults to current directory name)
    pub name: Option<String>,

    /// Directory to create the project in (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<PathBuf>,
}

pub fn run(args: NewArgs) {
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

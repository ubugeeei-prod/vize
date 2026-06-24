//! `vize musea migrate` - convert Storybook CSF stories to Musea `.art.vue`.

#![allow(clippy::disallowed_macros)]

mod csf;
mod emit;
mod jsx;
mod text;

use clap::Args;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::fs;
use std::path::{Path, PathBuf};
use vize_carton::String as CartonString;
use vize_carton::append;

use crate::commands::fmt::collect_files;
use csf::extract_csf;
use emit::emit_art;

#[derive(Args)]
#[allow(clippy::disallowed_types)]
pub struct MigrateArgs {
    /// Glob pattern(s) matching Storybook CSF story files
    #[arg(default_values_t = default_migrate_patterns())]
    pub patterns: Vec<std::string::String>,

    /// Directory to write `.art.vue` files into (default: alongside each source)
    #[arg(long, value_name = "DIR")]
    pub out_dir: Option<PathBuf>,

    /// Print generated content to stdout instead of writing files
    #[arg(long)]
    pub dry_run: bool,
}

#[allow(clippy::disallowed_types)]
fn default_migrate_patterns() -> Vec<std::string::String> {
    vec![
        "**/*.stories.tsx".into(),
        "**/*.stories.ts".into(),
        "**/*.stories.jsx".into(),
        "**/*.stories.js".into(),
    ]
}

#[derive(Default)]
struct Summary {
    files: usize,
    variants: usize,
    todos: usize,
    errors: usize,
}

pub fn run(args: MigrateArgs) {
    let files = collect_story_files(&args.patterns);
    if files.is_empty() {
        eprintln!("vize musea migrate: no story files matched the patterns");
        return;
    }

    let mut summary = Summary::default();
    for path in &files {
        match migrate_file(path, args.out_dir.as_deref(), args.dry_run) {
            Ok(Some(outcome)) => {
                summary.files += 1;
                summary.variants += outcome.variants;
                summary.todos += outcome.todos;
            }
            Ok(None) => {}
            Err(message) => {
                summary.errors += 1;
                eprintln!("vize musea migrate: {}: {}", path.display(), message);
            }
        }
    }

    eprintln!(
        "vize musea migrate: {} file(s) migrated, {} variant(s) generated, {} TODO fallback(s)",
        summary.files, summary.variants, summary.todos
    );
    if summary.errors > 0 {
        eprintln!("  {} file(s) could not be migrated", summary.errors);
    }
}

#[allow(clippy::disallowed_types)]
fn collect_story_files(patterns: &[std::string::String]) -> Vec<PathBuf> {
    collect_files(patterns, None)
        .into_iter()
        .filter(|path| is_story_file(path))
        .collect()
}

fn is_story_file(path: &Path) -> bool {
    let name = match path.file_name().and_then(|name| name.to_str()) {
        Some(name) => name,
        None => return false,
    };
    matches!(
        name.rsplit_once('.'),
        Some((stem, "tsx" | "ts" | "jsx" | "js")) if stem.ends_with(".stories")
    )
}

struct FileOutcome {
    variants: usize,
    todos: usize,
}

fn migrate_file(
    path: &Path,
    out_dir: Option<&Path>,
    dry_run: bool,
) -> Result<Option<FileOutcome>, CartonString> {
    let source = fs::read_to_string(path).map_err(|error| {
        let mut message = CartonString::from("failed to read file: ");
        append!(message, "{error}");
        message
    })?;

    let source_type = source_type_for_path(path);
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, source_type).parse();
    if parsed.panicked {
        return Err("failed to parse story file".into());
    }

    let module = extract_csf(&parsed.program);

    let component_path = module
        .component_path
        .clone()
        .unwrap_or_else(|| derive_component_path(path));
    let component_tag = component_tag_from_path(&component_path);

    let result = emit_art(&module, &component_tag, &component_path, &source);

    let target = output_path(path, out_dir);

    if dry_run {
        println!("// {}", target.display());
        print!("{}", result.content);
    } else {
        if let Some(parent) = target.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent).map_err(|error| {
                let mut message = CartonString::from("failed to create output directory: ");
                append!(message, "{error}");
                message
            })?;
        }
        fs::write(&target, result.content.as_str()).map_err(|error| {
            let mut message = CartonString::from("failed to write file: ");
            append!(message, "{error}");
            message
        })?;
        eprintln!("vize musea migrate: wrote {}", target.display());
    }

    Ok(Some(FileOutcome {
        variants: result.variants,
        todos: result.todos,
    }))
}

fn source_type_for_path(path: &Path) -> SourceType {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("tsx") => SourceType::tsx(),
        Some("jsx") => SourceType::jsx(),
        Some("ts") => SourceType::ts(),
        _ => SourceType::mjs(),
    }
}

/// Strip the `.stories.<ext>` suffix to get the output basename.
fn story_basename(path: &Path) -> CartonString {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Component");
    let stem = name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(name);
    let base = stem.strip_suffix(".stories").unwrap_or(stem);
    base.into()
}

fn output_path(path: &Path, out_dir: Option<&Path>) -> PathBuf {
    let mut file_name = story_basename(path);
    file_name.push_str(".art.vue");
    let dir = out_dir
        .map(Path::to_path_buf)
        .or_else(|| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));
    dir.join(file_name.as_str())
}

/// Fallback component import path when the CSF has no resolvable component.
fn derive_component_path(path: &Path) -> CartonString {
    let mut out = CartonString::from("./");
    out.push_str(story_basename(path).as_str());
    out.push_str(".vue");
    out
}

/// Derive the component element tag (PascalCase file stem) from an import path.
fn component_tag_from_path(component_path: &str) -> CartonString {
    let file = component_path
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(component_path);
    let stem = file.split(['?', '#']).next().unwrap_or(file);
    let stem = stem.strip_suffix(".vue").unwrap_or(stem);
    let stem = stem.rsplit_once('.').map(|(base, _)| base).unwrap_or(stem);
    if stem.is_empty() {
        return "Component".into();
    }
    stem.into()
}

#[cfg(test)]
mod tests;

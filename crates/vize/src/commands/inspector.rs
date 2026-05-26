//! Compiler inspector payload generation.
//!
//! The command does not run the JavaScript reference compiler itself. It packages
//! one or more Vue SFC sources into the same payload shape consumed by the
//! playground inspector, where the browser can compare @vue/compiler-sfc and
//! Vize WASM output.

use clap::{Args, ValueEnum};
use ignore::Walk;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
use vize_carton::ToCompactString;
use vize_curator::inspector as curator_inspector;

#[derive(Debug, Clone, Copy, ValueEnum, Default, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InspectorOutputFormat {
    /// Print a playground URL containing the encoded inspector payload
    #[default]
    Url,
    /// Print the raw inspector JSON payload
    Json,
    /// Print an AI-agent friendly JSON report with payload, URL, and graph metadata
    Agent,
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum InspectorTarget {
    /// Compare DOM compiler output
    #[default]
    Dom,
    /// Compare SSR compiler output
    Ssr,
}

impl From<InspectorTarget> for curator_inspector::InspectorTarget {
    fn from(target: InspectorTarget) -> Self {
        match target {
            InspectorTarget::Dom => Self::Dom,
            InspectorTarget::Ssr => Self::Ssr,
        }
    }
}

#[derive(Args, Default)]
#[allow(clippy::disallowed_types)]
pub struct InspectorArgs {
    /// File, directory, or glob pattern(s) to include (default: ./**/*.vue)
    #[arg(default_value = "./**/*.vue")]
    pub patterns: Vec<String>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "url")]
    pub format: InspectorOutputFormat,

    /// Playground URL used when --format url is selected
    #[arg(long, default_value = "https://vizejs.dev/play/")]
    pub playground_url: String,

    /// Compiler target to compare
    #[arg(long, value_enum, default_value = "dom")]
    pub target: InspectorTarget,

    /// Write output to a file instead of stdout
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Limit the number of files in the payload
    #[arg(long)]
    pub max_files: Option<usize>,

    /// Enable custom renderer comparison in the playground
    #[arg(long)]
    pub custom_renderer: bool,

    /// Enable Vue parser compatibility quirks in the Vize side of the comparison
    #[arg(long)]
    pub vue_parser_quirks: bool,
}

pub fn run(args: InspectorArgs) {
    let files = collect_files(&args.patterns, args.max_files);
    if files.is_empty() {
        eprintln!("No .vue files found matching the patterns");
        std::process::exit(1);
    }

    let source_files = collect_source_files(&files);
    let payload = build_payload(&args, source_files.clone());
    let json = curator_inspector::serialize_payload(&payload).unwrap_or_else(|error| {
        eprintln!("Failed to serialize inspector payload: {error}");
        std::process::exit(1);
    });
    let playground_url =
        curator_inspector::build_playground_url(&args.playground_url, json.as_str());
    let output = match args.format {
        InspectorOutputFormat::Json => json,
        InspectorOutputFormat::Url => {
            if playground_url.len() > 7000 {
                eprintln!(
                    "Inspector URL is {} bytes; use --format json for large batches if the browser rejects it.",
                    playground_url.len()
                );
            }
            playground_url
        }
        InspectorOutputFormat::Agent => {
            let report =
                curator_inspector::build_agent_report(payload, playground_url, source_files);
            curator_inspector::serialize_agent_report(&report).unwrap_or_else(|error| {
                eprintln!("Failed to serialize inspector agent report: {error}");
                std::process::exit(1);
            })
        }
    };

    if let Some(output_path) = args.output {
        if let Err(error) = fs::write(&output_path, output.as_str()) {
            eprintln!("Failed to write {}: {error}", output_path.display());
            std::process::exit(1);
        }
    } else {
        println!("{output}");
    }
}

#[allow(clippy::disallowed_types)]
fn collect_files(patterns: &[String], max_files: Option<usize>) -> Vec<PathBuf> {
    let mut files = BTreeSet::new();

    for pattern in patterns {
        let path = Path::new(pattern.as_str());
        if path.is_file() {
            if is_vue_file(path) {
                files.insert(path.to_path_buf());
            }
            continue;
        }

        if path.is_dir() {
            for entry in Walk::new(path).flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() && is_vue_file(entry_path) {
                    files.insert(entry_path.to_path_buf());
                    if max_files.is_some_and(|limit| files.len() >= limit) {
                        return files.into_iter().collect();
                    }
                }
            }
            continue;
        }

        match glob::glob(pattern.as_str()) {
            Ok(paths) => {
                for path in paths.flatten() {
                    if path.is_file() && is_vue_file(&path) {
                        files.insert(path);
                        if max_files.is_some_and(|limit| files.len() >= limit) {
                            return files.into_iter().collect();
                        }
                    }
                }
            }
            Err(error) => {
                eprintln!("Invalid glob pattern {pattern}: {error}");
                std::process::exit(1);
            }
        }
    }

    let mut files: Vec<_> = files.into_iter().collect();
    if let Some(limit) = max_files {
        files.truncate(limit);
    }
    files
}

fn is_vue_file(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "vue")
}

fn collect_source_files(files: &[PathBuf]) -> Vec<curator_inspector::InspectorSourceFile> {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    files
        .iter()
        .map(|path| {
            let source = fs::read_to_string(path).unwrap_or_else(|error| {
                eprintln!("Failed to read {}: {error}", path.display());
                std::process::exit(1);
            });
            let display_path = path
                .strip_prefix(&current_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/")
                .to_compact_string();

            curator_inspector::InspectorSourceFile {
                path: display_path,
                source: source.to_compact_string(),
            }
        })
        .collect()
}

fn build_payload(
    args: &InspectorArgs,
    files: Vec<curator_inspector::InspectorSourceFile>,
) -> curator_inspector::InspectorPayload {
    curator_inspector::build_payload(
        args.target.into(),
        curator_inspector::InspectorOptions {
            custom_renderer: args.custom_renderer,
            vue_parser_quirks: args.vue_parser_quirks,
        },
        files,
    )
}

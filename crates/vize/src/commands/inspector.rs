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
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};
use vize_carton::{String, ToCompactString};

#[derive(Debug, Clone, Copy, ValueEnum, Default, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InspectorOutputFormat {
    /// Print a playground URL containing the encoded inspector payload
    #[default]
    Url,
    /// Print the raw inspector JSON payload
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum, Default, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InspectorTarget {
    /// Compare DOM compiler output
    #[default]
    Dom,
    /// Compare SSR compiler output
    Ssr,
}

impl InspectorTarget {
    fn as_payload_str(self) -> &'static str {
        match self {
            Self::Dom => "dom",
            Self::Ssr => "ssr",
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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InspectorPayload {
    version: u8,
    target: &'static str,
    selected_file: Option<String>,
    options: InspectorPayloadOptions,
    files: Vec<InspectorPayloadFile>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InspectorPayloadOptions {
    custom_renderer: bool,
    vue_parser_quirks: bool,
}

#[derive(serde::Serialize)]
struct InspectorPayloadFile {
    path: String,
    source: String,
}

pub fn run(args: InspectorArgs) {
    let files = collect_files(&args.patterns, args.max_files);
    if files.is_empty() {
        eprintln!("No .vue files found matching the patterns");
        std::process::exit(1);
    }

    let payload = build_payload(&args, &files);
    let json = serde_json::to_string(&payload).unwrap_or_else(|error| {
        eprintln!("Failed to serialize inspector payload: {error}");
        std::process::exit(1);
    });
    let output = match args.format {
        InspectorOutputFormat::Json => json.to_compact_string(),
        InspectorOutputFormat::Url => {
            let url = build_playground_url(&args.playground_url, &json);
            if url.len() > 7000 {
                eprintln!(
                    "Inspector URL is {} bytes; use --format json for large batches if the browser rejects it.",
                    url.len()
                );
            }
            url
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

fn build_payload(args: &InspectorArgs, files: &[PathBuf]) -> InspectorPayload {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let files: Vec<_> = files
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

            InspectorPayloadFile {
                path: display_path,
                source: source.to_compact_string(),
            }
        })
        .collect();

    let selected_file = files.first().map(|file| file.path.clone());

    InspectorPayload {
        version: 1,
        target: args.target.as_payload_str(),
        selected_file,
        options: InspectorPayloadOptions {
            custom_renderer: args.custom_renderer,
            vue_parser_quirks: args.vue_parser_quirks,
        },
        files,
    }
}

fn build_playground_url(base: &str, payload_json: &str) -> String {
    let base_without_hash = base.split('#').next().unwrap_or(base);
    let separator = if base_without_hash.contains('?') {
        if base_without_hash.ends_with('?') || base_without_hash.ends_with('&') {
            ""
        } else {
            "&"
        }
    } else {
        "?"
    };

    let mut url = String::default();
    url.push_str(base_without_hash);
    url.push_str(separator);
    url.push_str("tab=inspector#inspector=");
    url.push_str(percent_encode(payload_json).as_str());
    url
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::default();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                let _ = write!(encoded, "%{byte:02X}");
            }
        }
    }
    encoded
}

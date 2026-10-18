//! Compiler inspector payload generation.
//!
//! The default formats package one or more Vue SFC sources into the same
//! payload shape consumed by the playground inspector. The development-only
//! compare format also runs @vue/compiler-sfc through local Node.js.

use clap::{Args, ValueEnum};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use vize_atelier_core::TemplateSyntaxMode;
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, StyleCompileOptions,
    TemplateCompileOptions, compile_sfc_with_template_syntax as compile_vize_sfc,
    parse_sfc as parse_vize_sfc,
};
use vize_curator::inspector as curator_inspector;
use vize_l0::{String, ToCompactString};

mod compare;
mod compare_error;

use compare::{VizeCompilerRun, build_compare_report};
mod input_files;
mod no_inputs;
use input_files::collect_files;

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
    /// Run a development-only CLI comparison against @vue/compiler-sfc
    Compare,
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum InspectorTarget {
    /// Compare DOM compiler output
    #[default]
    Dom,
    /// Compare SSR compiler output
    Ssr,
    /// Compare Vapor compiler output
    Vapor,
}

impl From<InspectorTarget> for curator_inspector::InspectorTarget {
    fn from(target: InspectorTarget) -> Self {
        match target {
            InspectorTarget::Dom => Self::Dom,
            InspectorTarget::Ssr => Self::Ssr,
            InspectorTarget::Vapor => Self::Vapor,
        }
    }
}

impl InspectorTarget {
    fn as_str(self) -> &'static str {
        match self {
            Self::Dom => "dom",
            Self::Ssr => "ssr",
            Self::Vapor => "vapor",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Default, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InspectorTemplateSyntaxArg {
    /// Warn and rewrite recoverable invalid template syntax.
    #[default]
    Standard,
    /// Report recoverable invalid template syntax as errors.
    Strict,
    /// Preserve parser compatibility quirks without warnings.
    Quirks,
}

impl From<InspectorTemplateSyntaxArg> for TemplateSyntaxMode {
    fn from(value: InspectorTemplateSyntaxArg) -> Self {
        match value {
            InspectorTemplateSyntaxArg::Standard => Self::Standard,
            InspectorTemplateSyntaxArg::Strict => Self::Strict,
            InspectorTemplateSyntaxArg::Quirks => Self::Quirks,
        }
    }
}

impl From<InspectorTemplateSyntaxArg> for curator_inspector::InspectorTemplateSyntax {
    fn from(value: InspectorTemplateSyntaxArg) -> Self {
        match value {
            InspectorTemplateSyntaxArg::Standard => Self::Standard,
            InspectorTemplateSyntaxArg::Strict => Self::Strict,
            InspectorTemplateSyntaxArg::Quirks => Self::Quirks,
        }
    }
}

#[derive(Args, Default)]
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

    /// Template syntax compatibility mode
    #[arg(long, value_enum, default_value = "standard")]
    pub template_syntax: InspectorTemplateSyntaxArg,
}

/// Build and print an inspector payload after collecting source files once.
///
/// `collect_files` intentionally honors ignore files before the expensive SFC
/// parse/compile work starts. Inspector payload generation is often pointed at
/// repository roots, so letting ignored benchmark output or dependency mirrors
/// into the payload would dominate runtime and produce URLs too large to use.
pub fn run(args: InspectorArgs) {
    let files = collect_files(&args.patterns, args.max_files);
    if files.is_empty() {
        no_inputs::handle(args.format);
    }

    let source_files = collect_source_files(&files);
    let payload = build_payload(&args, source_files.clone());
    let json = curator_inspector::serialize_payload(&payload).unwrap_or_else(|error| {
        eprintln!("Failed to serialize inspector payload: {error}");
        std::process::exit(1);
    });
    let playground_url =
        curator_inspector::build_playground_url(&args.playground_url, json.as_str());
    let output: String = match args.format {
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
        InspectorOutputFormat::Compare => {
            let report = build_compare_report(&args, &source_files);
            serde_json::to_string_pretty(&report)
                .unwrap_or_else(|error| {
                    eprintln!("Failed to serialize inspector compare report: {error}");
                    std::process::exit(1);
                })
                .to_compact_string()
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

fn compile_vize_for_compare(
    file: &curator_inspector::InspectorSourceFile,
    args: &InspectorArgs,
) -> VizeCompilerRun {
    let start = Instant::now();
    let filename = file.path.to_compact_string();
    let descriptor = match parse_vize_sfc(
        &file.source,
        SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
    ) {
        Ok(descriptor) => descriptor,
        Err(error) => {
            return VizeCompilerRun {
                code: String::default(),
                warnings: Vec::new(),
                error: Some(error.message),
                time_ms: elapsed_ms(start),
            };
        }
    };

    let has_scoped = descriptor.styles.iter().any(|style| style.scoped);
    let is_ts = descriptor_uses_type_script(&descriptor);
    let is_ssr = matches!(args.target, InspectorTarget::Ssr);
    let is_vapor = matches!(args.target, InspectorTarget::Vapor);
    let compile_options = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some(filename.clone()),
            is_ts,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            id: Some(filename.clone()),
            scoped: has_scoped,
            ssr: is_ssr,
            is_prod: true,
            is_ts,
            custom_renderer: args.custom_renderer,
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename,
            scoped: has_scoped,
            ..Default::default()
        },
        vapor: is_vapor,
        ..Default::default()
    };

    let result = compile_vize_sfc(&descriptor, compile_options, args.template_syntax.into());

    match result {
        Ok(result) => VizeCompilerRun {
            code: result.code,
            warnings: result
                .warnings
                .into_iter()
                .map(|warning| warning.message)
                .collect(),
            error: format_sfc_errors(result.errors),
            time_ms: elapsed_ms(start),
        },
        Err(error) => VizeCompilerRun {
            code: String::default(),
            warnings: Vec::new(),
            error: Some(error.message),
            time_ms: elapsed_ms(start),
        },
    }
}

fn descriptor_uses_type_script(descriptor: &vize_atelier_sfc::SfcDescriptor) -> bool {
    descriptor
        .script
        .as_ref()
        .and_then(|script| script.lang.as_deref())
        .is_some_and(is_type_script_lang)
        || descriptor
            .script_setup
            .as_ref()
            .and_then(|script| script.lang.as_deref())
            .is_some_and(is_type_script_lang)
}

fn is_type_script_lang(lang: &str) -> bool {
    matches!(lang, "ts" | "tsx")
}

fn format_sfc_errors(errors: Vec<vize_atelier_sfc::SfcError>) -> Option<String> {
    if errors.is_empty() {
        return None;
    }

    let mut message = String::default();
    for (index, error) in errors.into_iter().enumerate() {
        if index > 0 {
            message.push('\n');
        }
        message.push_str(&error.message);
    }
    Some(message)
}

fn dev_module_root() -> Option<String> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent()?.parent()?;
    [root.to_path_buf(), root.join("tests")]
        .into_iter()
        .find(|candidate| candidate.join("node_modules/vue/compiler-sfc").is_dir())
        .map(|candidate| candidate.to_string_lossy().as_ref().to_compact_string())
}
fn output_text<'a>(code: &'a str, formatted_code: &'a str, error: Option<&'a str>) -> &'a str {
    error.unwrap_or({
        if formatted_code.is_empty() {
            code
        } else {
            formatted_code
        }
    })
}

fn elapsed_ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
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
                .or_else(|_| path.strip_prefix("."))
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
            template_syntax: args.template_syntax.into(),
        },
        files,
    )
}

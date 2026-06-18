//! Compiler inspector payload generation.
//!
//! The default formats package one or more Vue SFC sources into the same
//! payload shape consumed by the playground inspector. The development-only
//! compare format also runs @vue/compiler-sfc through local Node.js.

use clap::{Args, ValueEnum};
use ignore::WalkBuilder;
use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};
use vize_atelier_core::TemplateSyntaxMode;
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, StyleCompileOptions,
    TemplateCompileOptions, compile_sfc_with_template_syntax as compile_vize_sfc,
    parse_sfc as parse_vize_sfc,
};
use vize_carton::{String, ToCompactString};
use vize_curator::inspector as curator_inspector;

mod compare_error;

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

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InspectorCompareReport {
    schema: &'static str,
    version: u8,
    target: &'static str,
    summary: InspectorCompareSummary,
    files: Vec<InspectorCompareFile>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InspectorCompareSummary {
    file_count: usize,
    changed_files: usize,
    additions: usize,
    removals: usize,
    official_errors: usize,
    vize_errors: usize,
    options: InspectorCompareOptions,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InspectorCompareOptions {
    custom_renderer: bool,
    template_syntax: InspectorTemplateSyntaxArg,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InspectorCompareFile {
    path: String,
    parser: String,
    changed: bool,
    stats: curator_inspector::InspectorDiffStats,
    official: InspectorCompareCompilerRun,
    vize: InspectorCompareCompilerRun,
    diff: Vec<curator_inspector::InspectorDiffLine>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InspectorCompareCompilerRun {
    code: String,
    formatted_code: String,
    warnings: Vec<String>,
    error: Option<String>,
    time_ms: f64,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OfficialCompareInput {
    target: &'static str,
    module_root: Option<String>,
    files: Vec<OfficialCompareInputFile>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OfficialCompareInputFile {
    path: String,
    source: String,
    vize_code: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct OfficialCompareOutput {
    files: Vec<OfficialCompareOutputFile>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct OfficialCompareOutputFile {
    path: String,
    parser: String,
    official: OfficialCompilerRun,
    vize_formatted_code: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct OfficialCompilerRun {
    code: String,
    formatted_code: String,
    warnings: Vec<String>,
    error: Option<String>,
    time_ms: f64,
}

struct VizeCompilerRun {
    code: String,
    warnings: Vec<String>,
    error: Option<String>,
    time_ms: f64,
}

fn build_compare_report(
    args: &InspectorArgs,
    files: &[curator_inspector::InspectorSourceFile],
) -> InspectorCompareReport {
    let vize_runs = files
        .iter()
        .map(|file| compile_vize_for_compare(file, args))
        .collect::<Vec<_>>();
    let official = run_official_compiler_for_compare(files, &vize_runs, args);
    if official.files.len() != files.len() {
        eprintln!(
            "Official compiler returned {} file(s) while comparing {} file(s)",
            official.files.len(),
            files.len()
        );
        std::process::exit(1);
    }

    let mut compare_files = Vec::with_capacity(files.len());
    for ((file, vize), official_file) in files.iter().zip(vize_runs).zip(official.files) {
        if official_file.path != file.path {
            eprintln!(
                "Official compiler returned {} while comparing {}",
                official_file.path, file.path
            );
            std::process::exit(1);
        }

        let official_text = output_text(
            &official_file.official.code,
            &official_file.official.formatted_code,
            official_file.official.error.as_deref(),
        );
        let vize_text = output_text(
            &vize.code,
            &official_file.vize_formatted_code,
            vize.error.as_deref(),
        );
        let diff = curator_inspector::build_diff(official_text, vize_text);
        let changed = diff.stats.additions > 0 || diff.stats.removals > 0;
        let changed_diff = diff
            .lines
            .into_iter()
            .filter(|line| line.kind != "same")
            .collect();

        compare_files.push(InspectorCompareFile {
            path: file.path.clone(),
            parser: official_file.parser,
            changed,
            stats: diff.stats,
            official: InspectorCompareCompilerRun {
                code: official_file.official.code,
                formatted_code: official_file.official.formatted_code,
                warnings: official_file.official.warnings,
                error: official_file.official.error,
                time_ms: official_file.official.time_ms,
            },
            vize: InspectorCompareCompilerRun {
                code: vize.code,
                formatted_code: official_file.vize_formatted_code,
                warnings: vize.warnings,
                error: vize.error,
                time_ms: vize.time_ms,
            },
            diff: changed_diff,
        });
    }

    let summary = InspectorCompareSummary {
        file_count: compare_files.len(),
        changed_files: compare_files.iter().filter(|file| file.changed).count(),
        additions: compare_files.iter().map(|file| file.stats.additions).sum(),
        removals: compare_files.iter().map(|file| file.stats.removals).sum(),
        official_errors: compare_files
            .iter()
            .filter(|file| file.official.error.is_some())
            .count(),
        vize_errors: compare_files
            .iter()
            .filter(|file| file.vize.error.is_some())
            .count(),
        options: InspectorCompareOptions {
            custom_renderer: args.custom_renderer,
            template_syntax: args.template_syntax,
        },
    };

    InspectorCompareReport {
        schema: "vize.inspector.compare",
        version: 1,
        target: args.target.as_str(),
        summary,
        files: compare_files,
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

fn run_official_compiler_for_compare(
    files: &[curator_inspector::InspectorSourceFile],
    vize_runs: &[VizeCompilerRun],
    args: &InspectorArgs,
) -> OfficialCompareOutput {
    let input = OfficialCompareInput {
        target: args.target.as_str(),
        module_root: dev_module_root(),
        files: files
            .iter()
            .zip(vize_runs)
            .map(|(file, vize)| OfficialCompareInputFile {
                path: file.path.clone(),
                source: file.source.clone(),
                vize_code: vize.code.clone(),
            })
            .collect(),
    };
    let input_json = serde_json::to_vec(&input).unwrap_or_else(|error| {
        eprintln!("Failed to serialize official compiler input: {error}");
        std::process::exit(1);
    });
    let node = std::env::var_os("VIZE_INSPECTOR_NODE").unwrap_or_else(|| "node".into());
    let mut child = Command::new(&node)
        .arg("--input-type=module")
        .arg("-e")
        .arg(OFFICIAL_COMPILER_NODE_SCRIPT)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| {
            let node = node.to_string_lossy();
            eprintln!(
                "--format compare requires a local Node.js runtime and @vue/compiler-sfc. Failed to start {node}: {error}"
            );
            std::process::exit(1);
        });

    child
        .stdin
        .take()
        .and_then(|mut stdin| stdin.write_all(&input_json).ok())
        .unwrap_or_else(|| {
            eprintln!("Failed to write inspector compare input to Node.js");
            std::process::exit(1);
        });

    let output = child.wait_with_output().unwrap_or_else(|error| {
        eprintln!("Failed to wait for official compiler process: {error}");
        std::process::exit(1);
    });

    if !output.status.success() {
        let stderr = std::str::from_utf8(&output.stderr).unwrap_or("<stderr is not valid UTF-8>");
        eprintln!("{}", compare_error::official_compiler_error_message(stderr));
        std::process::exit(output.status.code().unwrap_or(1));
    }

    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        let stdout = std::str::from_utf8(&output.stdout).unwrap_or("<stdout is not valid UTF-8>");
        eprintln!("Failed to parse official compiler output: {error}\n{stdout}");
        std::process::exit(1);
    })
}

fn dev_module_root() -> Option<String> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent()?.parent()?;
    root.join("node_modules")
        .is_dir()
        .then(|| root.to_string_lossy().as_ref().to_compact_string())
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

const OFFICIAL_COMPILER_NODE_SCRIPT: &str = r#"
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import path from "node:path";
import process from "node:process";

async function readStdin() {
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  return Buffer.concat(chunks).toString("utf8");
}

async function importFromRoots(specifier, roots) {
  for (const root of roots) {
    if (!root) continue;
    try {
      const require = createRequire(path.join(root, "package.json"));
      const resolved = require.resolve(specifier);
      return await import(pathToFileURL(resolved).href);
    } catch {
      // Try the next root, then the default resolver below.
    }
  }
  return await import(specifier);
}

function normalizeCompilerMessages(messages) {
  return (messages ?? []).map((message) => {
    if (message instanceof Error) return message.message;
    if (typeof message === "object" && message && "message" in message) {
      return String(message.message);
    }
    return String(message);
  });
}

function descriptorUsesTypeScript(descriptor) {
  const langs = [descriptor.script?.lang, descriptor.scriptSetup?.lang];
  return langs.some((lang) => lang === "ts" || lang === "tsx");
}

function toErrorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

async function loadFormatter(roots) {
  try {
    const [prettier, parserBabel, parserEstree, parserTypescript] = await Promise.all([
      importFromRoots("prettier/standalone", roots),
      importFromRoots("prettier/plugins/babel", roots),
      importFromRoots("prettier/plugins/estree", roots),
      importFromRoots("prettier/plugins/typescript", roots),
    ]);
    return { prettier, parserBabel, parserEstree, parserTypescript };
  } catch {
    return null;
  }
}

async function formatCode(source, parser, formatter) {
  if (!source || !formatter) return source ?? "";
  const plugins =
    parser === "typescript"
      ? [formatter.parserTypescript, formatter.parserEstree]
      : [formatter.parserBabel, formatter.parserEstree];
  try {
    return await formatter.prettier.format(source, {
      parser,
      plugins,
      semi: true,
      singleQuote: false,
      printWidth: 100,
    });
  } catch {
    return source;
  }
}

async function compileOfficialVue(file, target, compiler, formatter) {
  const start = performance.now();
  try {
    const officialTarget = target === "ssr" ? "ssr" : "dom";
    const parsed = compiler.parse(file.source, { filename: file.path });
    const descriptor = parsed.descriptor;
    const isTypeScript = descriptorUsesTypeScript(descriptor);
    const parser = isTypeScript ? "typescript" : "babel";
    const warnings = normalizeCompilerMessages(parsed.errors);
    let bindingMetadata = {};
    let scriptCode = "";
    const scoped = descriptor.styles.some((style) => style.scoped);
    const inlineTemplate = Boolean(
      descriptor.scriptSetup && descriptor.template && officialTarget === "dom",
    );

    if (descriptor.script || descriptor.scriptSetup) {
      const script = compiler.compileScript(descriptor, {
        id: file.path,
        inlineTemplate,
        isProd: true,
        templateOptions: inlineTemplate
          ? {
              filename: file.path,
              id: file.path,
              scoped,
              isProd: true,
              compilerOptions: {
                expressionPlugins: isTypeScript ? ["typescript"] : undefined,
              },
            }
          : undefined,
      });
      scriptCode = script.content;
      bindingMetadata = script.bindings;
    }

    let templateCode = "";
    if (descriptor.template && !inlineTemplate) {
      const template = compiler.compileTemplate({
        source: descriptor.template.content,
        filename: file.path,
        id: file.path,
        scoped,
        isProd: true,
        ssr: officialTarget === "ssr",
        compilerOptions: {
          bindingMetadata,
          expressionPlugins: isTypeScript ? ["typescript"] : undefined,
        },
      });
      templateCode = template.code;
      warnings.push(...normalizeCompilerMessages(template.errors));
      warnings.push(...normalizeCompilerMessages(template.tips));
    }

    const code = [scriptCode, templateCode].filter(Boolean).join("\n\n");
    return {
      code,
      formattedCode: await formatCode(code, parser, formatter),
      warnings,
      error: null,
      timeMs: performance.now() - start,
    };
  } catch (error) {
    return {
      code: "",
      formattedCode: "",
      warnings: [],
      error: toErrorMessage(error),
      timeMs: performance.now() - start,
    };
  }
}

function parserForFile(file, compiler) {
  try {
    const parsed = compiler.parse(file.source, { filename: file.path });
    return descriptorUsesTypeScript(parsed.descriptor) ? "typescript" : "babel";
  } catch {
    return "babel";
  }
}

const input = JSON.parse(await readStdin());
const roots = [process.cwd(), input.moduleRoot].filter(Boolean);
const compiler = await importFromRoots("vue/compiler-sfc", roots);
const formatter = await loadFormatter(roots);
const files = [];

for (const file of input.files) {
  const parser = parserForFile(file, compiler);
  const official = await compileOfficialVue(file, input.target, compiler, formatter);
  files.push({
    path: file.path,
    parser,
    official,
    vizeFormattedCode: await formatCode(file.vizeCode, parser, formatter),
  });
}

process.stdout.write(JSON.stringify({ files }));
"#;

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
            if collect_walked_vue_files(path, &mut files, max_files) {
                return files.into_iter().collect();
            }
            continue;
        }

        if let Some(root) = recursive_vue_glob_root(pattern)
            && root.exists()
        {
            if collect_walked_vue_files(&root, &mut files, max_files) {
                return files.into_iter().collect();
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

fn collect_walked_vue_files(
    root: &Path,
    files: &mut BTreeSet<PathBuf>,
    max_files: Option<usize>,
) -> bool {
    for entry in WalkBuilder::new(root).require_git(false).build().flatten() {
        let entry_path = entry.path();
        if entry_path.is_file() && is_vue_file(entry_path) {
            files.insert(entry_path.to_path_buf());
            if max_files.is_some_and(|limit| files.len() >= limit) {
                return true;
            }
        }
    }

    false
}

fn recursive_vue_glob_root(pattern: &str) -> Option<PathBuf> {
    let normalized = pattern.replace('\\', "/");
    if normalized == "**/*.vue" || normalized == "./**/*.vue" {
        return Some(PathBuf::from("."));
    }

    let root = normalized.strip_suffix("/**/*.vue")?;
    if root.is_empty() || contains_glob_meta(root) {
        return None;
    }

    Some(PathBuf::from(root))
}

fn contains_glob_meta(value: &str) -> bool {
    value
        .bytes()
        .any(|byte| matches!(byte, b'*' | b'?' | b'[' | b']' | b'{' | b'}'))
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

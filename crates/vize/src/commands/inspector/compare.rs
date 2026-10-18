use std::{
    io::Write,
    process::{Command, Stdio},
};

use super::{
    InspectorArgs, InspectorTemplateSyntaxArg, String, compare_error, compile_vize_for_compare,
    curator_inspector, dev_module_root, output_text,
};

const OFFICIAL_COMPILER_NODE_SCRIPT: &str = include_str!("official_compiler.ts");

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct InspectorCompareReport {
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

pub(super) struct VizeCompilerRun {
    pub(super) code: String,
    pub(super) warnings: Vec<String>,
    pub(super) error: Option<String>,
    pub(super) time_ms: f64,
}

pub(super) fn build_compare_report(
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

    let write_result = child
        .stdin
        .take()
        .ok_or_else(|| std::io::Error::other("the Node.js process has no piped stdin"))
        .and_then(|mut stdin| stdin.write_all(&input_json));

    let output = child.wait_with_output().unwrap_or_else(|error| {
        eprintln!("Failed to wait for official compiler process: {error}");
        std::process::exit(1);
    });

    if !output.status.success() {
        let stderr = std::str::from_utf8(&output.stderr).unwrap_or("<stderr is not valid UTF-8>");
        eprintln!("{}", compare_error::official_compiler_error_message(stderr));
        std::process::exit(output.status.code().unwrap_or(1));
    }

    write_result.unwrap_or_else(|error| {
        eprintln!("Failed to write inspector compare input to Node.js: {error}");
        std::process::exit(1);
    });

    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        let stdout = std::str::from_utf8(&output.stdout).unwrap_or("<stdout is not valid UTF-8>");
        eprintln!("Failed to parse official compiler output: {error}\n{stdout}");
        std::process::exit(1);
    })
}

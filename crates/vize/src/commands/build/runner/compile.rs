//! Per-file production compilation from a shared Atlas snapshot.

use std::time::Instant;

use vize_atelier_sfc::{SfcCompileProduct, SfcDescriptorProduct};
use vize_atlas::CompilationSnapshot;
use vize_carton::profiler::global_profiler;
use vize_carton::{String, ToCompactString, cstr, profile};

use crate::commands::build::ScriptExtension;
use crate::commands::build::config::{
    CompileError, CompileOutput, CompileStats, ErrorPhase, FileProfile,
};

use super::artifact_graph::PreparedSource;
use super::profile_facts::{
    self, FileProfileFacts, StatsCacheStatus, record_atelier_profile_facts,
};
use super::settings::CompileFileSettings;

pub(super) fn compile_file_with_profile(
    prepared: &PreparedSource,
    snapshot: &CompilationSnapshot,
    settings: CompileFileSettings,
    stats: &CompileStats,
) -> Result<(CompileOutput, FileProfile), CompileError> {
    let file_start = Instant::now();
    let path = &prepared.path;
    let source = snapshot
        .source(prepared.source)
        .expect("prepared build sources belong to the captured snapshot");
    let file_size = source.text().len();
    let filename: String = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("anonymous.vue")
        .into();

    let mut session = snapshot.query_session();
    let parse_start = Instant::now();
    let descriptor_outcome = profile!(
        "atelier.sfc.parse",
        session.query::<SfcDescriptorProduct>(prepared.source)
    )
    .map_err(|error| CompileError {
        path: path.clone(),
        error: cstr!("{error}"),
        phase: ErrorPhase::Parse,
    })?;
    let descriptor_artifact = descriptor_outcome.value();
    if let Some(error) = descriptor_artifact.diagnostic() {
        return Err(CompileError {
            path: path.clone(),
            error: error.message.clone(),
            phase: ErrorPhase::Parse,
        });
    }
    let descriptor = descriptor_artifact
        .descriptor()
        .expect("descriptor artifacts contain a descriptor or diagnostic");
    let parse_time = parse_start.elapsed();
    if settings.record_profile_totals {
        stats.add_parse_time(parse_time);
    }

    let script_lang = descriptor
        .script_setup
        .as_ref()
        .and_then(|script| script.lang.as_deref())
        .or_else(|| {
            descriptor
                .script
                .as_ref()
                .and_then(|script| script.lang.as_deref())
        })
        .unwrap_or("js")
        .to_compact_string();
    let template_size = descriptor
        .template
        .as_ref()
        .map(|template| template.content.len())
        .unwrap_or(0);
    let script_size = descriptor
        .script
        .as_ref()
        .map(|script| script.content.len())
        .unwrap_or(0)
        + descriptor
            .script_setup
            .as_ref()
            .map(|script| script.content.len())
            .unwrap_or(0);
    let style_count = descriptor.styles.len();
    let has_scoped = descriptor.styles.iter().any(|style| style.scoped);
    let is_ts = matches!(settings.script_ext, ScriptExtension::Preserve);
    record_atelier_profile_facts(
        settings,
        template_size,
        script_size,
        style_count,
        has_scoped,
        is_ts,
    );

    let compile_start = Instant::now();
    let result = profile!(
        "atelier.sfc.compile",
        session.query::<SfcCompileProduct>(prepared.source)
    )
    .map_err(|error| CompileError {
        path: path.clone(),
        error: cstr!("{error}"),
        phase: ErrorPhase::Compile,
    })?
    .value()
    .clone();
    let compile_time = compile_start.elapsed();
    if settings.record_profile_totals {
        stats.add_compile_time(compile_time);
    }
    let total_time = prepared.read_time.saturating_add(file_start.elapsed());
    let profile = profile_facts::file_profile(
        path,
        FileProfileFacts {
            file_size,
            parse_time,
            compile_time,
            total_time,
            template_size,
            script_size,
            style_count,
        },
        settings,
        StatsCacheStatus::NotRequested,
    );
    let output = CompileOutput {
        filename,
        code: result.code,
        css: result.css,
        errors: result
            .errors
            .into_iter()
            .map(|error| error.message)
            .collect(),
        warnings: result
            .warnings
            .into_iter()
            .map(|warning| warning.message)
            .collect(),
        script_lang,
        macro_artifacts: result.macro_artifacts,
    };
    global_profiler().record_counter("atlas.query.requests", 2);

    Ok((output, profile))
}

//! Per-file compilation with profiling for the build command.

use std::{fs, path::PathBuf, sync::atomic::Ordering, time::Instant};

use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, StyleCompileOptions,
    TemplateCompileOptions, compile_sfc_with_template_syntax, parse_sfc,
};
use vize_carton::cstr;
use vize_carton::profile;
use vize_carton::profiler::global_profiler;
use vize_carton::{String, ToCompactString};

use crate::commands::build::ScriptExtension;
use crate::commands::build::config::{
    CompileError, CompileOutput, CompileStats, ErrorPhase, FileProfile,
};

use super::profile_facts::{self, FileProfileFacts, StatsCacheStatus};
use super::settings::CompileFileSettings;

pub(super) fn compile_file_with_profile(
    path: &PathBuf,
    settings: CompileFileSettings,
    stats: &CompileStats,
) -> Result<(CompileOutput, FileProfile), CompileError> {
    let file_start = Instant::now();

    // Read file
    let source = match profile!("cli.build.file.read", fs::read_to_string(path)) {
        Ok(source) => {
            global_profiler().record_fs_read_to_string(source.len());
            source
        }
        Err(error) => {
            global_profiler().record_fs_read_to_string_failure();
            return Err(CompileError {
                path: path.clone(),
                error: cstr!("Failed to read file: {}", error),
                phase: ErrorPhase::Read,
            });
        }
    };

    let file_size = source.len();
    stats.total_bytes.fetch_add(file_size, Ordering::Relaxed);

    let filename: String = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("anonymous.vue")
        .into();
    let source_id = path.to_string_lossy().as_ref().to_compact_string();

    // Parse
    let parse_start = Instant::now();
    let parse_opts = SfcParseOptions {
        filename: filename.clone(),
        ..Default::default()
    };

    let descriptor =
        profile!("atelier.sfc.parse", parse_sfc(&source, parse_opts)).map_err(|e| {
            CompileError {
                path: path.clone(),
                error: e.message,
                phase: ErrorPhase::Parse,
            }
        })?;
    let parse_time = parse_start.elapsed();
    if settings.record_profile_totals {
        stats.add_parse_time(parse_time);
    }

    let script_lang = descriptor
        .script_setup
        .as_ref()
        .and_then(|s| s.lang.as_deref())
        .or_else(|| descriptor.script.as_ref().and_then(|s| s.lang.as_deref()))
        .unwrap_or("js")
        .to_compact_string();

    // Calculate sizes
    let template_size = descriptor
        .template
        .as_ref()
        .map(|t| t.content.len())
        .unwrap_or(0);
    let script_size = descriptor
        .script
        .as_ref()
        .map(|s| s.content.len())
        .unwrap_or(0)
        + descriptor
            .script_setup
            .as_ref()
            .map(|s| s.content.len())
            .unwrap_or(0);
    let style_count = descriptor.styles.len();

    // Compile
    let compile_start = Instant::now();
    let has_scoped = descriptor.styles.iter().any(|s| s.scoped);
    let is_ts = matches!(settings.script_ext, ScriptExtension::Preserve);
    let compile_opts = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some(source_id),
            is_ts,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            id: Some(filename.clone()),
            scoped: has_scoped,
            ssr: settings.ssr,
            is_ts,
            custom_renderer: settings.custom_renderer,
            dialect: settings.dialect,
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename.clone(),
            scoped: has_scoped,
            ..Default::default()
        },
        vapor: settings.vapor,
        scope_id: None,
    };

    let result = profile!(
        "atelier.sfc.compile",
        compile_sfc_with_template_syntax(&descriptor, compile_opts, settings.template_syntax)
    )
    .map_err(|e| CompileError {
        path: path.clone(),
        error: e.message,
        phase: ErrorPhase::Compile,
    })?;
    let compile_time = compile_start.elapsed();
    if settings.record_profile_totals {
        stats.add_compile_time(compile_time);
    }

    let total_time = file_start.elapsed();

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
        errors: result.errors.into_iter().map(|e| e.message).collect(),
        warnings: result.warnings.into_iter().map(|e| e.message).collect(),
        script_lang,
        macro_artifacts: result.macro_artifacts,
    };

    Ok((output, profile))
}

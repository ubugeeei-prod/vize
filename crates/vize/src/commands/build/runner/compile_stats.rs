//! Stats-only per-file compilation with content-addressed cache reuse.

use std::{
    fs,
    path::PathBuf,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, StyleCompileOptions,
    TemplateCompileOptions, compile_sfc_with_template_syntax, parse_sfc,
};
use vize_carton::cstr;
use vize_carton::hash::hash_str;
use vize_carton::profile;
use vize_carton::profiler::global_profiler;
use vize_carton::{String, ToCompactString};

use crate::commands::build::ScriptExtension;
use crate::commands::build::config::{CompileError, CompileStats, ErrorPhase, FileProfile};

use super::cache::{
    StatsCompileCache, StatsCompileCacheEntry, StatsCompileCacheKey,
    stats_compile_cache_bypass_reason,
};
use super::profile_facts::{self, FileProfileFacts, StatsCacheStatus};
use super::settings::CompileFileSettings;

/// Compiles one file for `--format stats`, using content-addressed reuse.
///
/// The stats path only needs aggregate counters, so repeated source bodies can
/// skip parse/compile and reuse the cached output length and block metadata.
/// Every file is still read and counted. Cache hits get zero parse/compile time
/// so profile totals represent actual compiler work.
pub(super) fn compile_file_stats_with_cache(
    path: &PathBuf,
    settings: CompileFileSettings,
    stats: &CompileStats,
    cache: &StatsCompileCache,
) -> Result<(usize, FileProfile), CompileError> {
    let file_start = Instant::now();

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
    let component_name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
    let cache_bypass = stats_compile_cache_bypass_reason(&source, component_name);
    if let Some(reason) = cache_bypass {
        global_profiler().record_counter("cache.stats_compile.bypasses", 1);
        if reason == "self-component" {
            global_profiler().record_counter("cache.stats_compile.bypass.self_component", 1);
        }
    }
    let cache_key = cache_bypass.is_none().then(|| StatsCompileCacheKey {
        source_hash: hash_str(&source),
        source_len: file_size,
        component_name_len: component_name.len(),
        settings: settings.cache_bits(),
    });

    if let Some(key) = cache_key
        && let Some(entry) = cache
            .entries
            .lock()
            .map(|entries| entries.get(&key).cloned())
            .unwrap_or(None)
    {
        global_profiler().record_counter("cache.stats_compile.hits", 1);
        return match entry {
            StatsCompileCacheEntry::Success {
                output_bytes,
                template_size,
                script_size,
                style_count,
            } => Ok((
                output_bytes,
                profile_facts::file_profile(
                    path,
                    FileProfileFacts {
                        file_size,
                        parse_time: Duration::ZERO,
                        compile_time: Duration::ZERO,
                        total_time: file_start.elapsed(),
                        template_size,
                        script_size,
                        style_count,
                    },
                    settings,
                    StatsCacheStatus::Hit,
                ),
            )),
            StatsCompileCacheEntry::Failure { phase, message } => Err(CompileError {
                path: path.clone(),
                error: message,
                phase,
            }),
        };
    }
    if cache_key.is_some() {
        global_profiler().record_counter("cache.stats_compile.misses", 1);
    }

    let parse_start = Instant::now();
    let parse_opts = SfcParseOptions {
        filename: filename.clone(),
        ..Default::default()
    };
    let descriptor = match profile!("atelier.sfc.parse", parse_sfc(&source, parse_opts)) {
        Ok(descriptor) => descriptor,
        Err(error) => {
            cache_failure(cache, cache_key, ErrorPhase::Parse, error.message.clone());
            return Err(CompileError {
                path: path.clone(),
                error: error.message,
                phase: ErrorPhase::Parse,
            });
        }
    };
    let parse_time = parse_start.elapsed();
    if settings.record_profile_totals {
        stats.add_parse_time(parse_time);
    }

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
            id: filename,
            scoped: has_scoped,
            ..Default::default()
        },
        vapor: settings.vapor,
        scope_id: None,
    };

    let result = match profile!(
        "atelier.sfc.compile",
        compile_sfc_with_template_syntax(&descriptor, compile_opts, settings.template_syntax)
    ) {
        Ok(result) => result,
        Err(error) => {
            cache_failure(cache, cache_key, ErrorPhase::Compile, error.message.clone());
            return Err(CompileError {
                path: path.clone(),
                error: error.message,
                phase: ErrorPhase::Compile,
            });
        }
    };
    let compile_time = compile_start.elapsed();
    if settings.record_profile_totals {
        stats.add_compile_time(compile_time);
    }

    let output_bytes = result.code.len();
    let cache_status = if cache_key.is_some() {
        StatsCacheStatus::Miss
    } else {
        StatsCacheStatus::BypassSelfComponent
    };
    if let Some(key) = cache_key
        && let Ok(mut entries) = cache.entries.lock()
    {
        entries.entry(key).or_insert_with(|| {
            global_profiler().record_counter("cache.stats_compile.stores", 1);
            StatsCompileCacheEntry::Success {
                output_bytes,
                template_size,
                script_size,
                style_count,
            }
        });
    }

    Ok((
        output_bytes,
        profile_facts::file_profile(
            path,
            FileProfileFacts {
                file_size,
                parse_time,
                compile_time,
                total_time: file_start.elapsed(),
                template_size,
                script_size,
                style_count,
            },
            settings,
            cache_status,
        ),
    ))
}

fn cache_failure(
    cache: &StatsCompileCache,
    key: Option<StatsCompileCacheKey>,
    phase: ErrorPhase,
    message: String,
) {
    if let Some(key) = key
        && let Ok(mut entries) = cache.entries.lock()
    {
        entries.entry(key).or_insert_with(|| {
            global_profiler().record_counter("cache.stats_compile.stores", 1);
            StatsCompileCacheEntry::Failure { phase, message }
        });
    }
}

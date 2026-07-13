//! Stats-only compilation over Atlas with content-addressed result reuse.

use std::{time::Duration, time::Instant};

use vize_atelier_sfc::{SfcCompileProduct, SfcDescriptorProduct};
use vize_atlas::CompilationSnapshot;
use vize_carton::hash::hash_str;
use vize_carton::profiler::global_profiler;
use vize_carton::{String, cstr, profile};

use crate::commands::build::ScriptExtension;
use crate::commands::build::config::{CompileError, CompileStats, ErrorPhase, FileProfile};

use super::artifact_graph::PreparedSource;
use super::cache::{
    StatsCompileCache, StatsCompileCacheEntry, StatsCompileCacheKey, classify_stats_compile_cache,
};
use super::profile_facts::{
    self, FileProfileFacts, StatsCacheStatus, record_atelier_cache_decision,
    record_atelier_profile_facts,
};
use super::settings::CompileFileSettings;

pub(super) fn compile_file_stats_with_cache(
    prepared: &PreparedSource,
    snapshot: &CompilationSnapshot,
    settings: CompileFileSettings,
    stats: &CompileStats,
    cache: &StatsCompileCache,
) -> Result<(usize, FileProfile), CompileError> {
    let file_start = Instant::now();
    let path = &prepared.path;
    let source = snapshot
        .source(prepared.source)
        .expect("prepared build sources belong to the captured snapshot")
        .text();
    let file_size = source.len();
    let component_name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let cache_decision = classify_stats_compile_cache(source, component_name);
    record_atelier_cache_decision(settings, cache_decision);
    if !cache_decision.is_cacheable() {
        global_profiler().record_counter("cache.stats_compile.bypasses", 1);
        global_profiler().record_counter("cache.stats_compile.bypass.self_component", 1);
    }
    let cache_key = cache_decision.is_cacheable().then(|| StatsCompileCacheKey {
        source_hash: hash_str(source),
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
                        total_time: prepared.read_time.saturating_add(file_start.elapsed()),
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

    let mut session = snapshot.query_session();
    let parse_start = Instant::now();
    let descriptor = match profile!(
        "atelier.sfc.parse",
        session.query::<SfcDescriptorProduct>(prepared.source)
    ) {
        Ok(outcome) => {
            if let Some(error) = outcome.value().diagnostic() {
                cache_failure(cache, cache_key, ErrorPhase::Parse, error.message.clone());
                return Err(CompileError {
                    path: path.clone(),
                    error: error.message.clone(),
                    phase: ErrorPhase::Parse,
                });
            }
            outcome.shared()
        }
        Err(error) => {
            let message = cstr!("{error}");
            cache_failure(cache, cache_key, ErrorPhase::Parse, message.clone());
            return Err(CompileError {
                path: path.clone(),
                error: message,
                phase: ErrorPhase::Parse,
            });
        }
    };
    let descriptor = descriptor
        .descriptor()
        .expect("descriptor artifacts contain a descriptor or diagnostic");
    let parse_time = parse_start.elapsed();
    if settings.record_profile_totals {
        stats.add_parse_time(parse_time);
    }
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
    record_atelier_profile_facts(
        settings,
        template_size,
        script_size,
        style_count,
        has_scoped,
        matches!(settings.script_ext, ScriptExtension::Preserve),
    );

    let compile_start = Instant::now();
    let result = match profile!(
        "atelier.sfc.compile",
        session.query::<SfcCompileProduct>(prepared.source)
    ) {
        Ok(outcome) => outcome,
        Err(error) => {
            let message = cstr!("{error}");
            cache_failure(cache, cache_key, ErrorPhase::Compile, message.clone());
            return Err(CompileError {
                path: path.clone(),
                error: message,
                phase: ErrorPhase::Compile,
            });
        }
    };
    let compile_time = compile_start.elapsed();
    global_profiler().record_counter("atlas.query.requests", 2);
    if settings.record_profile_totals {
        stats.add_compile_time(compile_time);
    }
    let output_bytes = result.value().code.len();
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
                total_time: prepared.read_time.saturating_add(file_start.elapsed()),
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

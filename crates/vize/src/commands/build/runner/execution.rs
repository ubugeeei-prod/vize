//! Parallel execution of the production build artifact graph.

use std::{
    path::PathBuf,
    sync::{Mutex, atomic::Ordering},
    time::{Duration, Instant},
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::commands::build::config::{CompileError, CompileOutput, CompileStats, FileProfile};

use super::{
    artifact_graph::BuildArtifactGraph, cache::StatsCompileCache,
    compile::compile_file_with_profile, compile_stats::compile_file_stats_with_cache,
    settings::CompileFileSettings,
};

pub(super) struct CompileExecution {
    pub(super) results: Vec<Option<(PathBuf, CompileOutput)>>,
    pub(super) errors: Vec<CompileError>,
    pub(super) slow_files: Vec<FileProfile>,
    pub(super) profiles: Vec<FileProfile>,
    pub(super) elapsed: Duration,
}

pub(super) fn execute(
    files: &[PathBuf],
    settings: CompileFileSettings,
    stats: &CompileStats,
    stats_only: bool,
    collect_profiles: bool,
    slow_threshold: Duration,
) -> Result<CompileExecution, String> {
    let started = Instant::now();
    let (graph, read_errors) = BuildArtifactGraph::prepare(files, settings, stats)?;
    for _ in &read_errors {
        stats.failed.fetch_add(1, Ordering::Relaxed);
    }

    let errors = Mutex::new(read_errors);
    let slow_files = Mutex::new(Vec::new());
    let profiles = Mutex::new(Vec::new());
    let results = if stats_only {
        execute_stats(
            &graph,
            settings,
            stats,
            collect_profiles,
            slow_threshold,
            &errors,
            &slow_files,
            &profiles,
        );
        Vec::new()
    } else {
        execute_outputs(
            &graph,
            settings,
            stats,
            collect_profiles,
            slow_threshold,
            &errors,
            &slow_files,
            &profiles,
        )
    };

    Ok(CompileExecution {
        results,
        errors: errors.into_inner().unwrap_or_default(),
        slow_files: slow_files.into_inner().unwrap_or_default(),
        profiles: profiles.into_inner().unwrap_or_default(),
        elapsed: started.elapsed(),
    })
}

#[allow(clippy::too_many_arguments)]
fn execute_stats(
    graph: &BuildArtifactGraph,
    settings: CompileFileSettings,
    stats: &CompileStats,
    collect_profiles: bool,
    slow_threshold: Duration,
    errors: &Mutex<Vec<CompileError>>,
    slow_files: &Mutex<Vec<FileProfile>>,
    profiles: &Mutex<Vec<FileProfile>>,
) {
    let cache = StatsCompileCache::default();
    graph.sources.par_iter().for_each(|prepared| {
        match compile_file_stats_with_cache(prepared, &graph.snapshot, settings, stats, &cache) {
            Ok((output_bytes, profile)) => {
                stats.success.fetch_add(1, Ordering::Relaxed);
                stats
                    .output_bytes
                    .fetch_add(output_bytes, Ordering::Relaxed);
                collect_profile(
                    profile,
                    collect_profiles,
                    slow_threshold,
                    slow_files,
                    profiles,
                );
            }
            Err(error) => collect_error(error, stats, errors),
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn execute_outputs(
    graph: &BuildArtifactGraph,
    settings: CompileFileSettings,
    stats: &CompileStats,
    collect_profiles: bool,
    slow_threshold: Duration,
    errors: &Mutex<Vec<CompileError>>,
    slow_files: &Mutex<Vec<FileProfile>>,
    profiles: &Mutex<Vec<FileProfile>>,
) -> Vec<Option<(PathBuf, CompileOutput)>> {
    graph
        .sources
        .par_iter()
        .map(|prepared| {
            match compile_file_with_profile(prepared, &graph.snapshot, settings, stats) {
                Ok((output, profile)) => {
                    stats.success.fetch_add(1, Ordering::Relaxed);
                    stats
                        .output_bytes
                        .fetch_add(output.code.len(), Ordering::Relaxed);
                    collect_profile(
                        profile,
                        collect_profiles,
                        slow_threshold,
                        slow_files,
                        profiles,
                    );
                    Some((prepared.path.clone(), output))
                }
                Err(error) => {
                    collect_error(error, stats, errors);
                    None
                }
            }
        })
        .collect()
}

fn collect_profile(
    profile: FileProfile,
    collect_profiles: bool,
    slow_threshold: Duration,
    slow_files: &Mutex<Vec<FileProfile>>,
    profiles: &Mutex<Vec<FileProfile>>,
) {
    if profile.is_slow(slow_threshold)
        && let Ok(mut slow) = slow_files.lock()
    {
        slow.push(profile.clone());
    }
    if collect_profiles && let Ok(mut collected) = profiles.lock() {
        collected.push(profile);
    }
}

fn collect_error(error: CompileError, stats: &CompileStats, errors: &Mutex<Vec<CompileError>>) {
    stats.failed.fetch_add(1, Ordering::Relaxed);
    if let Ok(mut collected) = errors.lock() {
        collected.push(error);
    }
}

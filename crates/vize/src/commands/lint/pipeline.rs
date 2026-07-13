//! Parallel file IO, Atlas lint queries, and autofix revalidation.

use std::{fs, path::PathBuf, time::Duration, time::Instant};

use rayon::prelude::*;
use vize_atlas::Shared;
use vize_carton::{
    String, ToCompactString, config::VueVersion, profile, profiler::global_profiler,
};
#[cfg(test)]
use vize_croquis::CroquisDocument;
use vize_patina::{LintResult, Linter};

use super::{
    aggregate::LintRunAccumulator,
    artifact_graph::{ArtifactLintOutcome, LintArtifactGraph},
    fix::apply_lint_fixes,
};

pub(super) struct LintInput {
    pub(super) path: PathBuf,
    pub(super) filename: String,
    pub(super) source: String,
    pub(super) read_time: Duration,
}

pub(super) struct LintedFile {
    pub(super) source_index: usize,
    pub(super) path: PathBuf,
    pub(super) filename: String,
    pub(super) source: String,
    pub(super) result: LintResult,
    #[cfg(test)]
    pub(super) semantics: Option<Shared<CroquisDocument>>,
    pub(super) read_time: Duration,
    pub(super) lint_time: Duration,
    pub(super) fixed: bool,
    pub(super) write_failed: bool,
    #[cfg(test)]
    pub(super) artifact_backed: bool,
}

pub(super) struct LintRun {
    graph: LintArtifactGraph,
    accumulator: LintRunAccumulator,
}

impl LintRun {
    pub(super) fn into_parts(self) -> (LintArtifactGraph, Vec<LintedFile>, (usize, usize, usize)) {
        let (files, totals) = self.accumulator.into_parts();
        (self.graph, files, totals)
    }
}

pub(super) fn read_lint_inputs(files: &[PathBuf], profiling: bool) -> Vec<LintInput> {
    files
        .par_iter()
        .filter_map(|path| {
            let read_start = profiling.then(Instant::now);
            let source: String = match profile!("cli.lint.file.read", fs::read_to_string(path)) {
                Ok(source) => {
                    global_profiler().record_fs_read_to_string(source.len());
                    source.into()
                }
                Err(error) => {
                    global_profiler().record_fs_read_to_string_failure();
                    eprintln!("Failed to read {}: {}", path.display(), error);
                    return None;
                }
            };
            Some(LintInput {
                filename: path.to_string_lossy().to_compact_string(),
                path: path.clone(),
                source,
                read_time: read_start
                    .map(|start| start.elapsed())
                    .unwrap_or(Duration::ZERO),
            })
        })
        .collect()
}

pub(super) fn lint_inputs(
    inputs: Vec<LintInput>,
    linter: Shared<Linter>,
    dialect: VueVersion,
    should_fix: bool,
    profiling: bool,
    retain_file_results: bool,
) -> LintRun {
    let graph = LintArtifactGraph::new(
        linter,
        dialect,
        inputs
            .iter()
            .map(|input| (input.path.as_path(), input.source.as_str())),
    )
    .expect("lint artifact graph registration must be valid");
    let accumulator = if should_fix {
        lint_with_fixes(inputs, &graph, profiling, retain_file_results)
    } else {
        reduce_files(
            inputs
                .into_par_iter()
                .enumerate()
                .map(|(index, input)| lint_input(index, input, &graph, profiling)),
            retain_file_results,
        )
    };
    LintRun { graph, accumulator }
}

fn lint_input(
    index: usize,
    input: LintInput,
    graph: &LintArtifactGraph,
    profiling: bool,
) -> LintedFile {
    let lint_start = profiling.then(Instant::now);
    let outcome = profile!("cli.lint.file.lint", lint_once(index, graph));
    into_linted_file(index, input, outcome, lint_start, false, false)
}

struct PreparedLint {
    index: usize,
    file: LintedFile,
    revalidate_artifact: bool,
    profiling: bool,
}

fn lint_with_fixes(
    inputs: Vec<LintInput>,
    graph: &LintArtifactGraph,
    profiling: bool,
    retain_file_results: bool,
) -> LintRunAccumulator {
    let prepared: Vec<_> = inputs
        .into_par_iter()
        .enumerate()
        .map(|(index, input)| prepare_fixed_input(index, input, graph, profiling))
        .collect();
    let revisions: Vec<_> = prepared
        .iter()
        .filter(|prepared| prepared.file.fixed)
        .map(|prepared| (prepared.index, prepared.file.source.as_str()))
        .collect();
    graph
        .revise_sources(&revisions)
        .unwrap_or_else(|error| panic!("fixed Atlas source update failed: {error}"));
    reduce_files(
        prepared
            .into_par_iter()
            .map(|prepared| finish_fixed_input(prepared, graph)),
        retain_file_results,
    )
}

fn prepare_fixed_input(
    index: usize,
    mut input: LintInput,
    graph: &LintArtifactGraph,
    profiling: bool,
) -> PreparedLint {
    let lint_start = profiling.then(Instant::now);
    let outcome = profile!("cli.lint.file.lint", lint_once(index, graph));
    let mut fixed = false;
    let mut write_failed = false;
    let mut revalidate_artifact = false;
    if let Some(fixed_source) = apply_lint_fixes(&input.source, &outcome.result)
        && fixed_source != input.source
    {
        if let Err(error) =
            crate::commands::atomic_write::atomic_write(&input.path, fixed_source.as_bytes())
        {
            global_profiler().record_fs_write_failure(fixed_source.len());
            eprintln!("Failed to write {}: {}", input.path.display(), error);
            write_failed = true;
        } else {
            global_profiler().record_fs_write(fixed_source.len());
            input.source = fixed_source;
            fixed = true;
            revalidate_artifact = true;
        }
    }
    PreparedLint {
        index,
        file: into_linted_file(index, input, outcome, lint_start, fixed, write_failed),
        revalidate_artifact,
        profiling,
    }
}

fn finish_fixed_input(mut prepared: PreparedLint, graph: &LintArtifactGraph) -> LintedFile {
    if prepared.revalidate_artifact {
        let start = prepared.profiling.then(Instant::now);
        let outcome = profile!(
            "cli.lint.file.lint",
            graph
                .query(prepared.index)
                .map(atlas_outcome)
                .unwrap_or_else(|error| panic!("fixed Atlas lint query failed: {error}"))
        );
        prepared.file.result = outcome.result;
        #[cfg(test)]
        {
            prepared.file.semantics = outcome.semantics;
        }
        prepared.file.lint_time += start.map_or(Duration::ZERO, |start| start.elapsed());
    }
    prepared.file
}

fn into_linted_file(
    source_index: usize,
    input: LintInput,
    outcome: LintOnceOutcome,
    lint_start: Option<Instant>,
    fixed: bool,
    write_failed: bool,
) -> LintedFile {
    LintedFile {
        source_index,
        path: input.path,
        filename: input.filename,
        source: input.source,
        result: outcome.result,
        #[cfg(test)]
        semantics: outcome.semantics,
        read_time: input.read_time,
        lint_time: lint_start.map_or(Duration::ZERO, |start| start.elapsed()),
        fixed,
        write_failed,
        #[cfg(test)]
        artifact_backed: outcome.artifact_backed,
    }
}

fn reduce_files<I>(files: I, retain_file_results: bool) -> LintRunAccumulator
where
    I: rayon::iter::ParallelIterator<Item = LintedFile>,
{
    files
        .fold(
            || LintRunAccumulator::new(retain_file_results),
            LintRunAccumulator::push,
        )
        .reduce(
            || LintRunAccumulator::new(retain_file_results),
            LintRunAccumulator::merge,
        )
}

struct LintOnceOutcome {
    result: LintResult,
    #[cfg(test)]
    semantics: Option<Shared<CroquisDocument>>,
    #[cfg(test)]
    artifact_backed: bool,
}

fn lint_once(index: usize, graph: &LintArtifactGraph) -> LintOnceOutcome {
    atlas_outcome(
        graph
            .query(index)
            .unwrap_or_else(|error| panic!("Atlas lint query failed: {error}")),
    )
}

fn atlas_outcome(outcome: ArtifactLintOutcome) -> LintOnceOutcome {
    LintOnceOutcome {
        result: outcome.result,
        #[cfg(test)]
        semantics: outcome.semantics,
        #[cfg(test)]
        artifact_backed: true,
    }
}

#[cfg(test)]
#[path = "pipeline/tests.rs"]
mod tests;

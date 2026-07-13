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
    artifact_graph::{ArtifactLintOutcome, LintArtifactGraph, is_artifact_path},
    fix::{apply_lint_fixes, lint_source},
};

pub(super) struct LintInput {
    pub(super) path: PathBuf,
    pub(super) filename: String,
    pub(super) source: String,
    pub(super) read_time: Duration,
}

pub(super) struct LintedFile {
    pub(super) path: PathBuf,
    pub(super) filename: String,
    pub(super) source: String,
    pub(super) result: LintResult,
    #[cfg(test)]
    pub(super) semantics: Option<Shared<CroquisDocument>>,
    pub(super) read_time: Duration,
    pub(super) lint_time: Duration,
    pub(super) fixed: bool,
    #[cfg(test)]
    pub(super) artifact_backed: bool,
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
) -> Vec<LintedFile> {
    let graph = LintArtifactGraph::new(
        Shared::clone(&linter),
        dialect,
        inputs
            .iter()
            .map(|input| (input.path.as_path(), input.source.as_str())),
    )
    .expect("lint artifact graph registration must be valid");
    inputs
        .into_par_iter()
        .enumerate()
        .map(|(index, input)| {
            lint_input(
                index,
                input,
                &graph,
                Shared::clone(&linter),
                should_fix,
                profiling,
            )
        })
        .collect()
}

fn lint_input(
    index: usize,
    mut input: LintInput,
    graph: &LintArtifactGraph,
    linter: Shared<Linter>,
    should_fix: bool,
    profiling: bool,
) -> LintedFile {
    let lint_start = profiling.then(Instant::now);
    let (outcome, fixed) = profile!("cli.lint.file.lint", {
        let mut outcome = lint_once(index, &input, graph, Shared::clone(&linter));
        let mut fixed = false;
        if should_fix
            && let Some(fixed_source) = apply_lint_fixes(&input.source, &outcome.result)
            && fixed_source != input.source
        {
            if let Err(error) = fs::write(&input.path, fixed_source.as_bytes()) {
                global_profiler().record_fs_write_failure(fixed_source.len());
                eprintln!("Failed to write {}: {}", input.path.display(), error);
            } else {
                global_profiler().record_fs_write(fixed_source.len());
                input.source = fixed_source;
                outcome = lint_fixed(index, &input, graph, Shared::clone(&linter));
                fixed = true;
            }
        }
        (outcome, fixed)
    });
    LintedFile {
        path: input.path,
        filename: input.filename,
        source: input.source,
        result: outcome.result,
        #[cfg(test)]
        semantics: outcome.semantics,
        read_time: input.read_time,
        lint_time: lint_start
            .map(|start| start.elapsed())
            .unwrap_or(Duration::ZERO),
        fixed,
        #[cfg(test)]
        artifact_backed: outcome.artifact_backed,
    }
}

struct LintOnceOutcome {
    result: LintResult,
    #[cfg(test)]
    semantics: Option<Shared<CroquisDocument>>,
    #[cfg(test)]
    artifact_backed: bool,
}

fn lint_once(
    index: usize,
    input: &LintInput,
    graph: &LintArtifactGraph,
    linter: Shared<Linter>,
) -> LintOnceOutcome {
    if is_artifact_path(&input.path) {
        return atlas_outcome(
            graph
                .query(index)
                .unwrap_or_else(|error| panic!("Vue Atlas lint query failed: {error}")),
        );
    }
    direct_outcome(&linter, input)
}

fn lint_fixed(
    index: usize,
    input: &LintInput,
    graph: &LintArtifactGraph,
    linter: Shared<Linter>,
) -> LintOnceOutcome {
    if is_artifact_path(&input.path) {
        return atlas_outcome(
            graph
                .query_revised(index, &input.source)
                .unwrap_or_else(|error| panic!("fixed Atlas lint query failed: {error}")),
        );
    }
    direct_outcome(&linter, input)
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

fn direct_outcome(linter: &Linter, input: &LintInput) -> LintOnceOutcome {
    debug_assert!(!is_artifact_path(&input.path));
    LintOnceOutcome {
        result: lint_source(linter, &input.path, &input.source, &input.filename),
        #[cfg(test)]
        semantics: None,
        #[cfg(test)]
        artifact_backed: false,
    }
}

#[cfg(test)]
#[path = "pipeline/tests.rs"]
mod tests;

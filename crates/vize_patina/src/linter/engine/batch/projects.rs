use super::{LintResult, Linter, PreparedLint, TypeAwareFailure};
use crate::linter::{corsa_session, corsa_session_pool::CorsaSessionKey, native_type_aware};
use rayon::prelude::*;
use std::path::PathBuf;
use vize_l0::{FxHashMap, ToCompactString, profile};

#[derive(Eq, PartialEq, Hash)]
struct ProjectKey {
    session: CorsaSessionKey,
    runtime: Option<PathBuf>,
}

type Input<'a> = (usize, &'a Linter, PreparedLint);

pub(super) fn finish(inputs: Vec<(&Linter, PreparedLint)>) -> Vec<LintResult> {
    let count = inputs.len();
    let mut groups: FxHashMap<ProjectKey, Vec<Input<'_>>> = FxHashMap::default();
    let mut results = Vec::with_capacity(count);
    for (index, (linter, prepared)) in inputs.into_iter().enumerate() {
        if prepared.virtual_content().is_none() {
            results.push((index, prepared.finish(linter, None)));
            continue;
        }
        match project_key(linter, &prepared.filename) {
            Ok(key) => groups
                .entry(key)
                .or_default()
                .push((index, linter, prepared)),
            Err(failure) => results.push((index, prepared.finish(linter, Some(failure)))),
        }
    }
    let projects: Vec<_> = groups
        .into_values()
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(finish_project)
        .collect();
    results.extend(projects.into_iter().flatten());
    results.sort_unstable_by_key(|(index, _)| *index);
    results.into_iter().map(|(_, result)| result).collect()
}

fn project_key(linter: &Linter, filename: &str) -> Result<ProjectKey, TypeAwareFailure> {
    let mut ownership = linter.type_aware_project_ownership.lock().map_err(|_| {
        TypeAwareFailure::Runtime("Failed to lock type-aware project ownership".to_compact_string())
    })?;
    let tsconfig = corsa_session::effective_tsconfig_for_source(
        filename,
        linter.type_aware_tsconfig.as_deref(),
        &mut ownership,
    );
    Ok(ProjectKey {
        session: CorsaSessionKey {
            project_root: corsa_session::resolve_project_root(filename),
            tsconfig,
        },
        runtime: linter.corsa_path.clone(),
    })
}

fn finish_project(mut inputs: Vec<Input<'_>>) -> Vec<(usize, LintResult)> {
    let Some((_, owner, first)) = inputs.first() else {
        return Vec::new();
    };
    let owner = *owner;
    let filename = first.filename.clone();
    let mut published = false;
    let mut failures = Vec::with_capacity(inputs.len());
    let outcome = native_type_aware::with_corsa_session(owner, &filename, |session| {
        let documents: Vec<_> = inputs
            .iter()
            .filter_map(|(_, _, prepared)| {
                prepared
                    .virtual_content()
                    .map(|source| (source, prepared.filename.as_str()))
            })
            .collect();
        profile!(
            "patina.type_aware.project.publish",
            session.open_virtual_projects(&documents)
        )?;
        published = true;
        for (_, linter, prepared) in &mut inputs {
            failures.push(
                profile!(
                    "patina.type_aware.project.evaluate",
                    prepared.evaluate(linter, session)
                )
                .err(),
            );
        }
        // Keep independent file queries available, then retire a suspect session.
        failures
            .iter()
            .find_map(Option::as_ref)
            .cloned()
            .map_or(Ok(()), Err)
    });

    inputs
        .into_iter()
        .enumerate()
        .map(|(position, (index, linter, prepared))| {
            let failure = if published {
                failures
                    .get_mut(position)
                    .and_then(Option::take)
                    .map(TypeAwareFailure::Runtime)
            } else {
                outcome.as_ref().err().map(|failure| match failure {
                    TypeAwareFailure::Runtime(message) => {
                        TypeAwareFailure::Runtime(message.clone())
                    }
                    TypeAwareFailure::ProjectConfiguration { message, report } => {
                        TypeAwareFailure::ProjectConfiguration {
                            message: message.clone(),
                            report: *report && position == 0,
                        }
                    }
                })
            };
            (index, prepared.finish(linter, failure))
        })
        .collect()
}

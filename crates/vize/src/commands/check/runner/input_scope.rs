//! What a `vize check` run's collected inputs actually cover, and what to
//! report when that scope does not match the directory the run was invoked
//! from.
//!
//! A run with no explicit inputs takes its file set from the nearest
//! `tsconfig.json` at or above the working directory, and that walk-up has no
//! stop condition: it keeps going until some ancestor happens to own a
//! `tsconfig.json`. When the project it lands on contains no file under the
//! working directory, the run type-checks that unrelated project and reports
//! success — the working directory's own broken sources are never looked at
//! (#3320). The failure mode is a silent false negative, so it is reported as
//! an error instead of being checked around.

use std::path::{Path, PathBuf};

use vize_s0::{String, cstr};

use super::super::{CheckArgs, patterns::CHECK_INPUTS_DISPLAY, reporting::JsonOutput};
use super::{collect::path_is_inside_root, diagnostics::emit_json_output};

/// Exit code for a run that could not determine what to check. Distinct from
/// `1`, which means "type errors were reported".
const UNRESOLVED_SCOPE_EXIT_CODE: i32 = 2;

/// How a default run's collected inputs relate to the working directory.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum DefaultRunScope {
    /// The project root is the working directory or below it, or every input
    /// already lives inside the working directory. Nothing to surface.
    Owned,
    /// The project root sits above the working directory and part of its
    /// program lives outside it. Legitimate — a monorepo-root `tsconfig.json`
    /// that owns this directory's sources alongside its siblings' — but the run
    /// covers more than the invocation implies, so the scope is surfaced.
    Widened { inputs: usize, outside: usize },
    /// The project root sits above the working directory and *no* input lives
    /// inside it: discovery walked past the working directory into a project
    /// that does not contain it.
    Unowned { inputs: usize },
}

/// Classify what `files` covers relative to `cwd`.
///
/// Paths are compared through [`path_is_inside_root`], which canonicalizes both
/// sides. That is what makes the classification stable for the layouts in
/// #3320: a `node_modules` symlinked into an out-of-tree store, an individual
/// package symlinked in from a pnpm store, or the whole project reached through
/// a symlinked path all produce inputs whose spelling differs from the
/// invocation directory's while still belonging to it. Comparing resolved paths
/// — rather than asking whether any component is a link — also keeps Windows
/// directory junctions equivalent to symlinks here, since `canonicalize`
/// resolves both.
///
/// A project root that is neither the working directory nor an ancestor of it
/// (an explicit `--tsconfig` pointing at a sibling tree, say) is left alone:
/// only the unbounded walk-up is under judgement here.
pub(super) fn classify_default_run_scope(
    files: &[PathBuf],
    cwd: &Path,
    project_root: &Path,
) -> DefaultRunScope {
    if files.is_empty() || !is_strict_ancestor(project_root, cwd) {
        return DefaultRunScope::Owned;
    }

    let outside = files
        .iter()
        .filter(|file| !path_is_inside_root(cwd, file))
        .count();
    if outside == 0 {
        DefaultRunScope::Owned
    } else if outside == files.len() {
        DefaultRunScope::Unowned {
            inputs: files.len(),
        }
    } else {
        DefaultRunScope::Widened {
            inputs: files.len(),
            outside,
        }
    }
}

/// Report the resolved scope of a default run, and exit when the resolved
/// project does not contain the working directory at all.
///
/// `tsconfig_path` is `None` only when no config was adopted, which means no
/// walk-up happened and there is nothing to judge.
pub(super) fn exit_if_default_run_leaves_cwd(
    files: &[PathBuf],
    cwd: &Path,
    project_root: &Path,
    tsconfig_path: Option<&Path>,
    quiet: bool,
) {
    let Some(tsconfig_path) = tsconfig_path else {
        return;
    };
    match classify_default_run_scope(files, cwd, project_root) {
        DefaultRunScope::Owned => {}
        DefaultRunScope::Widened { inputs, outside } => {
            if !quiet {
                eprintln!("{}", widened_scope_note(cwd, project_root, inputs, outside));
            }
        }
        DefaultRunScope::Unowned { inputs } => {
            eprintln!(
                "\x1b[31mError:\x1b[0m {}",
                unowned_project_error(cwd, project_root, tsconfig_path, inputs)
            );
            std::process::exit(UNRESOLVED_SCOPE_EXIT_CODE);
        }
    }
}

/// Message for a run whose resolved project contains nothing under the working
/// directory. It names both directories, the config that was adopted, and the
/// size of the program that would have been reported instead, because the
/// symptom users see otherwise is an unexplained clean run.
pub(super) fn unowned_project_error(
    cwd: &Path,
    project_root: &Path,
    tsconfig_path: &Path,
    inputs: usize,
) -> String {
    cstr!(
        "`{}` has no tsconfig.json, and the nearest one above it (`{}`) type-checks {} files \
         under `{}`, none of them inside `{}`. Reporting that project's result for this \
         directory would hide every error here, so nothing was checked: add a tsconfig.json to \
         `{}`, pass `--tsconfig <path>`, or name the files to check.",
        cwd.display(),
        tsconfig_path.display(),
        inputs,
        project_root.display(),
        cwd.display(),
        cwd.display()
    )
}

/// Note for a run whose resolved project legitimately owns this directory but
/// reaches beyond it, so a root resolved higher than intended stays visible
/// instead of hiding inside the file count.
pub(super) fn widened_scope_note(
    cwd: &Path,
    project_root: &Path,
    inputs: usize,
    outside: usize,
) -> String {
    cstr!(
        "vize check: the project root resolved to `{}`, above the working directory `{}`; {} of \
         {} checked files are outside `{}`.",
        project_root.display(),
        cwd.display(),
        outside,
        inputs,
        cwd.display()
    )
}

/// Report a run that collected no inputs at all.
pub(super) fn report_no_inputs(args: &CheckArgs) {
    if args.format == "json" {
        emit_json_output(JsonOutput {
            files: Vec::new(),
            programs: Vec::new(),
            error_count: 0,
            warning_count: 0,
            file_count: 0,
            declarations: None,
        })
        .unwrap_or_else(|error| {
            eprintln!("Failed to report empty check result: {error}");
            std::process::exit(1);
        });
        return;
    }
    eprintln!(
        "No {CHECK_INPUTS_DISPLAY} files found matching inputs: {:?}",
        args.patterns
    );
}

fn is_strict_ancestor(ancestor: &Path, path: &Path) -> bool {
    let ancestor = vize_s0::path::canonicalize_non_verbatim(ancestor);
    let path = vize_s0::path::canonicalize_non_verbatim(path);
    path != ancestor && path.starts_with(&ancestor)
}

#[cfg(test)]
mod tests;

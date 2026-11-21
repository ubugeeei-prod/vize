//! Resolve lint and compiler whitespace policy against the same configured file.

use std::path::{Path, PathBuf};

use super::{
    CompilerConfigEntry, LinterConfigPlanWithConfigRuleOptions,
    ResolvedLinterConfigWithConfigRuleOptions,
    matcher::{LintPlanScope, ProjectIgnoreSet, absolute_path, preserves_authored_text},
};
use crate::{FxHashMap, SmallVec, String};

/// Policies requested together by formatting and editor hosts.
#[derive(Debug, PartialEq, Eq)]
pub struct FilePolicy {
    /// An ignored file has no active lint policy.
    pub lint: Option<ResolvedLinterConfigWithConfigRuleOptions>,
    /// Whether the compiler preserves authored template whitespace.
    pub preserves_text: bool,
}

type ScopeKey<'k> = (Option<&'k str>, Option<&'k [String]>, &'k [String]);

struct Scope {
    matcher: LintPlanScope,
    lint_entries: SmallVec<[usize; 1]>,
    whitespace: Option<(usize, bool)>,
}

/// Compiled once for one evaluated configuration and its host-selected directory.
/// Both projections share the file path and each distinct active scope match.
pub struct ProjectFilePlan {
    scopes: Vec<Scope>,
    lint: LinterConfigPlanWithConfigRuleOptions,
    ignores: Option<ProjectIgnoreSet>,
    root_preserves_text: bool,
    cwd: PathBuf,
}

impl ProjectFilePlan {
    /// Consume the lint projection and borrow the compiler entries of the same
    /// configuration evaluation. Rebuild the plan when the host replaces that snapshot.
    #[must_use]
    pub fn new(
        lint: LinterConfigPlanWithConfigRuleOptions,
        root_whitespace: Option<&str>,
        compiler_entries: &[CompilerConfigEntry],
        config_dir: &Path,
    ) -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut scopes = FxHashMap::default();
        for (index, entry) in lint.plan.entries.iter().enumerate() {
            let key = (
                entry.base_path.as_deref(),
                entry.files.as_deref(),
                entry.ignores.as_slice(),
            );
            intern(&mut scopes, key, config_dir, &cwd)
                .lint_entries
                .push(index);
        }
        for (index, entry) in compiler_entries.iter().enumerate() {
            if let Some(mode) = entry.whitespace.as_deref() {
                let key = (
                    entry.base_path.as_deref(),
                    entry.files.as_deref(),
                    entry.ignores.as_slice(),
                );
                // The last compiler entry with an identical selector always wins.
                intern(&mut scopes, key, config_dir, &cwd).whitespace =
                    Some((index, preserves_authored_text(Some(mode))));
            }
        }
        let ignores = ProjectIgnoreSet::new(&lint.plan.global_ignores, config_dir);
        Self {
            scopes: scopes.into_values().collect(),
            lint,
            ignores,
            root_preserves_text: preserves_authored_text(root_whitespace),
            cwd,
        }
    }

    /// Compute the original per-file values, including compiler policy on lint-ignored files.
    #[must_use]
    pub fn resolve(&self, path: &Path) -> FilePolicy {
        let path = absolute_path(path, &self.cwd);
        let ignored = self
            .ignores
            .as_ref()
            .is_some_and(|ignores| ignores.is_ignored(&path));
        let mut lint_entries = SmallVec::<[usize; 4]>::new();
        let mut whitespace = None;
        for scope in &self.scopes {
            if (ignored && scope.whitespace.is_none()) || !scope.matcher.matches(&path) {
                continue;
            }
            if !ignored {
                lint_entries.extend_from_slice(&scope.lint_entries);
            }
            if let Some(candidate) = scope.whitespace
                && whitespace.is_none_or(|(index, _)| candidate.0 > index)
            {
                whitespace = Some(candidate);
            }
        }
        FilePolicy {
            lint: (!ignored).then(|| self.lint.resolve_matching_entries(&lint_entries)),
            preserves_text: whitespace.map_or(self.root_preserves_text, |(_, preserve)| preserve),
        }
    }
}

fn intern<'s, 'k>(
    scopes: &'s mut FxHashMap<ScopeKey<'k>, Scope>,
    key: ScopeKey<'k>,
    config_dir: &Path,
    cwd: &Path,
) -> &'s mut Scope {
    scopes.entry(key).or_insert_with(|| Scope {
        matcher: LintPlanScope::new(key.0, key.1, key.2, config_dir, cwd),
        lint_entries: SmallVec::new(),
        whitespace: None,
    })
}

#[cfg(test)]
mod tests;

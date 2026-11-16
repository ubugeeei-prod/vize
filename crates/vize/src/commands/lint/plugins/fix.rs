//! `vize lint --fix` for plugin rules: apply the fixes of rules declared
//! `fixable: "safe"`, re-lint the changed files, and repeat a bounded number
//! of passes. Unsafe fixes and v1 suggestions are never applied.

use std::path::Path;

use rayon::prelude::*;
use vize_l0::{FxHashSet, String, cstr, profiler::global_profiler};
use vize_patina::LintResult;

use super::{PluginLint, PluginLintReceipt};
use crate::commands::atomic_write::atomic_write;
use crate::commands::lint::cross_file::CliLintFileResult;
use crate::commands::lint::fix::apply_lint_fixes;

/// Each pass is one more sidecar crossing over the changed files only.
const MAX_PLUGIN_FIX_PASSES: usize = 5;

impl PluginLint {
    /// `relint` lints a fixed source with the built-in rules and applies their
    /// fixes; `None` when no linter owns the file.
    pub(in crate::commands::lint) fn fix(
        &self,
        results: &mut [CliLintFileResult],
        receipt: &mut PluginLintReceipt,
        relint: impl Fn(&Path, String, &str) -> Option<(String, LintResult)> + Sync,
    ) -> Result<(), String> {
        let safe: FxHashSet<&str> = self
            .jobs
            .iter()
            .filter_map(|job| job.rules.safe_fix_id())
            .collect();
        if safe.is_empty() {
            return Ok(());
        }
        for _ in 0..MAX_PLUGIN_FIX_PASSES {
            // Files are independent, so they are fixed, re-linted and written in
            // parallel, with one write per changed file and pass.
            let changed = results
                .par_iter_mut()
                .enumerate()
                .map(|(index, (path, filename, source, result))| {
                    let fixes = LintResult {
                        filename: result.filename.clone(),
                        diagnostics: result
                            .diagnostics
                            .iter()
                            .filter(|diagnostic| {
                                diagnostic.fix.is_some() && safe.contains(diagnostic.rule_name)
                            })
                            .cloned()
                            .collect(),
                        error_count: 0,
                        warning_count: 0,
                    };
                    let Some(fixed) =
                        apply_lint_fixes(source, &fixes).filter(|fixed| fixed != source)
                    else {
                        return Ok(None);
                    };
                    let (relinted_source, relinted) = relint(path, fixed, filename)
                        .ok_or_else(|| cstr!("Failed to re-lint {}", path.display()))?;
                    // The built-in fixes can undo a plugin fix; the file is then unchanged.
                    if relinted_source == *source {
                        return Ok(None);
                    }
                    atomic_write(path, relinted_source.as_bytes()).map_err(|error| {
                        global_profiler().record_fs_write_failure(relinted_source.len());
                        cstr!("Failed to write {}: {error}", path.display())
                    })?;
                    global_profiler().record_fs_write(relinted_source.len());
                    *source = relinted_source;
                    *result = relinted;
                    Ok(Some(index))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let changed: Vec<usize> = changed.into_iter().flatten().collect();
            if changed.is_empty() {
                return Ok(());
            }
            self.pass(results, Some(&changed), receipt)?;
        }
        Ok(())
    }
}

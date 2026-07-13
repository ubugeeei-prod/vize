//! External type graph traversal for script compile contexts.

mod resolution;
mod summary;

use std::path::Path;
use std::sync::atomic::AtomicU64;

use vize_carton::{FxHashSet, String};

use super::ScriptCompileContext;
use super::batch_epoch::current_batch_epoch;
use resolution::{canonical_base_file, path_key, resolve_import_path};
use summary::{
    CachedFileSummary, FILE_TYPE_CACHE, FileTypeSummary, build_file_summary,
    extract_script_summary, file_stamp,
};

pub(crate) use summary::type_import_specifiers_from_program;

impl ScriptCompileContext {
    /// Walk the script's type-bearing imports/re-exports on disk and merge the
    /// interfaces/type aliases they declare into this context.
    ///
    /// `is_ts` must reflect whether the script block is TypeScript
    /// (`lang="ts"`/`"tsx"`, computed once per compile at the call site) — it
    /// is the real signal, derived from the parsed SFC, that replaced the old
    /// `source.contains("type")` substring pre-check. Imported *types* can only
    /// be referenced from TypeScript (`defineProps<Props>()`), so for plain JS
    /// the walk would only burn stat/realpath syscalls; the substring heuristic
    /// misfired on JS object keys like `type: 'text'` next to any `import`,
    /// which is exactly what the `is_ts` gate cuts off.
    pub fn collect_imported_types_from_path(&mut self, source: &str, filename: &str, is_ts: bool) {
        if !is_ts {
            return;
        }

        // The root source lives in memory (possibly unsaved editor state), so
        // parse it directly; only files read from disk go through the cache.
        // The parsed specifier list is the precise "is there anything to
        // follow?" signal — strictly tighter than the old substring guard, so
        // no separate text pre-check is needed.
        let mut root = FileTypeSummary::default();
        extract_script_summary(source, &mut root);
        if root.specifiers.is_empty() {
            // Nothing to resolve — skip base-file canonicalization entirely
            // (the common case: scripts with only runtime imports).
            return;
        }

        self.collect_imported_types_from_specifiers(&root.specifiers, filename);
    }

    /// Resolve type-bearing specifiers captured from an already parsed authored
    /// script. Only the referenced external files are parsed here.
    pub fn collect_imported_types_from_specifiers(
        &mut self,
        specifiers: &[String],
        filename: &str,
    ) {
        if specifiers.is_empty() {
            return;
        }
        let owned_base = canonical_base_file(filename);
        let base_file = owned_base.as_path();
        let Some(base_dir) = base_file.parent() else {
            return;
        };
        if base_dir.as_os_str().is_empty() {
            return;
        }

        let mut visited = FxHashSet::default();
        for specifier in specifiers {
            self.collect_types_from_specifier(specifier, base_file, &mut visited);
        }
    }

    fn collect_types_from_specifier(
        &mut self,
        specifier: &str,
        current_file: &Path,
        visited: &mut FxHashSet<String>,
    ) {
        let Some(resolved_path) = resolve_import_path(current_file, specifier) else {
            return;
        };

        let key = path_key(&resolved_path);
        if !visited.insert(key) {
            return;
        }

        // Fast path: merge the declarations under the read guard and only
        // clone the (small) specifier list for the recursion below — taking
        // the lock recursively would risk deadlock against writers.
        //
        // Within a batch (epoch != NO_EPOCH), an entry already revalidated this
        // epoch is trusted with no syscall: the only `file_stamp` (a `metadata`
        // call) is paid on the first hit of the batch. Outside a batch every
        // hit re-stats, preserving the edit-detection behavior single compiles
        // rely on.
        let epoch = current_batch_epoch();
        let mut specifiers: Option<std::vec::Vec<String>> = None;
        if let Ok(cache) = FILE_TYPE_CACHE.read()
            && let Some(entry) = cache.get(&resolved_path)
            && entry.is_fresh(&resolved_path, epoch)
        {
            self.merge_file_summary(&entry.summary);
            specifiers = Some(entry.summary.specifiers.clone());
        }

        let specifiers = match specifiers {
            Some(specifiers) => specifiers,
            None => {
                // Capture the stamp from the same snapshot we parse so the
                // entry is consistent; a concurrent edit just loses the race
                // and re-stamps on the next miss.
                let stamp = file_stamp(&resolved_path);
                let Some(summary) = build_file_summary(&resolved_path) else {
                    return;
                };
                self.merge_file_summary(&summary);
                let specifiers = summary.specifiers.clone();
                if let Ok(mut cache) = FILE_TYPE_CACHE.write() {
                    cache.insert(
                        resolved_path.clone(),
                        CachedFileSummary {
                            stamp,
                            validated_epoch: AtomicU64::new(epoch),
                            summary,
                        },
                    );
                }
                specifiers
            }
        };

        for specifier in &specifiers {
            self.collect_types_from_specifier(specifier, &resolved_path, visited);
        }
    }

    fn merge_file_summary(&mut self, summary: &FileTypeSummary) {
        for (name, body) in &summary.interfaces {
            self.interfaces
                .entry(name.clone())
                .or_insert_with(|| body.clone());
        }
        for (name, body) in &summary.type_aliases {
            self.type_aliases
                .entry(name.clone())
                .or_insert_with(|| body.clone());
        }
    }
}

#[cfg(test)]
mod tests;

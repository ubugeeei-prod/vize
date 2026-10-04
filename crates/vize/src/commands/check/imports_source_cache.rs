//! Reuse lexical scans while keeping disk bytes authoritative on every walk.

use std::path::{Path, PathBuf};

use vize_l0::{FxHashMap, String, profiler::global_profiler};

use super::super::specifiers::{ModuleSpecifierOccurrence, extract_module_specifier_occurrences};

struct SourceOccurrences {
    source: String,
    occurrences: Vec<ModuleSpecifierOccurrence>,
}

#[derive(Default)]
pub(in crate::commands::check::imports) struct SourceOccurrenceCache {
    sources: FxHashMap<PathBuf, SourceOccurrences>,
    reuse: bool,
    #[cfg(test)]
    scans: usize,
    #[cfg(test)]
    hits: usize,
}

impl SourceOccurrenceCache {
    pub(in crate::commands::check::imports) fn with_reuse() -> Self {
        Self {
            reuse: true,
            ..Self::default()
        }
    }

    pub(in crate::commands::check::imports) fn occurrences(
        &mut self,
        path: &Path,
    ) -> std::io::Result<Vec<ModuleSpecifierOccurrence>> {
        // Metadata cannot detect same-length edits whose mtime is restored.
        // Re-read even cached paths and compare the complete source bytes.
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                self.sources.remove(path);
                return Err(error);
            }
        };
        if let Some(cached) = self.sources.get(path)
            && cached.source.as_str() == source
        {
            #[cfg(test)]
            {
                self.hits += 1;
            }
            global_profiler().record_counter("check.import.scan.reused", 1);
            return Ok(cached.occurrences.clone());
        }
        #[cfg(test)]
        {
            self.scans += 1;
        }
        global_profiler().record_counter("check.import.scan.calls", 1);
        let occurrences = extract_module_specifier_occurrences(&source);
        if self.reuse {
            self.sources.insert(
                path.to_path_buf(),
                SourceOccurrences {
                    source: source.into(),
                    occurrences: occurrences.clone(),
                },
            );
        }
        Ok(occurrences)
    }

    #[cfg(test)]
    pub(in crate::commands::check::imports) fn counts(&self) -> (usize, usize) {
        (self.scans, self.hits)
    }
}

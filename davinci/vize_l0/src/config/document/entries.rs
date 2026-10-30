//! Ordered entry projections from an already-deserialized document.

use super::ConfigDocument;
use crate::config::{CompilerConfigEntry, ConfigEntryFiles, ConfigEntryIgnore};

impl ConfigDocument {
    /// Compiler override scopes in declaration order.
    pub fn compiler_entries(&self) -> Vec<CompilerConfigEntry> {
        self.0
            .entries
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(CompilerConfigEntry::from_raw)
            .filter(CompilerConfigEntry::has_overrides)
            .collect()
    }

    /// Top-level ignores precede entry-local ignores, retaining each base path.
    pub fn entry_ignores(&self) -> Vec<ConfigEntryIgnore> {
        self.project_entry_ignores(false)
    }

    /// Formatter discovery excludes only global ignores and ignores-only entries.
    /// Entry-local filters narrow compiler/linter settings without excluding files.
    pub fn formatter_entry_ignores(&self) -> Vec<ConfigEntryIgnore> {
        self.project_entry_ignores(true)
    }

    fn project_entry_ignores(&self, global_only: bool) -> Vec<ConfigEntryIgnore> {
        let top_level_ignores = self
            .0
            .ignores
            .as_deref()
            .unwrap_or_default()
            .iter()
            .cloned()
            .map(|pattern| ConfigEntryIgnore {
                base_path: self.0.project_root.clone(),
                pattern,
            });
        let entry_ignores = self
            .0
            .entries
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter(|entry| !global_only || entry.is_global_ignore())
            .flat_map(|entry| {
                entry
                    .ignores
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .cloned()
                    .map(|pattern| ConfigEntryIgnore {
                        base_path: entry.base_path.clone(),
                        pattern,
                    })
            })
            .collect::<Vec<_>>();
        top_level_ignores.chain(entry_ignores).collect()
    }

    /// Consume nonempty file entries in their original declaration order.
    pub fn into_entry_files(self) -> Vec<ConfigEntryFiles> {
        let mut entries = Vec::new();
        if let Some(files) = self.0.files.filter(|files| !files.is_empty()) {
            entries.push(ConfigEntryFiles {
                base_path: self.0.base_path,
                files,
            });
        }
        entries.extend(
            self.0
                .entries
                .unwrap_or_default()
                .into_iter()
                .filter_map(|entry| {
                    entry
                        .files
                        .filter(|files| !files.is_empty())
                        .map(|files| ConfigEntryFiles {
                            base_path: entry.base_path,
                            files,
                        })
                }),
        );
        entries
    }
}

//! Workspace-wide index of importable Vue components and composables.
//!
//! Foundation for the auto-import code action in #689. When an unknown
//! identifier appears in template or script context the LSP wants to
//! offer "Auto-import from <path>" as a quick fix. That requires a
//! pre-built index of every `.vue` file in the workspace plus every
//! `use*` composable export it can resolve.
//!
//! This module ships the data model and a single `from_directory`
//! constructor. The code-action consumer in
//! `crate::ide::code_action` will be wired up in a follow-up. The
//! intent of landing the index now is so other roadmap milestones
//! (workspace symbols, completion auto-import suggestions) can share
//! the same scan.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use std::path::{Path, PathBuf};

/// Where an importable name comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoImportKind {
    /// Default export from a `.vue` file.
    VueComponent,
    /// Named export from a `.ts` / `.js` file matching `use*` convention.
    Composable,
}

/// One entry in the auto-import index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoImportEntry {
    /// The identifier as it appears in source (`MyButton`, `useCounter`).
    pub name: String,
    /// Absolute path to the file that exports the identifier.
    pub source: PathBuf,
    /// Kind of import (drives the code-action wording).
    pub kind: AutoImportKind,
}

/// Index of importable names discovered during a workspace scan.
#[derive(Debug, Default, Clone)]
pub struct AutoImportIndex {
    entries: Vec<AutoImportEntry>,
}

impl AutoImportIndex {
    /// Build the index by scanning `root` for `.vue` files (treated as
    /// default exports of components) and `use*.ts` / `use*.js` files
    /// (treated as composable exports).
    ///
    /// The scan is shallow on purpose so the foundation costs ~milliseconds
    /// per directory. The full recursive scan, tsconfig path-alias
    /// resolution, and re-export following live behind #689 follow-ups.
    pub fn from_directory(root: &Path) -> Self {
        let mut entries = Vec::new();
        let Ok(read_dir) = std::fs::read_dir(root) else {
            return Self { entries };
        };
        for entry in read_dir.flatten() {
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if let Some(stem) = file_name.strip_suffix(".vue") {
                entries.push(AutoImportEntry {
                    name: pascal_case(stem),
                    source: path,
                    kind: AutoImportKind::VueComponent,
                });
            } else if file_name.starts_with("use") {
                let lowered = file_name.to_ascii_lowercase();
                if lowered.ends_with(".ts") || lowered.ends_with(".js") {
                    let stem = path
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or(file_name);
                    entries.push(AutoImportEntry {
                        name: stem.to_string(),
                        source: path,
                        kind: AutoImportKind::Composable,
                    });
                }
            }
        }
        Self { entries }
    }

    /// Look up every entry matching `name`. Component imports use
    /// PascalCase and composables use the raw export name, so equality
    /// is exact rather than fuzzy at this layer.
    pub fn lookup(&self, name: &str) -> impl Iterator<Item = &AutoImportEntry> {
        self.entries.iter().filter(move |entry| entry.name == name)
    }

    /// Number of indexed entries. Useful for tests and tracing.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when the index has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn pascal_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut capitalize = true;
    for ch in s.chars() {
        if ch == '-' || ch == '_' || ch == '.' {
            capitalize = true;
            continue;
        }
        if capitalize {
            out.extend(ch.to_uppercase());
            capitalize = false;
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{AutoImportIndex, AutoImportKind};

    #[test]
    fn indexes_vue_components_and_composables() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("MyButton.vue"), "").unwrap();
        std::fs::write(dir.path().join("useCounter.ts"), "").unwrap();
        std::fs::write(dir.path().join("random.txt"), "").unwrap();

        let index = AutoImportIndex::from_directory(dir.path());
        assert_eq!(index.len(), 2, "expected 2 entries, got {:?}", index);

        let button = index.lookup("MyButton").next().unwrap();
        assert_eq!(button.kind, AutoImportKind::VueComponent);
        let counter = index.lookup("useCounter").next().unwrap();
        assert_eq!(counter.kind, AutoImportKind::Composable);
    }

    #[test]
    fn empty_directory_yields_empty_index() {
        let dir = tempfile::tempdir().unwrap();
        let index = AutoImportIndex::from_directory(dir.path());
        assert!(index.is_empty());
    }
}

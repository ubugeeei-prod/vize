//! Data types describing the resolved `tsconfig.json` input configuration.

use std::path::{Path, PathBuf};

use glob::Pattern;

use super::glob::{normalize_path_separators, normalize_tsconfig_glob_base};
use super::matching::glob_match_options;

#[derive(Debug, Clone, Default)]
pub(super) struct TsconfigInputSpec {
    pub(super) files: Vec<RelativePathSpec>,
    pub(super) includes: Vec<GlobSpec>,
    pub(super) excludes: Vec<GlobSpec>,
    pub(super) has_files: bool,
    pub(super) has_includes: bool,
    pub(super) has_excludes: bool,
}

impl TsconfigInputSpec {
    pub(super) fn apply_extended(&mut self, extended: Self) {
        if extended.has_files {
            self.files = extended.files;
            self.has_files = true;
        }
        if extended.has_includes {
            self.includes = extended.includes;
            self.has_includes = true;
        }
        if extended.has_excludes {
            self.excludes = extended.excludes;
            self.has_excludes = true;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TsconfigDeclarationOptions {
    pub(crate) declaration_dir: Option<PathBuf>,
    pub(crate) out_dir: Option<PathBuf>,
    pub(crate) declaration_map: Option<bool>,
}

impl TsconfigDeclarationOptions {
    pub(super) fn apply_extended(&mut self, extended: Self) {
        if extended.declaration_dir.is_some() {
            self.declaration_dir = extended.declaration_dir;
        }
        if extended.out_dir.is_some() {
            self.out_dir = extended.out_dir;
        }
        if extended.declaration_map.is_some() {
            self.declaration_map = extended.declaration_map;
        }
    }

    pub(crate) fn output_dir(&self) -> Option<&Path> {
        self.declaration_dir.as_deref().or(self.out_dir.as_deref())
    }
}

#[derive(Debug, Clone)]
pub(super) struct RelativePathSpec {
    base_dir: PathBuf,
    value: std::string::String,
}

impl RelativePathSpec {
    pub(super) fn new(base_dir: &Path, value: &str) -> Self {
        Self {
            base_dir: base_dir.to_path_buf(),
            value: value.replace('\\', "/"),
        }
    }

    pub(super) fn resolve(&self) -> PathBuf {
        self.base_dir.join(&self.value)
    }
}

#[derive(Debug, Clone)]
pub(super) struct GlobSpec {
    pub(super) base_dir: PathBuf,
    pattern: Pattern,
    pub(super) normalized: std::string::String,
}

impl GlobSpec {
    pub(super) fn new(base_dir: &Path, value: &str) -> Option<Self> {
        let (base_dir, normalized) = normalize_tsconfig_glob_base(base_dir, value);
        Pattern::new(&normalized).ok().map(|pattern| Self {
            base_dir,
            pattern,
            normalized,
        })
    }

    pub(super) fn matches(&self, path: &Path) -> bool {
        let Ok(relative) = path.strip_prefix(&self.base_dir) else {
            return false;
        };
        let normalized = normalize_path_separators(relative);
        self.pattern.matches_with(&normalized, glob_match_options())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct FileCollectionOptions {
    pub(super) include_hidden: bool,
    pub(super) include_jsx: bool,
}

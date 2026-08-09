//! Data types describing the resolved `tsconfig.json` input configuration.

use std::path::{Path, PathBuf};

use glob::Pattern;

use super::glob::{normalize_path_separators, normalize_tsconfig_glob_base};
use super::implicit_exclude::wildcard_segment_hits_package_folder;
use super::matching::glob_match_options;

#[derive(Debug, Clone, Default)]
pub(super) struct TsconfigInputSpec {
    pub(super) files: Vec<RelativePathSpec>,
    pub(super) includes: Vec<GlobSpec>,
    pub(super) excludes: Vec<GlobSpec>,
    pub(super) has_files: bool,
    pub(super) has_includes: bool,
    pub(super) has_excludes: bool,
    /// The merged `compilerOptions.allowJs` value for this project.
    pub(super) allow_js: Option<bool>,
    /// `compilerOptions.outDir`, as a glob over the directory it resolves to.
    pub(super) out_dir_exclude: Option<GlobSpec>,
    /// `compilerOptions.declarationDir`, likewise.
    pub(super) declaration_dir_exclude: Option<GlobSpec>,
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
        if extended.allow_js.is_some() {
            self.allow_js = extended.allow_js;
        }
        // Both are ordinary compiler options, so an extending config that does
        // not restate them keeps the base's value.
        if extended.out_dir_exclude.is_some() {
            self.out_dir_exclude = extended.out_dir_exclude;
        }
        if extended.declaration_dir_exclude.is_some() {
            self.declaration_dir_exclude = extended.declaration_dir_exclude;
        }
    }

    /// The `exclude` specs in force for this project.
    ///
    /// `tsc` has no `node_modules` / `bower_components` / `jspm_packages`
    /// default `exclude`: it rejects those three names from *wildcard* `include`
    /// segments instead (see [`super::implicit_exclude`]), which is why a
    /// literal include segment naming one of them stays a program root. The real
    /// default is `[outDir, declarationDir]`, applied only when neither the
    /// config nor anything it extends declares an `exclude` of its own — an
    /// explicit `exclude`, including `[]`, replaces it wholesale (#3395).
    pub(super) fn effective_excludes(&self) -> Vec<GlobSpec> {
        if self.has_excludes {
            return self.excludes.clone();
        }
        self.out_dir_exclude
            .iter()
            .chain(self.declaration_dir_exclude.iter())
            .cloned()
            .collect()
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

    /// Whether `path` matches this spec as a plain glob. This is the `exclude`
    /// semantics: TypeScript's exclude matcher has no implicit exclusions.
    pub(super) fn matches(&self, path: &Path) -> bool {
        self.matched_relative(path).is_some()
    }

    /// Whether `path` matches this spec as an `include` entry, i.e. as a plain
    /// glob *and* without a package folder in any wildcard-matched segment. See
    /// [`super::implicit_exclude`] for why `include` and `exclude` differ here.
    pub(super) fn matches_include(&self, path: &Path) -> bool {
        self.matched_relative(path).is_some_and(|relative| {
            !wildcard_segment_hits_package_folder(&self.normalized, relative)
        })
    }

    fn matched_relative<'path>(&self, path: &'path Path) -> Option<&'path Path> {
        let relative = path.strip_prefix(&self.base_dir).ok()?;
        let normalized = normalize_path_separators(relative);
        self.pattern
            .matches_with(&normalized, glob_match_options())
            .then_some(relative)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct FileCollectionOptions {
    pub(super) include_hidden: bool,
    pub(super) include_js: bool,
    pub(super) include_jsx: bool,
}

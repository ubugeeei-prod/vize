//! TypeScript project-reference ownership: which project of a solution owns a
//! source.
//!
//! `tsc --build`, tsserver and `vue-tsc` read a solution-style `tsconfig.json`
//! (`files: []` plus `references`, the create-vue layout) as a graph and give
//! each source the compiler options of the referenced project whose effective
//! `files` / `include` / `exclude` admits it. Compiler, checker, linter and
//! editor consumers share this one interpretation: a source owned by exactly
//! one project of the graph takes that project, and an unowned or ambiguous
//! source keeps the root.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::typescript_project::normalize_path_lexically;

mod graph;
mod wildcard;
pub use graph::ProjectReferenceGraph;
pub use wildcard::{TsconfigWildcard, TsconfigWildcardUsage};

/// Whether membership also requires the owning project to enable `allowJs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsconfigSourceKind {
    /// Vue files and the TypeScript family.
    Typed,
    /// `.js`, `.jsx`, `.mjs`, and `.cjs` inputs.
    JavaScript,
}

impl TsconfigSourceKind {
    /// The membership kind implied by a source's extension.
    pub fn for_path(path: &Path) -> Self {
        match path.extension().and_then(|extension| extension.to_str()) {
            Some("js" | "jsx" | "mjs" | "cjs") => Self::JavaScript,
            _ => Self::Typed,
        }
    }
}

/// The effective `files` / `include` / `exclude` of one config after its
/// `extends` chain, each anchored at the config that declared it.
#[derive(Clone, Debug)]
pub struct OwnershipSpec {
    /// `**/*` under the config directory, used when no `include` is inherited.
    default_include: Option<TsconfigWildcard>,
    files: Option<Vec<PathBuf>>,
    includes: Option<Vec<TsconfigWildcard>>,
    excludes: Option<Vec<TsconfigWildcard>>,
    out_dir: Option<Option<TsconfigWildcard>>,
    declaration_dir: Option<Option<TsconfigWildcard>>,
    allow_js: Option<bool>,
}

impl OwnershipSpec {
    /// Build the spec of one parsed config. `config_path` is already passed
    /// through [`normalize_path`]; `extended` holds the specs of its resolved
    /// `extends` entries in declaration order, later entries overriding
    /// earlier ones and the config itself overriding them all.
    pub fn from_config(
        config_path: &Path,
        config: &Value,
        extended: impl IntoIterator<Item = Self>,
    ) -> Self {
        let base = config_path.parent().unwrap_or(Path::new("."));
        let mut effective = Self {
            default_include: TsconfigWildcard::new(base, "**/*", TsconfigWildcardUsage::Include),
            files: None,
            includes: None,
            excludes: None,
            out_dir: None,
            declaration_dir: None,
            allow_js: None,
        };
        for parent in extended {
            effective.apply_extended(parent);
        }

        if let Some(files) = string_array(config, "files") {
            effective.files = Some(
                files
                    .into_iter()
                    .map(|file| normalize_path(&base.join(file)))
                    .collect(),
            );
        }
        for (name, usage, target) in [
            (
                "include",
                TsconfigWildcardUsage::Include,
                &mut effective.includes,
            ),
            (
                "exclude",
                TsconfigWildcardUsage::Exclude,
                &mut effective.excludes,
            ),
        ] {
            if let Some(specs) = string_array(config, name) {
                *target = Some(
                    specs
                        .into_iter()
                        .filter_map(|spec| TsconfigWildcard::new(base, spec, usage))
                        .collect(),
                );
            }
        }

        let compiler_options = config.get("compilerOptions").and_then(Value::as_object);
        let own_allow_js = compiler_options.and_then(|options| options.get("allowJs"));
        if config_path
            .file_name()
            .is_some_and(|name| name == "jsconfig.json")
        {
            effective.allow_js = match own_allow_js {
                None => Some(true),
                Some(value) => value.as_bool(),
            };
        } else if let Some(allow_js) = own_allow_js.and_then(Value::as_bool) {
            effective.allow_js = Some(allow_js);
        }
        for (name, target) in [
            ("outDir", &mut effective.out_dir),
            ("declarationDir", &mut effective.declaration_dir),
        ] {
            if let Some(output_dir) = compiler_options
                .and_then(|options| options.get(name))
                .and_then(Value::as_str)
            {
                // `tsc` skips an empty directory; its default exclude is the raw
                // option value.
                *target = Some(
                    (!output_dir.is_empty())
                        .then(|| {
                            TsconfigWildcard::new(base, output_dir, TsconfigWildcardUsage::Exclude)
                        })
                        .flatten(),
                );
            }
        }
        effective
    }

    /// The inherited `compilerOptions.allowJs`, when any config declares it.
    pub fn allow_js(&self) -> Option<bool> {
        self.allow_js
    }

    /// Membership of a source met under any of its [`source_spellings`].
    pub fn includes_any(
        &self,
        spellings: &[PathBuf],
        case_sensitive: bool,
        source_kind: TsconfigSourceKind,
    ) -> bool {
        spellings
            .iter()
            .any(|spelling| self.includes_normalized(spelling, case_sensitive, source_kind))
    }

    /// Membership of a path already passed through [`normalize_path`].
    pub fn includes_normalized(
        &self,
        source_path: &Path,
        case_sensitive: bool,
        source_kind: TsconfigSourceKind,
    ) -> bool {
        if self.files.as_ref().is_some_and(|files| {
            files
                .iter()
                .any(|file| paths_equal(file, source_path, case_sensitive))
        }) {
            return true;
        }
        if source_kind == TsconfigSourceKind::JavaScript && self.allow_js != Some(true) {
            return false;
        }

        let included = match self.includes.as_ref() {
            Some(includes) => includes
                .iter()
                .any(|include| include.matches(source_path, case_sensitive)),
            None if self.files.is_some() => false,
            None => self
                .default_include
                .as_ref()
                .is_some_and(|include| include.matches(source_path, case_sensitive)),
        };
        if !included {
            return false;
        }

        let excluded = match self.excludes.as_ref() {
            Some(excludes) => excludes
                .iter()
                .any(|exclude| exclude.matches(source_path, case_sensitive)),
            None => self
                .out_dir
                .as_ref()
                .and_then(Option::as_ref)
                .into_iter()
                .chain(self.declaration_dir.as_ref().and_then(Option::as_ref))
                .any(|exclude| exclude.matches(source_path, case_sensitive)),
        };
        !excluded
    }

    fn apply_extended(&mut self, extended: Self) {
        if extended.files.is_some() {
            self.files = extended.files;
        }
        if extended.includes.is_some() {
            self.includes = extended.includes;
        }
        if extended.excludes.is_some() {
            self.excludes = extended.excludes;
        }
        if extended.out_dir.is_some() {
            self.out_dir = extended.out_dir;
        }
        if extended.declaration_dir.is_some() {
            self.declaration_dir = extended.declaration_dir;
        }
        if extended.allow_js.is_some() {
            self.allow_js = extended.allow_js;
        }
    }
}

/// The `extends` entries of one parsed config, in declaration order.
pub fn extends_entries(config: &Value) -> Vec<&str> {
    match config.get("extends") {
        Some(Value::String(parent)) => vec![parent.as_str()],
        Some(Value::Array(parents)) => parents.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

/// The `references[].path` entries of one parsed config, in declaration order.
pub fn reference_entries(config: &Value) -> Vec<&str> {
    config
        .get("references")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.get("path").and_then(Value::as_str))
        .collect()
}

/// The files a `references[].path` may name, in TypeScript's probe order: the
/// path itself when it has an extension, otherwise the path, the path with
/// `.json`, and `tsconfig.json` inside it as a directory.
pub fn reference_config_candidates(config_path: &Path, reference: &str) -> Vec<PathBuf> {
    let base_dir = config_path.parent().unwrap_or(Path::new("."));
    let reference = Path::new(reference);
    let base = if reference.is_absolute() {
        reference.to_path_buf()
    } else {
        base_dir.join(reference)
    };
    let candidates = if base.extension().is_some() {
        vec![base]
    } else {
        vec![
            base.clone(),
            base.with_extension("json"),
            base.join("tsconfig.json"),
        ]
    };
    candidates
        .into_iter()
        .map(|candidate| normalize_path_lexically(&candidate))
        .collect()
}

/// The unique owner among `projects`, or `root` when none or several own it.
pub fn unique_owner<'a>(
    root: &Path,
    projects: impl IntoIterator<Item = &'a PathBuf>,
    mut owns: impl FnMut(&Path) -> bool,
) -> PathBuf {
    let mut owner = None;
    for project in projects {
        if !owns(project) {
            continue;
        }
        if owner.is_some() {
            return root.to_path_buf();
        }
        owner = Some(project);
    }
    owner.cloned().unwrap_or_else(|| root.to_path_buf())
}

/// The path spelling ownership compares: lexically normalized, then
/// canonicalized (symlinks resolved) without a Windows verbatim prefix.
pub fn normalize_path(path: &Path) -> PathBuf {
    crate::path::canonicalize_non_verbatim(&normalize_path_lexically(path))
}

/// Every spelling under which a project's specs may meet `source`.
///
/// Configs are compared by their [`normalize_path`] spelling, but a spec
/// keeps the literal directories of an `include` below that config as
/// written, as TypeScript's directory walk spells them. A source is
/// therefore matched with each symlinked ancestor resolved in turn and the
/// components below it kept as written: a symlinked `src` (or a symlinked
/// directory under `**`) still lies under `<config dir>/src`, while a
/// symlinked ancestor of the config directory, such as macOS `/var`,
/// meets the canonical config spelling. The last spelling is the fully
/// canonical one, which `files` entries use. Without symlinks this is one
/// spelling, the source itself.
pub fn source_spellings(source: &Path) -> Vec<PathBuf> {
    // A relative source is spelled from the working directory, as
    // canonicalization would resolve it.
    let lexical = normalize_path_lexically(&if source.is_absolute() {
        source.to_path_buf()
    } else {
        std::env::current_dir().map_or_else(|_| source.to_path_buf(), |cwd| cwd.join(source))
    });
    let mut spellings = vec![lexical.clone()];
    let mut prefix = PathBuf::new();
    for component in lexical.components() {
        prefix.push(component);
        crate::micro_count!("fs.symlink_metadata.calls", 1);
        let is_symlink = std::fs::symlink_metadata(&prefix)
            .is_ok_and(|metadata| metadata.file_type().is_symlink());
        if !is_symlink {
            continue;
        }
        let resolved = normalize_path(&prefix);
        let spelling = match lexical.strip_prefix(&prefix) {
            Ok(rest) if !rest.as_os_str().is_empty() => resolved.join(rest),
            _ => resolved,
        };
        if !spellings.contains(&spelling) {
            spellings.push(spelling);
        }
    }
    spellings
}

/// Match TypeScript's host policy: Windows is insensitive; other hosts probe
/// the filesystem containing the running checker by swapping the executable
/// path's ASCII case. This preserves case-sensitive macOS volumes instead of
/// hard-coding an operating-system guess.
pub fn host_uses_case_sensitive_file_names() -> bool {
    if cfg!(windows) {
        return false;
    }
    let Ok(executable) = std::env::current_exe() else {
        return !cfg!(target_os = "macos");
    };
    let spelled = executable.to_string_lossy();
    let swapped = spelled
        .chars()
        .map(|character| {
            if character.is_ascii_lowercase() {
                character.to_ascii_uppercase()
            } else {
                character.to_ascii_lowercase()
            }
        })
        .collect::<crate::String>();
    swapped == spelled || !Path::new(swapped.as_str()).exists()
}

fn string_array<'a>(config: &'a Value, name: &str) -> Option<Vec<&'a str>> {
    config
        .get(name)?
        .as_array()
        .map(|entries| entries.iter().filter_map(Value::as_str).collect())
}

fn paths_equal(left: &Path, right: &Path, case_sensitive: bool) -> bool {
    relative_to(left, right, case_sensitive).is_some_and(|path| path.as_os_str().is_empty())
}

/// Both paths are already [`normalize_path`] results: specs normalize their
/// bases and files once at load, and callers normalize each source once.
fn relative_to(path: &Path, base: &Path, case_sensitive: bool) -> Option<PathBuf> {
    let mut path_parts = path.components();
    for base_part in base.components() {
        let path_part = path_parts.next()?;
        if !names_equal(path_part.as_os_str(), base_part.as_os_str(), case_sensitive) {
            return None;
        }
    }
    Some(path_parts.map(|part| part.as_os_str()).collect())
}

fn names_equal(left: &std::ffi::OsStr, right: &std::ffi::OsStr, case_sensitive: bool) -> bool {
    if case_sensitive {
        left == right
    } else {
        left.to_string_lossy().to_lowercase() == right.to_string_lossy().to_lowercase()
    }
}

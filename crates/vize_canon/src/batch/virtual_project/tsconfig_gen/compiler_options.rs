//! Loading and flattening effective `compilerOptions` from a tsconfig chain.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use vize_carton::{FxHashMap, FxHashSet, profile};

use crate::batch::error::CorsaResult;

use super::super::VirtualProject;
use super::super::tsconfig_paths::{
    normalize_path_lexically, normalize_tsconfig_path_target, parse_jsonc_value,
    resolve_extended_tsconfig_path,
};
use super::path_rebase;

#[derive(Clone, Copy)]
enum PathOptions {
    Rebase,
    Verbatim,
}

/// Directories of the configs whose `paths` and `baseUrl` declarations survived
/// the `extends` merge.
///
/// TypeScript resolves a relative `paths` target against the effective
/// `baseUrl` when one is declared anywhere in the chain, and against the
/// directory of the config declaring the winning `paths` map otherwise.
/// `extends` merging replaces whole values, so each anchor is simply the
/// directory of the config whose declaration won.
#[derive(Clone, Default)]
struct DeclarationDirs {
    paths: Option<PathBuf>,
    base_url: Option<PathBuf>,
}

impl DeclarationDirs {
    fn absorb_overriding(&mut self, overriding: DeclarationDirs) {
        if overriding.paths.is_some() {
            self.paths = overriding.paths;
        }
        if overriding.base_url.is_some() {
            self.base_url = overriding.base_url;
        }
    }
}

/// A flattened `extends` chain: the effective options plus the directories the
/// surviving `paths` and `baseUrl` declarations came from.
#[allow(clippy::disallowed_types)]
type FlattenedChain = (Map<std::string::String, Value>, DeclarationDirs);

/// The state of one flattening operation.
#[derive(Default)]
struct ChainLoad {
    /// The configs between the entry point and the one being flattened. Only
    /// these can close a cycle.
    active: FxHashSet<PathBuf>,
    /// Chains already flattened during this operation. A config reachable
    /// through several `extends` paths is read, parsed and merged once instead
    /// of once per path, which keeps a diamond graph linear rather than
    /// exponential in its depth.
    completed: FxHashMap<PathBuf, FlattenedChain>,
    /// Whether a cycle was cut while flattening the current chain. That result
    /// depends on which configs happened to be active, so it is not reusable
    /// from anywhere else in the graph and must not be cached.
    cycle_cut: bool,
}

/// Flattened options plus the effective `baseUrl` (#3886).
#[allow(clippy::disallowed_types)]
#[derive(Default)]
pub(in super::super) struct FlattenedCompilerOptions {
    pub(in super::super) options: Map<std::string::String, Value>,
    /// The effective `baseUrl` rebased the way `paths` targets are: relative to
    /// the project root without a `./` prefix (empty for the root itself), or
    /// absolute when it escapes the root. `None` when no config declares one.
    pub(in super::super) base_url: Option<vize_carton::String>,
    /// Every config file actually read while flattening this chain.
    pub(in super::super) input_paths: Vec<PathBuf>,
}

impl VirtualProject {
    #[allow(clippy::disallowed_types)]
    pub(in super::super) fn load_compiler_options(
        &self,
        tsconfig_path: Option<&Path>,
    ) -> CorsaResult<Map<std::string::String, Value>> {
        Ok(self.load_compiler_options_flattened(tsconfig_path)?.options)
    }

    /// Flatten the effective options without changing path spellings. The
    /// input-less option probe validates syntax only, so its paths stay inert.
    #[allow(clippy::disallowed_types)]
    pub(in super::super) fn load_compiler_options_verbatim(
        &self,
        tsconfig_path: Option<&Path>,
    ) -> CorsaResult<Map<std::string::String, Value>> {
        let Some(tsconfig_path) = tsconfig_path else {
            return Ok(Map::new());
        };
        let mut load = ChainLoad::default();
        Ok(self
            .load_compiler_options_inner(tsconfig_path, &mut load, PathOptions::Verbatim)?
            .0)
    }

    /// Flatten the chain and rebase `paths` targets onto the project root from
    /// the anchor TypeScript actually uses: the effective `baseUrl` when one is
    /// declared, the winning `paths` map's declaring directory otherwise.
    #[allow(clippy::disallowed_types)]
    pub(in super::super) fn load_compiler_options_flattened(
        &self,
        tsconfig_path: Option<&Path>,
    ) -> CorsaResult<FlattenedCompilerOptions> {
        let Some(tsconfig_path) = tsconfig_path else {
            return Ok(FlattenedCompilerOptions {
                options: Map::new(),
                base_url: None,
                input_paths: Vec::new(),
            });
        };
        let mut load = ChainLoad::default();
        let (mut options, dirs) =
            self.load_compiler_options_inner(tsconfig_path, &mut load, PathOptions::Rebase)?;

        let raw_base_url = options
            .get("baseUrl")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let base_url = match (&raw_base_url, &dirs.base_url) {
            (Some(raw), Some(dir)) => {
                Some(normalize_tsconfig_path_target(dir, &self.project_root, raw))
            }
            _ => None,
        };

        let paths_anchor = match (&raw_base_url, &dirs.base_url) {
            (Some(raw), Some(dir)) => Some(normalize_path_lexically(&dir.join(raw))),
            _ => dirs.paths,
        };
        if let (Some(anchor), Some(paths)) = (
            paths_anchor.as_deref(),
            options.get_mut("paths").and_then(Value::as_object_mut),
        ) {
            path_rebase::paths_onto_project_root(paths, anchor, &self.project_root);
        }

        let mut input_paths = load.completed.keys().cloned().collect::<Vec<_>>();
        input_paths.push(normalize_path_lexically(tsconfig_path));
        input_paths.sort();
        input_paths.dedup();
        Ok(FlattenedCompilerOptions {
            options,
            base_url,
            input_paths,
        })
    }

    /// `load.active` is the *active* `extends` path, not a set of everything
    /// already loaded: each config is removed again once its own chain is
    /// flattened. Only a config currently being recursed into is a cycle. A
    /// visited set would also short-circuit an ancestor two sibling `extends`
    /// entries share, handing the later sibling nothing and leaving the earlier
    /// one's overrides in place where TypeScript gives the later sibling's
    /// inherited values. Repeated work is avoided by `load.completed` instead,
    /// which reuses the flattened chain rather than suppressing it.
    #[allow(clippy::disallowed_types)]
    fn load_compiler_options_inner(
        &self,
        tsconfig_path: &Path,
        load: &mut ChainLoad,
        path_options: PathOptions,
    ) -> CorsaResult<FlattenedChain> {
        if !tsconfig_path.exists() {
            return Ok((Map::new(), DeclarationDirs::default()));
        }
        let normalized = normalize_path_lexically(tsconfig_path);
        if let Some(cached) = load.completed.get(&normalized) {
            return Ok(cached.clone());
        }
        if !load.active.insert(normalized.clone()) {
            load.cycle_cut = true;
            return Ok((Map::new(), DeclarationDirs::default()));
        }
        let enclosing_cycle_cut = std::mem::replace(&mut load.cycle_cut, false);
        let flattened = self.load_extended_compiler_options(&normalized, load, path_options);
        load.active.remove(&normalized);
        let chain_cycle_cut = load.cycle_cut;
        load.cycle_cut = enclosing_cycle_cut || chain_cycle_cut;
        if !chain_cycle_cut && let Ok(flattened) = &flattened {
            load.completed.insert(normalized, flattened.clone());
        }
        flattened
    }

    /// The chain rooted at an already-normalized config, with `load.active`
    /// holding the configs between it and the entry point.
    #[allow(clippy::disallowed_types)]
    fn load_extended_compiler_options(
        &self,
        normalized: &Path,
        load: &mut ChainLoad,
        path_options: PathOptions,
    ) -> CorsaResult<FlattenedChain> {
        let content = profile!("canon.tsconfig.read", std::fs::read_to_string(normalized))?;
        let config = profile!("canon.tsconfig.parse", parse_jsonc_value(&content))?;
        let mut compiler_options = config
            .get("compilerOptions")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let base_dir = normalized.parent().unwrap_or(self.project_root.as_path());
        let mut dirs = DeclarationDirs::default();
        if compiler_options.contains_key("paths") {
            dirs.paths = Some(base_dir.to_path_buf());
        }
        if compiler_options.contains_key("baseUrl") {
            dirs.base_url = Some(base_dir.to_path_buf());
        }
        if matches!(path_options, PathOptions::Rebase) {
            path_rebase::onto_project_root(&mut compiler_options, base_dir, &self.project_root);
        }

        // `extends` may be a single specifier or an array; array entries are
        // applied in order, with later entries overriding earlier ones, and
        // the extending file overriding them all.
        let mut inherited = Map::new();
        let mut inherited_dirs = DeclarationDirs::default();
        match config.get("extends") {
            Some(Value::String(extends)) => {
                if let Some(parent_path) = resolve_extended_tsconfig_path(normalized, extends) {
                    let (parent, parent_dirs) =
                        self.load_compiler_options_inner(&parent_path, load, path_options)?;
                    inherited = parent;
                    inherited_dirs = parent_dirs;
                }
            }
            Some(Value::Array(entries)) => {
                for extends in entries.iter().filter_map(Value::as_str) {
                    if let Some(parent_path) = resolve_extended_tsconfig_path(normalized, extends) {
                        let (parent, parent_dirs) =
                            self.load_compiler_options_inner(&parent_path, load, path_options)?;
                        inherited.extend(parent);
                        inherited_dirs.absorb_overriding(parent_dirs);
                    }
                }
            }
            _ => {}
        }
        inherited_dirs.absorb_overriding(dirs);
        if inherited.is_empty() {
            return Ok((compiler_options, inherited_dirs));
        }

        inherited.extend(compiler_options);
        Ok((inherited, inherited_dirs))
    }
}

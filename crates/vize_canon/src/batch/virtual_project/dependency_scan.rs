//! Reachability registration for out-of-root workspace `.vue` files (#3887).
//!
//! `register_paths` covers the scanned roots; an import that resolves to a
//! `.vue` outside them previously fell back to the ambient `*.vue` stub, so
//! its props and emits silently stopped being checked. This pass walks the
//! import graph of everything registered and registers reachable first-party
//! files — a `.vue`, or a TypeScript file that can re-export one (a barrel) —
//! to a fixpoint. Published packages are left alone: a canonical path that
//! stays inside `node_modules` keeps the stub (that half is #3282), while a
//! pnpm workspace symlink canonicalizes *out* of `node_modules` and is
//! first-party source. Batch projects leave declaration files alone so their
//! ambient declarations remain governed by the tsconfig. Editor sessions do
//! mirror reachable declarations, preserving their authored `.d.ts` spelling,
//! but keep them inferred rather than adding them as ambient program roots
//! (see [`is_declaration_file`]).
//!
//! Specifiers are collected from the *generated* content (always valid TS,
//! one collector for `.vue` and script files alike) and resolved against the
//! *original* file's directory; a `.vue.ts` spelling the import rewriter
//! produced is folded back to `.vue` first. Resolution covers relative
//! specifiers and tsconfig `paths` aliases — the shapes the monorepo defect
//! reproduces with; bare workspace-package specifiers resolve only through
//! their `paths` alias today.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
#[cfg(test)]
use vize_carton::cstr;
use vize_carton::{FxHashMap, FxHashSet, String as CompactString};

use crate::batch::error::CorsaResult;

use super::VirtualProject;

#[path = "dependency_scan/resolution.rs"]
mod resolution;
#[cfg(test)]
use resolution::probe_candidates;
pub(crate) use resolution::resolve_dependency;
use resolution::{
    alias_may_reach_first_party, canonical_key, inside_node_modules, is_declaration_file,
    may_resolve_a_dependency,
};

type PackageResolver<'a> = &'a mut dyn FnMut(&Path, &str, crate::PackageResolutionMode) -> bool;

#[allow(clippy::disallowed_types)]
impl VirtualProject {
    /// Register every reachable first-party dependency, to a fixpoint.
    pub fn register_reachable_dependencies(&mut self) -> CorsaResult<()> {
        self.register_reachable_dependencies_with_overlays(&FxHashMap::default())
    }

    /// Reconcile only dependencies reachable from sources rebuilt by one
    /// persistent patch. Existing edge ownership covers the untouched graph.
    pub(crate) fn register_reachable_dependencies_from(
        &mut self,
        sources: &[PathBuf],
    ) -> CorsaResult<()> {
        self.register_reachable_dependencies_inner(Some(sources), &FxHashMap::default(), &[], None)
    }

    /// The same walk, with unsaved editor buffers standing in for their on-disk
    /// contents.
    ///
    /// An editor session must see the buffer the user is typing in: a dependency
    /// reachable only through an import that exists in an unsaved file is
    /// invisible to a disk-only walk, so nothing registers it, its mirror
    /// companion is never generated, and the alias rewrite has no real file to
    /// point at until the buffer is saved (#3900).
    pub(crate) fn register_reachable_dependencies_with_overlays(
        &mut self,
        overlays: &FxHashMap<PathBuf, &str>,
    ) -> CorsaResult<()> {
        self.register_reachable_dependencies_inner(None, overlays, &[], None)
    }

    pub(crate) fn register_reachable_dependencies_with_package_resolver(
        &mut self,
        overlays: &FxHashMap<PathBuf, &str>,
        workspace_package_specifiers: &[CompactString],
        package_resolver: PackageResolver<'_>,
    ) -> CorsaResult<()> {
        self.register_reachable_dependencies_inner(
            None,
            overlays,
            workspace_package_specifiers,
            Some(package_resolver),
        )
    }

    fn register_reachable_dependencies_inner(
        &mut self,
        initial_sources: Option<&[PathBuf]>,
        overlays: &FxHashMap<PathBuf, &str>,
        workspace_package_specifiers: &[CompactString],
        mut package_resolver: Option<PackageResolver<'_>>,
    ) -> CorsaResult<()> {
        let aliases = self.dependency_alias_map();
        let alias_prefixes: Vec<CompactString> = aliases
            .iter()
            .filter(|(pattern, target)| {
                alias_may_reach_first_party(pattern, target, &self.project_root)
            })
            .map(|(pattern, _)| CompactString::from(pattern.trim_end_matches('*')))
            .collect();
        let mut queue: Vec<PathBuf> = match initial_sources {
            Some(paths) => paths
                .iter()
                .filter(|path| self.find_by_original(path).is_some())
                .cloned()
                .collect(),
            None => self
                .virtual_files_sorted()
                .iter()
                .map(|file| file.original_path.clone())
                .collect(),
        };
        let mut visited: FxHashSet<PathBuf> = queue
            .iter()
            .filter_map(|path| canonical_key(path))
            .collect();

        while let Some(importer) = queue.pop() {
            let Some((virtual_content, virtual_path)) = self
                .find_by_original(&importer)
                .map(|file| (file.content.clone(), file.virtual_path.clone()))
            else {
                continue;
            };
            let mut dependency_targets = FxHashSet::default();
            if !may_resolve_a_dependency(
                &virtual_content,
                &alias_prefixes,
                workspace_package_specifiers,
            ) {
                let released = self.replace_dependency_edges(&importer, dependency_targets);
                self.prune_unowned_sources(released);
                continue;
            }
            let Some(importer_dir) = importer.parent().map(Path::to_path_buf) else {
                continue;
            };
            let source_type = if virtual_path
                .extension()
                .is_some_and(|extension| extension == "tsx")
            {
                SourceType::tsx()
            } else {
                SourceType::ts()
            };
            let specifiers = self
                .rewriter()
                .collect_all_specifier_occurrences(&virtual_content, source_type);
            // Package-local edges only exist inside a package root that already
            // contains this importer, so scan the route table once per importer
            // instead of once per specifier (#4137).
            let importer_package_roots = self
                .package_routes
                .values()
                .filter_map(|binding| binding.route.as_ref())
                .flat_map(crate::PackageRoute::all_routes)
                .filter(|route| importer.starts_with(&route.package_root))
                .map(|route| route.package_root.clone())
                .collect::<Vec<_>>();

            for (specifier, mode) in specifiers {
                let native_target =
                    resolve_dependency(&specifier, &importer_dir, &self.project_root, &aliases);
                if native_target.is_none() {
                    if let Some(resolve) = package_resolver.as_deref_mut() {
                        let _ = resolve(&importer, &specifier, mode);
                    }
                    continue;
                }
                let target = native_target.expect("checked above");
                let Some(key) = canonical_key(&target) else {
                    continue;
                };
                let package_local = importer_package_roots
                    .iter()
                    .any(|package_root| key.starts_with(package_root));
                if inside_node_modules(&key) && !package_local {
                    continue;
                }
                if is_declaration_file(&key) && !package_local && !self.session_scripts {
                    continue;
                }
                // Only `.vue` files gain anything from registration — their
                // generated companion is what consumers resolve. A script
                // registers only when it lives *outside* the project root (a
                // workspace barrel whose `.vue` re-export must be rewritten in
                // mirror space); in-root scripts are the scan collector's job,
                // and force-registering them would change the scanned set that
                // incremental sessions and Tier-L pin (#3898).
                let is_vue = key.extension().is_some_and(|extension| extension == "vue");
                if !is_vue
                    && !self.session_scripts
                    && key.starts_with(&self.project_root)
                    && !package_local
                {
                    continue;
                }
                dependency_targets.insert(key.clone());
                if !visited.insert(key.clone()) {
                    continue;
                }
                // Register the canonical path: a workspace symlink is
                // first-party where it actually lives, so it must not enter the
                // virtual tree under `node_modules`.
                // A reachable dependency is inferred, not requested: an
                // unreadable file or a malformed sibling-package SFC must not
                // abort the check the user actually asked for, so registration
                // failure degrades to the pre-#3887 ambient stub for that one
                // import instead of propagating (#3898).
                let registered = match overlays.get(&key) {
                    Some(content) => self.register_path_with_content(&key, content),
                    None => self.register_path(&key),
                };
                if registered.is_ok() {
                    queue.push(key);
                }
            }
            let released = self.replace_dependency_edges(&importer, dependency_targets);
            self.prune_unowned_sources(released);
        }
        Ok(())
    }

    /// The effective `paths` aliases with project-root-relative targets, as
    /// (pattern, target) pairs. Both come from the flattened chain, so the
    /// anchors match what the generated tsconfig resolves (#3886).
    /// Opt an editor session into registering reachable in-root scripts (#3915).
    pub(crate) fn set_session_script_registration(&mut self, enabled: bool) {
        self.session_scripts = enabled;
    }

    pub(crate) fn dependency_alias_map(&self) -> Vec<(String, String)> {
        let anchored = self.resolved_tsconfig_path();
        let aliases = self.alias_map_of(anchored.as_deref());
        if !aliases.is_empty() {
            return aliases;
        }
        // A solution-style shell (create-vue's default) declares nothing
        // itself; the first referenced config that yields paths wins — the
        // standard app/node split has exactly one (#3915).
        let Some(anchored) = anchored else {
            return aliases;
        };
        for referenced in super::tsconfig_gen::references::referenced_project_configs(&anchored) {
            let referenced_aliases = self.alias_map_of(Some(&referenced));
            if !referenced_aliases.is_empty() {
                return referenced_aliases;
            }
        }
        aliases
    }

    fn alias_map_of(&self, tsconfig_path: Option<&Path>) -> Vec<(String, String)> {
        let Ok(flattened) = self.load_compiler_options_flattened(tsconfig_path) else {
            return Vec::new();
        };
        let Some(paths) = flattened
            .options
            .get("paths")
            .and_then(serde_json::Value::as_object)
        else {
            return Vec::new();
        };
        let mut aliases = Vec::new();
        for (pattern, targets) in paths {
            let Some(targets) = targets.as_array() else {
                continue;
            };
            for target in targets.iter().filter_map(serde_json::Value::as_str) {
                aliases.push((pattern.clone(), target.to_owned()));
            }
        }
        aliases
    }
}

#[cfg(test)]
#[path = "dependency_scan_tests.rs"]
mod tests;

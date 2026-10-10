//! Reachability registration for out-of-root workspace `.vue` files (#3887).
//!
//! Walk scanned roots to a fixpoint, registering reachable first-party Vue
//! files and script barrels (#3887). Canonical `node_modules` targets retain
//! the ambient stub (#3282); workspace symlinks outside it are first-party.
//! Batch leaves declarations to tsconfig; editor sessions mirror reachable
//! declarations with authored spelling and inferred ownership (see
//! [`is_declaration_file`]).
//!
//! Collect specifiers from valid generated TS and resolve them relative to
//! the original file, folding `.vue.ts` back to `.vue`. Relative imports and
//! tsconfig `paths` aliases are resolved; bare workspace packages require aliases.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
#[cfg(test)]
use vize_carton::cstr;
use vize_carton::{FxHashMap, FxHashSet, String as CompactString};

use crate::batch::error::CorsaResult;

use super::VirtualProject;

mod package_roots;
#[path = "dependency_scan/references.rs"]
mod references;
#[path = "dependency_scan/resolution.rs"]
mod resolution;
#[cfg(test)]
use resolution::probe_candidates;
use resolution::{
    alias_may_reach_first_party, canonical_key, inside_node_modules, is_declaration_file,
    may_resolve_a_dependency,
};
pub(crate) use resolution::{resolve_dependency, resolve_dependency_with_inputs};

type PackageResolver<'a> = &'a mut dyn FnMut(&Path, &str, crate::PackageResolutionMode) -> bool;

#[expect(clippy::disallowed_types, reason = "dependency API uses std String")]
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
            .map(|(pattern, _)| {
                CompactString::from(
                    pattern
                        .split_once('*')
                        .map_or(pattern.as_str(), |(prefix, _)| prefix),
                )
            })
            .collect();
        let declaration_alias_prefixes = aliases
            .iter()
            .map(|(pattern, _)| {
                CompactString::from(
                    pattern
                        .split_once('*')
                        .map_or(pattern.as_str(), |(prefix, _)| prefix),
                )
            })
            .collect::<Vec<_>>();
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
        // Explicit patch importers retain current target graphs; full walks
        // keep their original queue-derived lookup identities.
        let mut visited: FxHashSet<PathBuf> = if initial_sources.is_some() {
            self.registered_original_paths_sorted()
                .iter()
                .filter_map(|path| canonical_key(path))
                .collect()
        } else {
            queue
                .iter()
                .filter_map(|path| canonical_key(path))
                .collect()
        };
        let (raw_roots_indexed, checked_roots) = self.raw_package_roots_are_indexed();
        let mut indexed_package_root_lookups = 0usize;
        let mut fallback_package_root_lookups = 0usize;

        while let Some(importer) = queue.pop() {
            let Some((virtual_content, virtual_path, generated_sfc)) =
                self.find_by_original(&importer).map(|file| {
                    (
                        self.module_source(file).unwrap_or(&file.content).clone(),
                        file.virtual_path.clone(),
                        file.source_map.sfc_map.is_some(),
                    )
                })
            else {
                continue;
            };
            let mut dependency_targets = FxHashSet::default();
            // Generated SFC projections carry synthetic triple-slash paths for
            // the type checker. Only references in the authored SFC source are
            // dependency edges; scanning the projection materializes the entire
            // project for each editor open.
            let references = if generated_sfc {
                self.original_contents
                    .get(&virtual_path)
                    .map_or_else(Vec::new, |source| references::path_references(source))
            } else {
                references::path_references(&virtual_content)
            };
            if references.is_empty()
                && !may_resolve_a_dependency(
                    &virtual_content,
                    if is_declaration_file(&importer) {
                        &declaration_alias_prefixes
                    } else {
                        &alias_prefixes
                    },
                    workspace_package_specifiers,
                )
            {
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
                .collect_all_specifier_occurrences(&virtual_content, source_type)
                .into_iter()
                .map(|(specifier, mode)| (specifier, mode, false))
                .chain(
                    references
                        .into_iter()
                        .map(|specifier| (specifier, crate::PackageResolutionMode::Import, true)),
                );
            // Route bindings stay unchanged during this walk. Incomplete raw
            // roots and noncanonical originals retain their previous lookup.
            let use_root_index = raw_roots_indexed && visited.contains(&importer);
            if use_root_index {
                indexed_package_root_lookups += 1;
            } else {
                fallback_package_root_lookups += 1;
            }
            let importer_package_roots = self.package_roots_for_importer(&importer, use_root_index);

            for (specifier, mode, is_reference) in specifiers {
                // Path references are always relative to the containing file,
                // even without `./`; import aliases and packages do not apply.
                let native_target = if is_reference {
                    resolution::probe_candidates(&importer_dir.join(&specifier))
                } else {
                    resolve_dependency(&specifier, &importer_dir, &self.project_root, &aliases)
                };
                let Some(target) = native_target else {
                    if !is_reference && let Some(resolve) = package_resolver.as_deref_mut() {
                        let _ = resolve(&importer, &specifier, mode);
                    }
                    continue;
                };
                let Some(key) = canonical_key(&target) else {
                    continue;
                };
                let package_local = importer_package_roots
                    .iter()
                    .any(|package_root| key.starts_with(package_root));
                // A triple-slash path is an explicit program dependency, even
                // when it names a declaration under node_modules. Ordinary
                // package imports still use the resolver's package shadow.
                if inside_node_modules(&key) && !package_local && !is_reference {
                    continue;
                }
                // Declarations reached through a path reference must keep
                // their transitive declaration imports in this same mirror.
                // The final rewrite then points imports at that one identity.
                // Ordinary source imports can still use TypeScript's authored
                // declaration resolution without registering another root.
                if is_declaration_file(&key)
                    && !package_local
                    && !self.session_scripts
                    && !is_reference
                    && !is_declaration_file(&importer)
                {
                    continue;
                }
                // A requested subset still owns its complete import graph.
                // In-root scripts excluded by the scan are dependencies just
                // like out-of-root barrels; omitting them loses their types.
                dependency_targets.insert(key.clone());
                // A preceding changed owner can prune an already visited
                // target and its descendants before another owner acquires it.
                // The scoped walk skips only a registration that still exists;
                // full cold walks keep their original failed-inference policy.
                if !visited.insert(key.clone())
                    && (initial_sources.is_none() || self.find_by_original(&key).is_some())
                {
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
        crate::corsa_bridge::preparation_trace::Phase::package_root_lookup(
            checked_roots,
            indexed_package_root_lookups,
            fallback_package_root_lookups,
        );
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

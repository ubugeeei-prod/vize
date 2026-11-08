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
//! first-party source. Batch mirrors source-backed declaration barrels while
//! terminal ambient declarations remain governed by tsconfig. Editor sessions
//! mirror reachable declarations, preserving their authored `.d.ts` spelling,
//! but keep them inferred rather than adding them as ambient program roots
//! (see [`is_declaration_file`]).
//!
//! Specifiers are collected from valid TypeScript projections and resolved
//! against the *original* file's directory. Only generated Vue projections
//! fold their `.vue.ts` mirror spelling back to `.vue`; authored TypeScript
//! keeps a literal `.vue.ts` identity. Resolution covers relative
//! specifiers and tsconfig `paths` aliases — the shapes the monorepo defect
//! reproduces with; bare workspace-package specifiers resolve only through
//! their `paths` alias today.

use std::path::{Path, PathBuf};

use rayon::prelude::*;
#[cfg(test)]
use vize_carton::cstr;
use vize_carton::{FxHashMap, FxHashSet, String as CompactString};

use crate::batch::error::CorsaResult;
use crate::batch::import_rewriter::{ModuleSpecifierFactsCache, ModuleSpecifierSpan};

use super::VirtualProject;
use super::setup_props::RuntimePropResolveCache;

mod declaration_sources;
use declaration_sources::source_backed;
mod entrypoints;
mod package_roots;
mod references;
pub(super) use references::path_references;
mod resolution;
mod source_resolution;
use package_roots::PackageRoots;
use resolution::{
    alias_may_reach_first_party, canonical_key, inside_node_modules, is_declaration_file,
    may_resolve_a_dependency,
};
pub(crate) use resolution::{
    probe_candidates, resolve_dependency, resolve_dependency_with_inputs,
    resolve_generated_dependency, resolve_generated_dependency_with_inputs,
};

type PackageResolver<'a> = &'a mut dyn FnMut(&Path, &str, crate::PackageResolutionMode) -> bool;

enum DependencySpecifierIter<'a> {
    Parsed(std::slice::Iter<'a, (CompactString, crate::PackageResolutionMode)>),
    Reused {
        source: &'a str,
        spans: std::slice::Iter<'a, ModuleSpecifierSpan>,
    },
}

impl<'a> Iterator for DependencySpecifierIter<'a> {
    type Item = (&'a str, crate::PackageResolutionMode);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Parsed(occurrences) => occurrences
                .next()
                .map(|(specifier, mode)| (specifier.as_str(), *mode)),
            Self::Reused { source, spans } => loop {
                let (start, end, mode) = spans.next()?;
                if let Some(specifier) = source.get(*start as usize..*end as usize) {
                    return Some((specifier, *mode));
                }
            },
        }
    }
}

#[expect(clippy::disallowed_types, reason = "dependency API uses std String")]
impl VirtualProject {
    fn register_reachable_dependencies_inner(
        &mut self,
        initial_sources: Option<&[PathBuf]>,
        overlays: &FxHashMap<PathBuf, &str>,
        workspace_package_specifiers: &[CompactString],
        mut package_resolver: Option<PackageResolver<'_>>,
        mut facts: Option<&mut ModuleSpecifierFactsCache>,
    ) -> CorsaResult<()> {
        // One walk owns one immutable set of imported type facts, just like
        // register_paths_with_content. Reachable SFCs often share the same
        // imported props; rebuilding that snapshot per file repeats the entire
        // imported-type graph. Derived runtime facts remain root-local, and a
        // later walk always starts fresh.
        let runtime_prop_resolve_cache = RuntimePropResolveCache::default();
        let aliases = self.dependency_alias_map();
        let mut source_cache = FxHashMap::default();
        let alias_prefixes: Vec<CompactString> = aliases
            .iter()
            .filter(|(pattern, target)| {
                alias_may_reach_first_party(pattern, target, &self.project_root)
                    || declaration_sources::alias_reaches_project_source(
                        self,
                        target,
                        &aliases,
                        overlays,
                        &mut source_cache,
                    )
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
        // Canonicalize the initial importers once, in parallel; the edge index
        // and the visited set both key on these paths.
        let canonical_importers = queue
            .par_iter()
            .map(|path| {
                let canonical = vize_carton::path::canonicalize_non_verbatim(path);
                let is_file = canonical.is_file();
                (path.clone(), canonical, is_file)
            })
            .collect::<Vec<_>>();
        let mut visited: FxHashSet<PathBuf> = canonical_importers
            .iter()
            .filter(|(_, _, is_file)| *is_file)
            .map(|(_, canonical, _)| canonical.clone())
            .collect();
        let canonical_importers: FxHashMap<PathBuf, PathBuf> = canonical_importers
            .into_iter()
            .map(|(path, canonical, _)| (path, canonical))
            .collect();
        // Importers share targets; each one costs a `realpath` and a `stat`.
        let mut canonical_keys: FxHashMap<PathBuf, Option<PathBuf>> = FxHashMap::default();
        let mut package_roots = None;

        while let Some(importer) = queue.pop() {
            vize_carton::micro_count!("canon.dependency_scan.importers.popped", 1);
            let Some((original_path, virtual_path, generated_vue)) =
                self.find_by_original(&importer).map(|file| {
                    (
                        file.original_path.clone(),
                        file.virtual_path.clone(),
                        file.source_map.sfc_map.is_some(),
                    )
                })
            else {
                continue;
            };
            let reused_syntax = self
                .source_artifacts
                .get_mut(&original_path)
                .and_then(|artifacts| artifacts.dependency_syntax_facts.take());
            let mut virtual_content = None;
            let fallback_references;
            let references = if let Some(syntax) = reused_syntax.as_ref() {
                vize_carton::micro_count!("canon.dependency_scan.occurrences.reused.calls", 1);
                vize_carton::micro_count!(
                    "canon.dependency_scan.occurrences.reused.source_bytes",
                    syntax.source_bytes
                );
                syntax.references.as_ref()
            } else {
                let Some((content, references)) = self.find_by_original(&importer).map(|file| {
                    let content = self.walked_source(file);
                    vize_carton::micro_count!("canon.dependency_scan.content.clone.calls", 1);
                    vize_carton::micro_count!(
                        "canon.dependency_scan.content.clone.source_bytes",
                        content.len()
                    );
                    vize_carton::micro_count!(
                        "canon.dependency_scan.content.generated_source_bytes",
                        if file.source_map.sfc_map.is_some() {
                            content.len()
                        } else {
                            0
                        }
                    );
                    (
                        vize_carton::micro_span!(
                            "canon.dependency_scan.content.clone",
                            vize_carton::String::from(content.as_ref())
                        ),
                        self.walked_references(file, &content).into_boxed_slice(),
                    )
                }) else {
                    continue;
                };
                virtual_content = Some(content);
                fallback_references = references;
                fallback_references.as_ref()
            };
            let mut dependency_targets = FxHashSet::default();
            let has_no_dependency_syntax = if references.is_empty() {
                match reused_syntax.as_ref() {
                    Some(syntax) => syntax.occurrences.is_empty(),
                    None => {
                        let Some(virtual_content) = virtual_content.as_deref() else {
                            continue;
                        };
                        !may_resolve_a_dependency(
                            virtual_content,
                            if is_declaration_file(&importer) {
                                &declaration_alias_prefixes
                            } else {
                                &alias_prefixes
                            },
                            workspace_package_specifiers,
                        )
                    }
                }
            } else {
                false
            };
            if has_no_dependency_syntax {
                let released = self.replace_canonical_dependency_edges(
                    canonical_importer(&canonical_importers, &importer),
                    dependency_targets,
                );
                self.prune_unowned_sources(released);
                continue;
            }
            let Some(importer_dir) = importer.parent().map(Path::to_path_buf) else {
                continue;
            };
            let source_type = super::editor_source_facts::walked_source_type(&virtual_path);
            let parsed_occurrences;
            let occurrences = if let Some(syntax) = reused_syntax.as_ref() {
                let source = if syntax.occurrences.is_empty() {
                    ""
                } else {
                    let Some(content) = self.find_by_original(&importer).map(|file| {
                        let content = self.walked_source(file);
                        vize_carton::micro_count!("canon.dependency_scan.content.clone.calls", 1);
                        vize_carton::micro_count!(
                            "canon.dependency_scan.content.clone.source_bytes",
                            content.len()
                        );
                        vize_carton::micro_span!(
                            "canon.dependency_scan.content.clone",
                            vize_carton::String::from(content.as_ref())
                        )
                    }) else {
                        continue;
                    };
                    virtual_content = Some(content);
                    virtual_content.as_deref().unwrap_or_default()
                };
                DependencySpecifierIter::Reused {
                    source,
                    spans: syntax.occurrences.iter(),
                }
            } else {
                let Some(virtual_content) = virtual_content.as_deref() else {
                    continue;
                };
                parsed_occurrences = match facts.as_deref_mut() {
                    Some(facts) => {
                        vize_carton::micro_count!(
                            "canon.dependency_scan.occurrences.cached.calls",
                            1
                        );
                        vize_carton::micro_count!(
                            "canon.dependency_scan.occurrences.cached.source_bytes",
                            virtual_content.len()
                        );
                        vize_carton::micro_span!(
                            "canon.dependency_scan.occurrences.cached",
                            facts.collect_occurrences(virtual_content, source_type)
                        )
                    }
                    None => {
                        vize_carton::micro_count!(
                            "canon.dependency_scan.occurrences.uncached.calls",
                            1
                        );
                        vize_carton::micro_count!(
                            "canon.dependency_scan.occurrences.uncached.source_bytes",
                            virtual_content.len()
                        );
                        #[cfg(feature = "profiling")]
                        crate::profiling::observe_dependency_input(virtual_content, source_type);
                        vize_carton::micro_span!(
                            "canon.dependency_scan.occurrences.uncached",
                            self.rewriter()
                                .collect_all_specifier_occurrences(virtual_content, source_type)
                        )
                    }
                };
                DependencySpecifierIter::Parsed(parsed_occurrences.iter())
            };
            let specifiers = occurrences
                .map(|(specifier, mode)| (specifier, mode, false))
                .chain(references.iter().map(|specifier| {
                    (
                        specifier.as_str(),
                        crate::PackageResolutionMode::Import,
                        true,
                    )
                }));
            // Routes are stable for one walk. Collect their logical roots only
            // when needed, then borrow matching importer ancestors instead of
            // scanning and cloning every route again for every importer.
            let importer_package_roots = package_roots
                .get_or_insert_with(|| {
                    PackageRoots::new(
                        self.package_routes
                            .values()
                            .filter_map(|binding| binding.route.as_ref())
                            .flat_map(crate::PackageRoute::all_routes)
                            .map(|route| route.package_root.as_path()),
                    )
                })
                .for_importer(&importer);

            for (specifier, mode, is_reference) in specifiers {
                // Path references are always relative to the containing file,
                // even without `./`; import aliases and packages do not apply.
                let native_target = if is_reference {
                    resolution::probe_candidates(&importer_dir.join(specifier))
                } else {
                    self.resolve_source_dependency(&importer, generated_vue, specifier, &aliases)
                };
                let Some(target) = native_target else {
                    if !is_reference && let Some(resolve) = package_resolver.as_deref_mut() {
                        let _ = resolve(&importer, specifier, mode);
                    }
                    continue;
                };
                let Some(key) = canonical_keys
                    .entry(target)
                    .or_insert_with_key(|target| canonical_key(target))
                    .clone()
                else {
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
                // Terminal declarations keep their authored identity, but a
                // source-backed barrel must own its projected source edges.
                if is_declaration_file(&key)
                    && !package_local
                    && !self.session_scripts
                    && !is_reference
                    && !is_declaration_file(&importer)
                    && !source_backed(self, &key, &aliases, overlays, &mut source_cache)
                {
                    continue;
                }
                // A requested subset still owns its complete import graph.
                // In-root scripts excluded by the scan are dependencies just
                // like out-of-root barrels; omitting them loses their types.
                dependency_targets.insert(key.clone());
                if !visited.insert(key.clone()) {
                    continue;
                }
                // Another program of this check owns the source and mirrors
                // its declaration here; it is never projected again.
                if self.is_foreign_source(&key) {
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
                let root_cache = runtime_prop_resolve_cache.with_shared_type_sources();
                let registered = match overlays.get(&key) {
                    Some(content) => {
                        self.register_content_with_prop_cache(&key, content, Some(&root_cache))
                    }
                    None => match vize_carton::profile!(
                        "canon.file.read",
                        std::fs::read_to_string(&key)
                    ) {
                        Ok(content) => {
                            self.register_content_with_prop_cache(&key, &content, Some(&root_cache))
                        }
                        Err(error) => Err(error.into()),
                    },
                };
                if registered.is_ok() {
                    queue.push(key);
                }
            }
            let released = self.replace_canonical_dependency_edges(
                canonical_importer(&canonical_importers, &importer),
                dependency_targets,
            );
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

fn canonical_importer(known: &FxHashMap<PathBuf, PathBuf>, importer: &Path) -> PathBuf {
    known
        .get(importer)
        .cloned()
        .unwrap_or_else(|| vize_carton::path::canonicalize_non_verbatim(importer))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod shared_type_sources_tests;

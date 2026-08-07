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
//! first-party source. Declaration files are left alone too: they hold no
//! component to check and their ambient declarations are program-wide, so the
//! tsconfig decides which ones a program includes (see
//! [`is_declaration_file`]).
//!
//! Specifiers are collected from the *generated* content (always valid TS,
//! one collector for `.vue` and script files alike) and resolved against the
//! *original* file's directory; a `.vue.ts` spelling the import rewriter
//! produced is folded back to `.vue` first. Resolution covers relative
//! specifiers and tsconfig `paths` aliases — the shapes the monorepo defect
//! reproduces with; bare workspace-package specifiers resolve only through
//! their `paths` alias today.

use std::path::{Component, Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::{FxHashMap, FxHashSet, String as CompactString, cstr};

use crate::batch::error::CorsaResult;

use super::VirtualProject;

#[allow(clippy::disallowed_types)]
impl VirtualProject {
    /// Register every reachable first-party dependency, to a fixpoint.
    pub fn register_reachable_dependencies(&mut self) -> CorsaResult<()> {
        self.register_reachable_dependencies_with_overlays(&FxHashMap::default())
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
        let aliases = self.dependency_alias_map();
        let alias_prefixes: Vec<CompactString> = aliases
            .iter()
            .filter(|(pattern, target)| {
                alias_may_reach_first_party(pattern, target, &self.project_root)
            })
            .map(|(pattern, _)| CompactString::from(pattern.trim_end_matches('*')))
            .collect();
        let mut queue: Vec<PathBuf> = self
            .virtual_files_sorted()
            .iter()
            .map(|file| file.original_path.clone())
            .collect();
        let mut visited: FxHashSet<PathBuf> = queue
            .iter()
            .filter_map(|path| canonical_key(path))
            .collect();

        while let Some(importer) = queue.pop() {
            let Some(virtual_file) = self.find_by_original(&importer) else {
                continue;
            };
            if !may_resolve_a_dependency(&virtual_file.content, &alias_prefixes) {
                continue;
            }
            let Some(importer_dir) = importer.parent().map(Path::to_path_buf) else {
                continue;
            };
            let source_type = if virtual_file
                .virtual_path
                .extension()
                .is_some_and(|extension| extension == "tsx")
            {
                SourceType::tsx()
            } else {
                SourceType::ts()
            };
            let specifiers = self
                .rewriter()
                .collect_all_specifiers(&virtual_file.content, source_type);

            for specifier in specifiers {
                let Some(target) =
                    resolve_dependency(&specifier, &importer_dir, &self.project_root, &aliases)
                else {
                    continue;
                };
                let Some(key) = canonical_key(&target) else {
                    continue;
                };
                if inside_node_modules(&key) || is_declaration_file(&key) {
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
                if !is_vue && !self.session_scripts && key.starts_with(&self.project_root) {
                    continue;
                }
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

/// First-party classification key: the canonical path, so a pnpm workspace
/// symlink is judged by where it actually lives.
fn canonical_key(path: &Path) -> Option<PathBuf> {
    let canonical = vize_carton::path::canonicalize_non_verbatim(path);
    canonical.is_file().then_some(canonical)
}

fn inside_node_modules(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::Normal(part) if part == "node_modules"))
}

/// Whether `path` is a TypeScript declaration file.
///
/// Reachability never registers one. A declaration file cannot *be* a `.vue`
/// component, so it adds nothing this pass exists to check, while its ambient
/// declarations are program-wide: a `declare module "vue"` in a script `.d.ts`
/// is an ambient module declaration rather than an augmentation, so pulling one
/// in replaces Vue's real typings and every `import { ref } from "vue"` in the
/// project becomes `TS2305` (#3898). Which declaration files a program includes
/// is the tsconfig's decision, not this inference pass's.
///
/// [`probe_candidates`] still resolves them: [`alias_may_reach_first_party`]
/// classifies an alias by what the walk would resolve, and the `vue` alias of a
/// Vue tsconfig resolves to `node_modules/vue/index.d.ts`.
fn is_declaration_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        })
}

/// Whether any specifier in `content` could resolve, cheaply and without a
/// parse. [`resolve_dependency`] only ever succeeds for a relative specifier or
/// one matching a `paths` alias, so a module whose text contains neither shape
/// cannot register anything. This pass runs on every check, and registration
/// itself skips parsing a module with no rewritable specifier, so parsing every
/// generated file here would be a plain regression on projects that import
/// nothing first-party (#3898). Deliberately a conservative superset: alias
/// prefixes are matched anywhere in the text.
/// Whether an alias can ever contribute a first-party file, judged once per
/// alias rather than once per module. A wildcard pattern always can: its target
/// is a directory prefix whose entries may each be a pnpm workspace symlink out
/// of `node_modules`. A wildcard-free pattern names exactly one target, so it is
/// resolved here the same way the walk resolves it and kept only when the walk
/// would accept the result. Its prefix must not make
/// [`may_resolve_a_dependency`] parse every module in the project: the `vue`
/// alias a Vue tsconfig carries is that shape, and matching it as a bare
/// substring defeated the prefilter for every generated file (#3898).
///
/// A probe that finds nothing means the walk resolves nothing either, so the
/// prefix is dropped. Treating that as "keep, to be safe" is what kept the
/// benchmark regression alive: `vue` publishes its types as `dist/vue.d.ts` with
/// no root `index.d.ts`, so probing the package directory legitimately fails.
#[allow(clippy::disallowed_types)]
fn alias_may_reach_first_party(pattern: &str, target: &str, project_root: &Path) -> bool {
    if pattern.contains('*') {
        return true;
    }
    let absolute = if Path::new(target).is_absolute() {
        PathBuf::from(target)
    } else {
        project_root.join(target)
    };
    // Resolve exactly as the walk would, so dropping the prefix can only ever
    // drop a target the walk itself refuses.
    let Some(resolved) = probe_candidates(&absolute) else {
        return false;
    };
    let Some(key) = canonical_key(&resolved) else {
        return false;
    };
    !inside_node_modules(&key) && !is_declaration_file(&key)
}

fn may_resolve_a_dependency(content: &str, alias_prefixes: &[CompactString]) -> bool {
    crate::batch::import_rewriter::source_may_contain_relative_specifier(content)
        || alias_prefixes
            .iter()
            .any(|prefix| prefix.is_empty() || content.contains(prefix.as_str()))
}

/// Resolve one specifier to a registrable first-party file, or `None`.
#[allow(clippy::disallowed_types)]
pub(crate) fn resolve_dependency(
    specifier: &str,
    importer_dir: &Path,
    project_root: &Path,
    aliases: &[(String, String)],
) -> Option<PathBuf> {
    // Fold the rewriter's `.vue.ts` spelling back to the real file.
    let specifier = specifier
        .strip_suffix(".vue.ts")
        .map_or_else(|| specifier.to_owned(), |stem| cstr!("{stem}.vue").into());

    if specifier.starts_with("./") || specifier.starts_with("../") {
        return probe_candidates(&importer_dir.join(&specifier));
    }

    // Longest matching alias pattern wins, mirroring TypeScript's `paths`.
    let mut best: Option<(usize, PathBuf)> = None;
    for (pattern, target) in aliases {
        let substituted = if let Some(prefix) = pattern.strip_suffix('*') {
            match (specifier.strip_prefix(prefix), target.strip_suffix('*')) {
                (Some(rest), Some(target_prefix)) => {
                    let mut joined = target_prefix.to_owned();
                    joined.push_str(rest);
                    Some(joined)
                }
                _ => None,
            }
        } else if specifier == *pattern {
            Some(target.clone())
        } else {
            None
        };
        let Some(substituted) = substituted else {
            continue;
        };
        let absolute = if Path::new(&substituted).is_absolute() {
            PathBuf::from(&substituted)
        } else {
            project_root.join(&substituted)
        };
        if let Some(resolved) = probe_candidates(&absolute)
            && best.as_ref().is_none_or(|(len, _)| pattern.len() > *len)
        {
            best = Some((pattern.len(), resolved));
        }
    }
    best.map(|(_, path)| path)
}

/// Extension and index probing for a resolved base path, in TypeScript's
/// order. Only extensions the registration pipeline accepts are produced.
fn probe_candidates(base: &Path) -> Option<PathBuf> {
    if base.extension().is_some() && base.is_file() {
        return Some(base.to_path_buf());
    }
    for extension in ["ts", "tsx", "d.ts", "vue"] {
        let candidate = PathBuf::from(cstr!("{}.{extension}", base.display()).as_str());
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    for index in ["index.ts", "index.tsx", "index.d.ts"] {
        let candidate = base.join(index);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
#[path = "dependency_scan_tests.rs"]
mod tests;

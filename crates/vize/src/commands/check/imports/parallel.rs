//! Parallel package and `paths` resolution for one import frontier.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use vize_canon::{
    PackageResolutionContext, PackageResolutionMode, PackageRouteLookup, PackageRouteResolver,
    PackageSourceOptions,
};
use vize_l0::{FxHashMap, FxHashSet, String};

use super::super::imports_aliases::PathAliasResolver;
use super::super::path_cache::CanonicalPathCache;
use super::session::{AliasScope, LocalImportSession};
use super::specifiers::{
    ModuleSpecifierOccurrence, extract_module_specifier_occurrences, is_relative_specifier,
};
use super::{
    ImportFileOptions, is_declaration_file, is_node_modules_path, resolve_import_base,
    resolve_relative_import,
};

struct BareImport {
    file: PathBuf,
    dir: PathBuf,
    specifier: String,
    mode: PackageResolutionMode,
    extension: Option<OsString>,
}

struct ContextJob {
    key: (PathBuf, Option<OsString>, PackageResolutionMode),
    file: PathBuf,
    mode: PackageResolutionMode,
}

struct PackageJob {
    dir: PathBuf,
    specifier: String,
    context: PackageResolutionContext,
}

/// Files one frontier looks ahead through before the serial walk resumes.
const LOOKAHEAD_FILES: usize = 4096;

pub(super) fn prepare_frontier(
    file: &Path,
    queue: &[(PathBuf, bool, bool)],
    session: &mut LocalImportSession,
    packages: &PackageRouteResolver,
    aliases: Option<&PathAliasResolver>,
    options: ImportFileOptions,
    canonical_paths: &mut CanonicalPathCache,
) {
    // A later pop, and the hidden-ambient pass, already cached these files.
    // Rebuilding bare imports for the whole queue repeats work the session holds.
    let mut wave = Vec::new();
    push_unscanned(&mut wave, file, session);
    for (path, _, _) in queue {
        push_unscanned(&mut wave, path, session);
    }
    if wave.is_empty() {
        return;
    }
    // Look ahead through relative, alias and workspace-route edges, so one
    // batch resolves the packages of a whole subtree instead of one file's
    // children. Only caches fill here; the serial walk keeps its order and
    // every decision.
    let mut seen = wave.iter().cloned().collect::<FxHashSet<_>>();
    let scope = AliasScope::new(aliases, options);
    let mut prepared = 0;
    while !wave.is_empty() && prepared < LOOKAHEAD_FILES {
        prepared += wave.len();
        prefetch_specifiers(&wave, session);
        prepare_pending(&wave, session, Some(packages), aliases, options);
        wave = next_wave(&wave, session, scope, options, canonical_paths, &mut seen);
    }
}

/// Resolve the `paths` aliases, resolution contexts and package routes of
/// files whose specifiers the session already holds, off this thread.
pub(super) fn prepare_pending(
    pending: &[PathBuf],
    session: &mut LocalImportSession,
    packages: Option<&PackageRouteResolver>,
    aliases: Option<&PathAliasResolver>,
    options: ImportFileOptions,
) {
    let bare = bare_imports(pending, session);
    let scope = AliasScope::new(aliases, options);
    if let Some(aliases) = aliases {
        resolve_alias_misses(&bare, aliases, options, scope, session);
    }
    let Some(packages) = packages else {
        return;
    };
    let alias_targets = session.alias_targets(scope);
    let package_imports = bare
        .into_iter()
        .filter(|import| !alias_hit(alias_targets, &import.specifier))
        .filter(|import| !vize_canon::batch::is_vue_runtime_support_specifier(&import.specifier))
        .collect::<Vec<_>>();
    resolve_context_misses(&package_imports, session, packages, aliases);
    resolve_package_misses(&package_imports, session, packages, options);
}

/// The files the serial walk reaches from `wave` through edges it resolves
/// itself: relative and alias imports, and the sources of workspace routes.
fn next_wave(
    wave: &[PathBuf],
    session: &LocalImportSession,
    scope: AliasScope,
    options: ImportFileOptions,
    canonical_paths: &mut CanonicalPathCache,
    seen: &mut FxHashSet<PathBuf>,
) -> Vec<PathBuf> {
    let source_options = PackageSourceOptions::new(options.include_js, options.include_jsx);
    let mut next = Vec::new();
    for path in wave {
        let Some(dir) = path.parent() else {
            continue;
        };
        let Some(occurrences) = session
            .registration_cache
            .module_specifiers
            .get(path)
            .and_then(|cached| cached.as_deref())
        else {
            continue;
        };
        let extension = path.extension().map(OsString::from);
        for occurrence in occurrences {
            let specifier = occurrence.specifier.as_str();
            if is_relative_specifier(specifier) {
                if let Some(target) =
                    resolve_relative_import(dir, specifier, canonical_paths, options)
                {
                    consider(target, session, seen, &mut next);
                }
                continue;
            }
            if Path::new(specifier).is_absolute() {
                continue;
            }
            if let Some(Some(target)) = session
                .alias_targets(scope)
                .and_then(|targets| targets.get(specifier))
            {
                consider(target.clone(), session, seen, &mut next);
                continue;
            }
            let Some((context, _)) = session.resolution_contexts.get(&(
                dir.to_path_buf(),
                extension.clone(),
                occurrence.mode,
            )) else {
                continue;
            };
            let key = (
                dir.to_path_buf(),
                occurrence.specifier.clone(),
                source_options,
                context.clone(),
            );
            let Some(route) = session
                .package_lookups
                .get(&key)
                .and_then(PackageRouteLookup::route)
            else {
                continue;
            };
            if route.workspace_source {
                for source in route.all_source_paths() {
                    consider(source.clone(), session, seen, &mut next);
                }
            }
        }
    }
    next
}

/// Queue a reached file once, unless the walk never descends into it.
fn consider(
    target: PathBuf,
    session: &LocalImportSession,
    seen: &mut FxHashSet<PathBuf>,
    next: &mut Vec<PathBuf>,
) {
    if is_declaration_file(&target)
        || is_node_modules_path(&target)
        || session
            .registration_cache
            .module_specifiers
            .contains_key(&target)
    {
        return;
    }
    if seen.insert(target.clone()) {
        next.push(target);
    }
}

fn push_unscanned(pending: &mut Vec<PathBuf>, path: &Path, session: &LocalImportSession) {
    if session
        .registration_cache
        .module_specifiers
        .contains_key(path)
    {
        return;
    }
    pending.push(path.to_path_buf());
}

fn prefetch_specifiers(paths: &[PathBuf], session: &mut LocalImportSession) {
    let pending = paths
        .iter()
        .filter(|path| {
            !session
                .registration_cache
                .module_specifiers
                .contains_key(*path)
        })
        .cloned()
        .collect::<Vec<_>>();
    if pending.is_empty() {
        return;
    }
    let loaded = pending
        .into_par_iter()
        .map(|path| (path.clone(), scan_module_specifiers(&path)))
        .collect::<Vec<_>>();
    session.registration_cache.module_specifiers.extend(loaded);
}

fn scan_module_specifiers(file: &Path) -> Option<Box<[ModuleSpecifierOccurrence]>> {
    let source = std::fs::read_to_string(file).ok()?;
    Some(extract_module_specifier_occurrences(&source).into_boxed_slice())
}

fn bare_imports(paths: &[PathBuf], session: &LocalImportSession) -> Vec<BareImport> {
    let mut imports = Vec::new();
    for path in paths {
        let Some(dir) = path.parent() else {
            continue;
        };
        let Some(occurrences) = session
            .registration_cache
            .module_specifiers
            .get(path)
            .and_then(|cached| cached.as_deref())
        else {
            continue;
        };
        let extension = path.extension().map(OsString::from);
        for occurrence in occurrences {
            if !is_bare_specifier(&occurrence.specifier) {
                continue;
            }
            imports.push(BareImport {
                file: path.clone(),
                dir: dir.to_path_buf(),
                specifier: occurrence.specifier.clone(),
                mode: occurrence.mode,
                extension: extension.clone(),
            });
        }
    }
    imports
}

fn is_bare_specifier(specifier: &str) -> bool {
    !is_relative_specifier(specifier) && !Path::new(specifier).is_absolute()
}

fn alias_hit(alias_targets: Option<&FxHashMap<String, Option<PathBuf>>>, specifier: &str) -> bool {
    alias_targets
        .and_then(|targets| targets.get(specifier))
        .and_then(Option::as_ref)
        .is_some()
}

fn resolve_alias_misses(
    imports: &[BareImport],
    aliases: &PathAliasResolver,
    options: ImportFileOptions,
    scope: AliasScope,
    session: &mut LocalImportSession,
) {
    let mut seen = FxHashSet::default();
    let mut missing = Vec::new();
    let cached = session.alias_targets(scope);
    for import in imports {
        if cached.is_some_and(|targets| targets.contains_key(&import.specifier))
            || !seen.insert(import.specifier.clone())
        {
            continue;
        }
        missing.push(import.specifier.clone());
    }
    let probe = path_probe(session);
    let resolved = map_concurrent(missing, |specifier| {
        if let Some(probe) = probe {
            probe();
        }
        let mut canonical_paths = CanonicalPathCache::default();
        let resolved = aliases.resolve(
            &specifier,
            &mut canonical_paths,
            options,
            resolve_import_base,
        );
        (specifier, resolved)
    });
    let targets = session.alias_targets_mut(scope);
    for (specifier, resolved) in resolved {
        targets.entry(specifier).or_insert(resolved);
    }
}

fn resolve_context_misses(
    imports: &[BareImport],
    session: &mut LocalImportSession,
    packages: &PackageRouteResolver,
    aliases: Option<&PathAliasResolver>,
) {
    let mut seen = FxHashSet::default();
    let mut jobs = Vec::new();
    for import in imports {
        let key = (import.dir.clone(), import.extension.clone(), import.mode);
        if session.resolution_contexts.contains_key(&key) || !seen.insert(key.clone()) {
            continue;
        }
        jobs.push(ContextJob {
            key,
            file: import.file.clone(),
            mode: import.mode,
        });
    }
    let resolved = map_concurrent(jobs, |job| {
        let value = if let Some(aliases) = aliases {
            aliases.package_resolution_context_concurrent(packages, &job.file, job.mode)
        } else {
            packages.resolution_context_concurrent(
                &job.file,
                job.mode,
                None,
                None,
                std::iter::empty::<String>(),
            )
        };
        (job.key, value)
    });
    for (key, value) in resolved {
        session.resolution_contexts.entry(key).or_insert(value);
    }
}

fn resolve_package_misses(
    imports: &[BareImport],
    session: &mut LocalImportSession,
    packages: &PackageRouteResolver,
    options: ImportFileOptions,
) {
    let source_options = PackageSourceOptions::new(options.include_js, options.include_jsx);
    let mut seen = FxHashSet::default();
    let mut jobs = Vec::new();
    for import in imports {
        let Some((context, _)) = session.resolution_contexts.get(&(
            import.dir.clone(),
            import.extension.clone(),
            import.mode,
        )) else {
            continue;
        };
        let key = (
            import.dir.clone(),
            import.specifier.clone(),
            source_options,
            context.clone(),
        );
        if session.package_lookups.contains_key(&key) || !seen.insert(key) {
            continue;
        }
        jobs.push(PackageJob {
            dir: import.dir.clone(),
            specifier: import.specifier.clone(),
            context: context.clone(),
        });
    }
    let probe = package_probe(session);
    let resolved = map_concurrent(jobs, |job| {
        if let Some(probe) = probe {
            probe();
        }
        let key = (
            job.dir.clone(),
            job.specifier.clone(),
            source_options,
            job.context.clone(),
        );
        let lookup = packages.lookup_with_context_concurrent(
            &job.dir,
            job.specifier.as_str(),
            source_options,
            job.context,
        );
        (key, lookup)
    });
    for (key, lookup) in resolved {
        session.package_lookups.entry(key).or_insert(lookup);
    }
}

fn path_probe(session: &LocalImportSession) -> Option<fn()> {
    #[cfg(test)]
    {
        session.path_lookup_probe
    }
    #[cfg(not(test))]
    {
        let _ = session;
        None
    }
}

fn package_probe(session: &LocalImportSession) -> Option<fn()> {
    #[cfg(test)]
    {
        session.package_lookup_probe
    }
    #[cfg(not(test))]
    {
        let _ = session;
        None
    }
}

/// `std::thread` so `RAYON_NUM_THREADS=1` cannot serialize independent lookups.
/// Map `items` on the shared pool, in order. The map runs on the caller for a
/// single item, since nothing would overlap.
fn map_concurrent<T, U, F>(items: Vec<T>, map: F) -> Vec<U>
where
    T: Send,
    U: Send,
    F: Fn(T) -> U + Sync + Send,
{
    if items.len() <= 1 {
        return items.into_iter().map(map).collect();
    }
    items.into_par_iter().map(map).collect()
}

#[cfg(test)]
mod tests;

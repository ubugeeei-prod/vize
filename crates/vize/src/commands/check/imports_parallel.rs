//! Parallel package and `paths` resolution for one import frontier.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use vize_canon::{
    PackageResolutionContext, PackageResolutionMode, PackageRouteResolver, PackageSourceOptions,
};
use vize_l0::{FxHashMap, FxHashSet, String};

use super::super::imports_aliases::PathAliasResolver;
use super::super::path_cache::CanonicalPathCache;
use super::session::LocalImportSession;
use super::specifiers::{
    ModuleSpecifierOccurrence, extract_module_specifier_occurrences, is_relative_specifier,
};
use super::{ImportFileOptions, resolve_import_base};

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

pub(super) fn prepare_frontier(
    file: &Path,
    queue: &[(PathBuf, bool, bool)],
    session: &mut LocalImportSession,
    packages: &PackageRouteResolver,
    aliases: Option<&PathAliasResolver>,
    options: ImportFileOptions,
    alias_targets: &mut FxHashMap<String, Option<PathBuf>>,
) {
    let mut paths = Vec::with_capacity(queue.len().saturating_add(1));
    paths.push(file.to_path_buf());
    paths.extend(queue.iter().map(|(path, _, _)| path.clone()));
    prefetch_specifiers(&paths, session);
    let bare = bare_imports(&paths, session);
    if let Some(aliases) = aliases {
        resolve_alias_misses(&bare, aliases, options, alias_targets, session);
    }
    let package_imports = bare
        .into_iter()
        .filter(|import| !alias_hit(alias_targets, &import.specifier))
        .filter(|import| !vize_canon::batch::is_vue_runtime_support_specifier(&import.specifier))
        .collect::<Vec<_>>();
    resolve_context_misses(&package_imports, session, packages, aliases);
    resolve_package_misses(&package_imports, session, packages, options);
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

fn alias_hit(alias_targets: &FxHashMap<String, Option<PathBuf>>, specifier: &str) -> bool {
    alias_targets
        .get(specifier)
        .and_then(Option::as_ref)
        .is_some()
}

fn resolve_alias_misses(
    imports: &[BareImport],
    aliases: &PathAliasResolver,
    options: ImportFileOptions,
    alias_targets: &mut FxHashMap<String, Option<PathBuf>>,
    session: &LocalImportSession,
) {
    let mut seen = FxHashSet::default();
    let mut missing = Vec::new();
    for import in imports {
        if alias_targets.contains_key(&import.specifier) || !seen.insert(import.specifier.clone()) {
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
    for (specifier, resolved) in resolved {
        alias_targets.entry(specifier).or_insert(resolved);
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
fn map_concurrent<T, U, F>(items: Vec<T>, map: F) -> Vec<U>
where
    T: Send,
    U: Send,
    F: Fn(T) -> U + Sync,
{
    let count = items.len();
    if count <= 1 {
        return items.into_iter().map(map).collect();
    }
    let workers = std::thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(1)
        .min(count);
    if workers <= 1 {
        return items.into_iter().map(map).collect();
    }
    let mut chunks = Vec::with_capacity(workers);
    for _ in 0..workers {
        chunks.push(Vec::new());
    }
    let mut leftover = Vec::new();
    for (index, item) in items.into_iter().enumerate() {
        if let Some(chunk) = chunks.get_mut(index % workers) {
            chunk.push((index, item));
        } else {
            leftover.push((index, item));
        }
    }
    let mut indexed = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            handles.push(scope.spawn(|| {
                chunk
                    .into_iter()
                    .map(|(index, item)| (index, map(item)))
                    .collect::<Vec<_>>()
            }));
        }
        handles
            .into_iter()
            .map(|handle| handle.join())
            .collect::<Vec<_>>()
    });
    let mut rows = Vec::with_capacity(count);
    for part in indexed.drain(..) {
        match part {
            Ok(part) => rows.extend(part),
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }
    rows.extend(leftover.into_iter().map(|(index, item)| (index, map(item))));
    rows.sort_by_key(|(index, _)| *index);
    rows.into_iter().map(|(_, value)| value).collect()
}

#[cfg(test)]
#[path = "imports_parallel_tests.rs"]
mod tests;

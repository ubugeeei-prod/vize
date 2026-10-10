//! Source-only patches retain the existing importer-local package authority.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::{FxHashMap, FxHashSet, String};

use crate::corsa_bridge::vue_dependencies_alias::AliasContext;

pub(super) fn apply(
    context: &mut AliasContext,
    host: &Path,
    content: &str,
    overlays: &FxHashMap<PathBuf, &str>,
    requested: &[(PathBuf, &str)],
    disk_changes: &[PathBuf],
) -> Option<()> {
    let host = vize_carton::path::canonicalize_non_verbatim(host);
    let mirror = context.mirror.as_mut()?;
    mirror.find_by_original(&host)?;
    let known = mirror
        .registered_original_paths_sorted()
        .into_iter()
        .collect::<FxHashSet<_>>();
    // Vue generation reads imported props through its existing source resolvers,
    // whose consumed paths need not be dependency-edge targets. Regenerate every
    // retained Vue original instead of treating that graph as a codegen-input map.
    let vue_sources = known
        .iter()
        .filter(|path| {
            mirror
                .find_by_original(path)
                .is_some_and(|file| file.source_map.sfc_map.is_some())
        })
        .cloned()
        .collect::<FxHashSet<_>>();
    let mut supplied = requested
        .iter()
        .map(|(path, text)| (vize_carton::path::canonicalize_non_verbatim(path), *text))
        .collect::<FxHashMap<_, _>>();
    supplied.extend(
        overlays
            .iter()
            .filter(|(path, _)| known.contains(*path))
            .map(|(path, text)| (path.clone(), *text)),
    );
    supplied.insert(host, content);
    let mut replacements = FxHashMap::<PathBuf, Option<String>>::default();
    let mut authored_changes = FxHashSet::default();
    for path in known.iter().filter(|path| {
        vue_sources.contains(*path) || supplied.contains_key(*path) || disk_changes.contains(path)
    }) {
        let file = mirror.find_by_original(path)?;
        let previous = mirror.original_content_for_virtual(&file.virtual_path)?;
        let current = match supplied.get(path) {
            Some(source) => Some(String::from(*source)),
            None if path.is_file() => Some(String::from(std::fs::read_to_string(path).ok()?)),
            None => None,
        };
        if current.as_deref() != Some(previous) {
            authored_changes.insert(path.clone());
        }
        if vue_sources.contains(path) || authored_changes.contains(path) {
            replacements.insert(path.clone(), current);
        }
    }
    let changed = replacements.keys().cloned().collect::<FxHashSet<_>>();
    if !vue_sources.is_subset(&changed) {
        return None;
    }
    if authored_changes
        .iter()
        .any(|path| context.route_inputs.contains(path))
    {
        return None;
    }
    let old_packages = changed
        .iter()
        .map(|path| {
            let file = mirror.find_by_original(path)?;
            Some((path.clone(), package_occurrences(mirror, file)))
        })
        .collect::<Option<FxHashMap<_, _>>>()?;
    // Removing a source with package edges changes the package frontier. Its
    // complete cold route discovery remains the authority for that revision.
    if replacements.iter().any(|(path, source)| {
        source.is_none()
            && old_packages
                .get(path)
                .is_none_or(|packages| !packages.is_empty())
    }) {
        return None;
    }
    let mut frontier = mirror.dependency_owners_for_sources(&changed);
    let aliases = mirror.dependency_alias_map();
    // A deleted alias target can become a package lookup in the cold producer.
    // Preserve that resolution-input authority rather than inferring new routes.
    if frontier.iter().any(|path| {
        mirror.find_by_original(path).is_none_or(|file| {
            package_occurrences(mirror, file)
                .iter()
                .any(|(specifier, _)| {
                    aliases
                        .iter()
                        .any(|(pattern, _)| alias_matches(pattern, specifier))
                })
        })
    }) {
        return None;
    }
    let register = crate::corsa_bridge::preparation_trace::Phase::start(
        "alias_register_inputs",
        requested.len() + 1,
    );
    let mut ordered = replacements.into_iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.0.cmp(&right.0));
    let deleted = ordered
        .iter()
        .filter(|(_, source)| source.is_none())
        .map(|(path, _)| path.clone())
        .collect::<FxHashSet<_>>();
    let mut regenerated = FxHashSet::default();
    for (path, source) in ordered {
        if let Some(source) = source {
            mirror.register_path_with_content(&path, &source).ok()?;
            let file = mirror.find_by_original(&path)?;
            if Some(&package_occurrences(mirror, file)) != old_packages.get(&path) {
                return None;
            }
            regenerated.insert(path.clone());
            frontier.push(path);
        } else {
            mirror.remove_source_and_dependencies(&path);
        }
    }
    register.finish();
    // No old Vue projection may earn takeover credit after a source revision.
    if vue_sources
        .iter()
        .any(|path| !regenerated.contains(path) && !deleted.contains(path))
    {
        return None;
    }
    frontier.sort();
    frontier.dedup();
    let reachable =
        crate::corsa_bridge::preparation_trace::Phase::start("alias_reachable_dependencies", 1);
    mirror
        .register_reachable_dependencies_from(&frontier)
        .ok()?;
    reachable.finish();
    let current = mirror
        .registered_original_paths_sorted()
        .into_iter()
        .collect::<FxHashSet<_>>();
    // A changed owner can release then reacquire an old target during the
    // scoped walk. That inferred registration must not replace supplied
    // requested/overlay/host text with disk bytes.
    if supplied.iter().any(|(path, source)| {
        mirror.find_by_original(path).is_some_and(|file| {
            mirror.original_content_for_virtual(&file.virtual_path) != Some(*source)
        })
    }) {
        return None;
    }
    // Unrelated package input facts must never outlive a pruned importer.
    // New package inputs and supplied overlays require the complete cold producer.
    if known
        .difference(&current)
        .any(|path| !deleted.contains(path))
        || current.difference(&known).any(|path| {
            overlays.contains_key(path)
                || mirror
                    .find_by_original(path)
                    .is_none_or(|file| !package_occurrences(mirror, file).is_empty())
        })
    {
        return None;
    }
    let routes = crate::corsa_bridge::preparation_trace::Phase::start("alias_package_routes", 1);
    mirror.finalize_package_routes().ok()?;
    routes.finish();
    Some(())
}

fn package_occurrences(
    project: &crate::batch::virtual_project::VirtualProject,
    file: &crate::batch::virtual_project::VirtualFile,
) -> Vec<(String, crate::PackageResolutionMode)> {
    let source = project
        .original_content_for_virtual(&file.virtual_path)
        .unwrap_or(&file.content);
    // Vue source is not TypeScript; scan its valid pre-rewrite projection.
    let source = project
        .editor_vue_document(&file.original_path)
        .map(|document| document.1.pre_rewrite_code)
        .unwrap_or_else(|| String::from(source));
    let source_type = if file
        .virtual_path
        .extension()
        .is_some_and(|extension| extension == "tsx")
    {
        SourceType::tsx()
    } else {
        SourceType::ts()
    };
    let aliases = project.dependency_alias_map();
    crate::batch::ImportRewriter::new()
        .collect_all_specifier_occurrences(&source, source_type)
        .into_iter()
        .filter(|(specifier, _)| {
            !specifier.starts_with('.')
                && !Path::new(specifier.as_str()).is_absolute()
                && !crate::batch::virtual_project::is_vue_runtime_support_specifier(specifier)
                && crate::batch::virtual_project::dependency_scan::resolve_dependency(
                    specifier,
                    file.original_path.parent().unwrap_or(&file.original_path),
                    project.project_root(),
                    &aliases,
                )
                .is_none()
        })
        .collect()
}

fn alias_matches(pattern: &str, specifier: &str) -> bool {
    pattern
        .split_once('*')
        .map_or(pattern == specifier, |(prefix, suffix)| {
            !suffix.contains('*')
                && specifier
                    .strip_prefix(prefix)
                    .is_some_and(|rest| rest.ends_with(suffix))
        })
}

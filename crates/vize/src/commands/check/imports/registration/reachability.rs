//! CLI reachability without discarded relative and absolute probe histories.
use super::{
    CanonicalPathCache, ImportFileOptions, Path, PathAliasResolver, is_relative_specifier,
    resolve_import_base, resolve_import_base_with_inputs,
};

/// Bounded Vue reachability for one package route, using the same local and
/// package resolution the check import walk uses for authored sources.
///
/// The sole production caller passes false with the scoped untracked sources
/// constructed by VirtualRegistrationCache::reachability_sources. True keeps
/// the former path-history behavior for differential and tracked-input controls.
/// Bare alias proof histories and package invalidation inputs are always retained.
pub(super) fn scan_route_reachability<const RECORD_INPUTS: bool>(
    route: &vize_canon::PackageRoute,
    sources: &mut vize_canon::batch::ReachabilitySources,
    canonical_paths: &mut CanonicalPathCache,
    options: ImportFileOptions,
    aliases: Option<&PathAliasResolver>,
    packages: &mut vize_canon::PackageRouteResolver,
) -> vize_canon::batch::PackageRouteReachability {
    vize_canon::batch::scan_package_route_reachability(
        route,
        sources,
        |importer, nested_specifier| {
            resolve_local::<RECORD_INPUTS>(
                importer,
                nested_specifier,
                canonical_paths,
                options,
                aliases,
            )
        },
        |importer, nested_specifier, nested_mode| {
            let nested_dir = importer.parent().unwrap_or(importer);
            let (nested_context, mut nested_inputs) = match aliases {
                Some(aliases) => {
                    aliases.package_resolution_context(packages, importer, nested_mode)
                }
                None => packages.resolution_context(
                    importer,
                    nested_mode,
                    None,
                    None,
                    std::iter::empty::<vize_l0::String>(),
                ),
            };
            let (nested, consulted) = packages
                .lookup_with_context(
                    nested_dir,
                    nested_specifier,
                    vize_canon::PackageSourceOptions::new(options.include_js, options.include_jsx),
                    nested_context,
                )
                .into_parts();
            nested_inputs.extend(consulted);
            (nested, nested_inputs)
        },
    )
}

fn resolve_local<const RECORD_INPUTS: bool>(
    importer: &Path,
    specifier: &str,
    canonical_paths: &mut CanonicalPathCache,
    options: ImportFileOptions,
    aliases: Option<&PathAliasResolver>,
) -> (Option<std::path::PathBuf>, Vec<std::path::PathBuf>) {
    let dir = importer.parent().unwrap_or(importer);
    let mut resolve_base = |candidate: &Path| {
        if RECORD_INPUTS {
            resolve_import_base_with_inputs(candidate, canonical_paths, options)
        } else {
            (
                resolve_import_base(candidate, canonical_paths, options),
                Vec::new(),
            )
        }
    };
    if is_relative_specifier(specifier) {
        return resolve_base(&dir.join(specifier));
    }
    let candidate = Path::new(specifier);
    if candidate.is_absolute() {
        return resolve_base(candidate);
    }
    // Bare answers (including consulted paths) also govern cross-program
    // AliasProbe agreement, even when final invalidation inputs are untracked.
    aliases.map_or_else(
        || (None, Vec::new()),
        |aliases| {
            aliases.resolve_with_inputs(
                specifier,
                canonical_paths,
                options,
                resolve_import_base_with_inputs,
            )
        },
    )
}

#[cfg(test)]
mod tests;

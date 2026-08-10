//! Relative and tsconfig-path resolution used by dependency reachability.

use std::path::{Component, Path, PathBuf};

use vize_carton::{String as CompactString, cstr};

/// First-party classification key: workspace symlinks use physical identity.
pub(super) fn canonical_key(path: &Path) -> Option<PathBuf> {
    let canonical = vize_carton::path::canonicalize_non_verbatim(path);
    canonical.is_file().then_some(canonical)
}

pub(super) fn inside_node_modules(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::Normal(part) if part == "node_modules"))
}

pub(super) fn is_declaration_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        })
}

#[allow(clippy::disallowed_types)]
pub(super) fn alias_may_reach_first_party(
    pattern: &str,
    target: &str,
    project_root: &Path,
) -> bool {
    if pattern.contains('*') {
        return true;
    }
    let absolute = if Path::new(target).is_absolute() {
        PathBuf::from(target)
    } else {
        project_root.join(target)
    };
    let Some(resolved) = probe_candidates(&absolute) else {
        return false;
    };
    let Some(key) = canonical_key(&resolved) else {
        return false;
    };
    !inside_node_modules(&key) && !is_declaration_file(&key)
}

pub(super) fn may_resolve_a_dependency(
    content: &str,
    alias_prefixes: &[CompactString],
    package_routes: bool,
) -> bool {
    crate::batch::import_rewriter::source_may_contain_relative_specifier(content)
        || alias_prefixes
            .iter()
            .any(|prefix| prefix.is_empty() || content.contains(prefix.as_str()))
        || package_routes && may_contain_bare_specifier(content)
}

/// Whether `content` can name a bare specifier — the only shape a package
/// route resolves.
///
/// The prefilter exists to keep the walk from parsing every generated module
/// (#3898), so a package resolver must not widen it to "contains ` from `":
/// nearly every module does, which is the whole prefilter gone. The character
/// after the opening quote decides instead — a relative or rooted specifier is
/// already covered by [`source_may_contain_relative_specifier`] and the alias
/// prefixes. Still a conservative superset: the lead-in is matched anywhere in
/// the text, and a bare specifier naming a published package survives here and
/// is rejected by the resolver (which memoizes that answer).
fn may_contain_bare_specifier(content: &str) -> bool {
    ["from", "import", "require"].iter().any(|lead_in| {
        content.match_indices(lead_in).any(|(index, _)| {
            let after =
                content[index + lead_in.len()..].trim_start_matches([' ', '\t', '\r', '\n', '(']);
            after
                .strip_prefix(['\'', '"'])
                .is_some_and(|specifier| !specifier.starts_with(['.', '/', '\'', '"']))
        })
    })
}

/// Resolve one specifier to a registrable first-party file, or `None`.
#[allow(clippy::disallowed_types)]
pub(crate) fn resolve_dependency(
    specifier: &str,
    importer_dir: &Path,
    project_root: &Path,
    aliases: &[(std::string::String, std::string::String)],
) -> Option<PathBuf> {
    let specifier = specifier
        .strip_suffix(".vue.ts")
        .map_or_else(|| specifier.to_owned(), |stem| cstr!("{stem}.vue").into());
    if specifier.starts_with("./") || specifier.starts_with("../") {
        return probe_candidates(&importer_dir.join(&specifier));
    }

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
        // Length first: a pattern that cannot win must not cost a probe.
        if best.as_ref().is_none_or(|(len, _)| pattern.len() > *len)
            && let Some(resolved) = probe_candidates(&absolute)
        {
            best = Some((pattern.len(), resolved));
        }
    }
    best.map(|(_, path)| path)
}

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

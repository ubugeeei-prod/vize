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
    workspace_package_specifiers: &[CompactString],
) -> bool {
    crate::batch::import_rewriter::source_may_contain_relative_specifier(content)
        || alias_prefixes
            .iter()
            .any(|prefix| prefix.is_empty() || content.contains(prefix.as_str()))
        || workspace_package_specifiers
            .iter()
            .any(|specifier| content.contains(specifier.as_str()))
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

pub(super) fn probe_candidates(base: &Path) -> Option<PathBuf> {
    if base.is_file() {
        return Some(base.to_path_buf());
    }
    let extension = base.extension().and_then(|extension| extension.to_str());
    let probe_base = match extension {
        Some("js" | "jsx" | "mjs" | "cjs") => base.with_extension(""),
        Some(_) => return None,
        None => base.to_path_buf(),
    };
    for extension in ["ts", "tsx", "mts", "cts", "vue", "js", "jsx", "mjs", "cjs"] {
        let candidate = PathBuf::from(cstr!("{}.{extension}", probe_base.display()).as_str());
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    if extension.is_some() {
        return None;
    }
    for index in [
        "index.ts",
        "index.tsx",
        "index.mts",
        "index.cts",
        "index.vue",
        "index.js",
        "index.jsx",
        "index.mjs",
        "index.cjs",
    ] {
        let candidate = base.join(index);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

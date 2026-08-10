//! Relative specifier rendering and rename-aware path operations.

use std::path::{Component, Path, PathBuf};

use tower_lsp::lsp_types::Url;

use crate::server::ServerState;

use super::RenameTarget;

pub(in crate::ide::file_rename) const RESOLVABLE_SCRIPT_EXTENSIONS: &[&str] = &[
    "ts", "tsx", "d.ts", "d.mts", "d.cts", "js", "jsx", "mts", "cts", "mjs", "cjs", "vue",
];
#[derive(Clone, Copy)]
pub(in crate::ide::file_rename) enum RenderStyle {
    Explicit,
    Extensionless,
    DirectoryIndex,
}

#[derive(Clone)]
struct SpecifierCandidate {
    resolved: PathBuf,
    style: RenderStyle,
}

pub(in crate::ide::file_rename) fn rewrite_relative_specifier(
    state: &ServerState,
    current_importer_dir: &Path,
    future_importer_dir: &Path,
    specifier: &str,
    rename_targets: &[RenameTarget],
) -> Option<std::string::String> {
    let (specifier_path, suffix) = split_specifier_suffix(specifier);
    if !specifier_path.starts_with("./") && !specifier_path.starts_with("../") {
        return None;
    }

    let mut selected = None;
    for candidate in specifier_candidates(current_importer_dir, specifier_path) {
        if candidate_exists(state, &candidate.resolved)
            || apply_all_path_renames(&candidate.resolved, rename_targets).is_some()
        {
            selected = Some(candidate);
            break;
        }
    }

    let selected = selected?;
    let future_target = apply_all_path_renames(&selected.resolved, rename_targets)
        .unwrap_or_else(|| normalize_path_buf(&selected.resolved));

    if future_target == normalize_path_buf(&selected.resolved)
        && normalize_path_buf(current_importer_dir) == normalize_path_buf(future_importer_dir)
    {
        return None;
    }

    let mut rewritten =
        render_module_specifier(future_importer_dir, &future_target, selected.style)?;
    rewritten.push_str(suffix);
    Some(rewritten)
}

pub(in crate::ide::file_rename) fn render_module_specifier(
    importer_dir: &Path,
    target_path: &Path,
    style: RenderStyle,
) -> Option<std::string::String> {
    let rendered_target = match style {
        RenderStyle::Explicit => normalize_path_buf(target_path),
        RenderStyle::Extensionless => strip_extension(target_path),
        RenderStyle::DirectoryIndex => {
            if is_index_file(target_path) {
                normalize_path_buf(target_path.parent()?)
            } else {
                strip_extension(target_path)
            }
        }
    };

    relative_module_path(importer_dir, &rendered_target)
}

pub(in crate::ide::file_rename) fn relative_module_path(
    from_dir: &Path,
    to_path: &Path,
) -> Option<std::string::String> {
    let from_dir = normalize_path_buf(from_dir);
    let to_path = normalize_path_buf(to_path);

    let from_components = from_dir.components().collect::<Vec<_>>();
    let to_components = to_path.components().collect::<Vec<_>>();

    let mut common = 0usize;
    while common < from_components.len()
        && common < to_components.len()
        && from_components[common] == to_components[common]
    {
        common += 1;
    }

    if common == 0
        && matches!(from_components.first(), Some(Component::Prefix(_)))
        && matches!(to_components.first(), Some(Component::Prefix(_)))
    {
        return None;
    }

    let mut parts = Vec::new();
    for _ in common..from_components.len() {
        parts.push("..".to_string());
    }
    for component in &to_components[common..] {
        let part = match component {
            Component::Normal(value) => value.to_string_lossy().to_string(),
            Component::CurDir => ".".to_string(),
            Component::ParentDir => "..".to_string(),
            Component::RootDir | Component::Prefix(_) => continue,
        };
        parts.push(part);
    }

    let joined = if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    };

    if joined.starts_with("../") || joined == ".." {
        Some(joined)
    } else if joined == "." {
        Some("./".to_string())
    } else if joined.starts_with("./") {
        Some(joined)
    } else {
        let mut prefixed = std::string::String::from("./");
        prefixed.push_str(&joined);
        Some(prefixed)
    }
}

fn specifier_candidates(importer_dir: &Path, specifier: &str) -> Vec<SpecifierCandidate> {
    let resolved = normalize_path_buf(&importer_dir.join(specifier));
    let mut candidates = Vec::new();

    if Path::new(specifier).extension().is_none() {
        for extension in RESOLVABLE_SCRIPT_EXTENSIONS {
            candidates.push(SpecifierCandidate {
                resolved: resolved.with_extension(extension),
                style: RenderStyle::Extensionless,
            });
        }

        for extension in RESOLVABLE_SCRIPT_EXTENSIONS {
            let mut index_name = std::string::String::from("index.");
            index_name.push_str(extension);
            candidates.push(SpecifierCandidate {
                resolved: resolved.join(index_name),
                style: RenderStyle::DirectoryIndex,
            });
        }

        candidates.push(SpecifierCandidate {
            resolved,
            style: RenderStyle::Explicit,
        });
    } else {
        candidates.push(SpecifierCandidate {
            resolved,
            style: RenderStyle::Explicit,
        });
    }

    candidates
}

pub(in crate::ide::file_rename) fn apply_all_path_renames(
    path: &Path,
    renames: &[RenameTarget],
) -> Option<PathBuf> {
    let mut updated = normalize_path_buf(path);
    let mut changed = false;

    for rename in renames {
        if let Some(next) = apply_path_rename(&updated, rename) {
            updated = next;
            changed = true;
        }
    }

    if changed { Some(updated) } else { None }
}

fn apply_path_rename(path: &Path, rename: &RenameTarget) -> Option<PathBuf> {
    let normalized = normalize_path_buf(path);
    if normalized == rename.old_path {
        return Some(rename.new_path.clone());
    }

    let suffix = normalized.strip_prefix(&rename.old_path).ok()?;
    if suffix.as_os_str().is_empty() {
        Some(rename.new_path.clone())
    } else {
        Some(normalize_path_buf(&rename.new_path.join(suffix)))
    }
}

pub(in crate::ide::file_rename) fn candidate_exists(state: &ServerState, path: &Path) -> bool {
    if path.exists() {
        return true;
    }

    let Ok(uri) = Url::from_file_path(path) else {
        return false;
    };

    state.documents.contains(&uri)
}

pub(in crate::ide::file_rename) fn split_specifier_suffix(specifier: &str) -> (&str, &str) {
    let split_at = specifier.find(['?', '#']).unwrap_or(specifier.len());
    (&specifier[..split_at], &specifier[split_at..])
}

pub(in crate::ide::file_rename) fn normalize_path_buf(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    normalized
}

pub(in crate::ide::file_rename) fn strip_extension(path: &Path) -> PathBuf {
    let mut stripped = normalize_path_buf(path);
    let path = stripped.to_string_lossy();
    let extension_depth = 1 + usize::from(
        path.ends_with(".d.ts") || path.ends_with(".d.mts") || path.ends_with(".d.cts"),
    );
    for _ in 0..extension_depth {
        let _ = stripped.set_extension("");
    }
    stripped
}

pub(in crate::ide::file_rename) fn is_index_file(path: &Path) -> bool {
    strip_extension(path).ends_with("index")
}

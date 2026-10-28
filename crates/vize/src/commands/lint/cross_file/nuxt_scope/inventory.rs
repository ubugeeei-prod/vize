//! Complete physical inventories for the first, flat Nuxt authority domain.
//! This does not change lint discovery, traverse subdirectories, or read source.

use std::path::{Component, Path, PathBuf};
use vize_croquis_cf::{ModuleRegistry, NuxtRenderDirs};
use vize_l0::FxHashSet;

use crate::commands::project_nuxt::host_authority::io::{Entry, EntryKind, ProbeIo};

const CONFIG_NAMES: [&str; 6] = [
    "nuxt.config.ts",
    "nuxt.config.js",
    "nuxt.config.mts",
    "nuxt.config.mjs",
    "nuxt.config.cts",
    "nuxt.config.cjs",
];

pub(super) fn complete(
    root: &Path,
    configured: &NuxtRenderDirs,
    registry: &ModuleRegistry,
    io: &mut ProbeIo,
) -> Option<()> {
    if io.canonical_directory(root, false)?.as_path() != root {
        return None;
    }
    let src = within(root, &configured.src)?;
    if src != root && src.parent() != Some(root) {
        return None;
    }
    let pages = within(root, &configured.pages)?;
    let layouts = within(root, &configured.layouts)?;
    if pages.parent() != Some(src.as_path())
        || layouts.parent() != Some(src.as_path())
        || pages == layouts
    {
        return None;
    }
    for path in [&src, &pages, &layouts] {
        if io.canonical_directory(path, false)?.as_path() != path.as_path() {
            return None;
        }
    }
    let root_entries = io.directory(root)?;
    top_level(root, root, &src, &pages, &layouts, &root_entries, io)?;
    let src_entries = if src == root {
        root_entries
    } else {
        io.directory(&src)?
    };
    if src != root {
        top_level(&src, root, &src, &pages, &layouts, &src_entries, io)?;
    }
    let mut apps = src_entries
        .iter()
        .filter(|entry| matches!(entry.name.to_str(), Some("app.vue" | "App.vue")));
    let app = apps.next()?;
    if apps.next().is_some() || app.kind != EntryKind::File {
        return None;
    }

    let selected: FxHashSet<_> = registry
        .vue_components()
        .map(|entry| entry.path.as_path())
        .collect();
    if !selected.contains(src.join(&app.name).as_path()) {
        return None;
    }
    for directory in [&pages, &layouts] {
        for entry in io.directory(directory)? {
            // Physical subdirectories, unknown suffixes and links make this
            // intentionally narrow domain unknown. No filename alone grants a
            // route/render relationship.
            if entry.kind != EntryKind::File || !static_vue_name(entry.name.to_str()?) {
                return None;
            }
            if !selected.contains(directory.join(&entry.name).as_path()) {
                return None;
            }
        }
    }
    Some(())
}

fn within(root: &Path, path: &Path) -> Option<PathBuf> {
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return None;
    }
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    resolved.starts_with(root).then_some(resolved)
}

fn top_level(
    directory: &Path,
    root: &Path,
    src: &Path,
    pages: &Path,
    layouts: &Path,
    entries: &[Entry],
    io: &mut ProbeIo,
) -> Option<()> {
    let mut configs = 0;
    for entry in entries {
        let name = entry.name.to_str()?;
        let path = directory.join(&entry.name);
        if name == "node_modules" {
            io.canonical_directory(&path, true)?;
            continue;
        }
        if entry.kind == EntryKind::Unknown {
            return None;
        }
        if entry.kind == EntryKind::Directory {
            if (directory == root && path == src)
                || (directory == src && (path == pages || path == layouts))
                || matches!(name, ".git" | ".nuxt" | ".output" | "assets" | "public")
            {
                continue;
            }
            return None;
        }
        if CONFIG_NAMES.contains(&name) {
            if directory != root {
                return None;
            }
            configs += 1;
            continue;
        }
        if directory == src && matches!(name, "app.vue" | "App.vue") {
            continue;
        }
        // These can add loader, runtime, or route inputs outside the closed
        // original config. Presence is unknown; their content is never read.
        if name.starts_with(".env")
            || name.starts_with(".nuxtrc")
            || name.starts_with(".nuxtignore")
            || name.starts_with(".pnp")
            || name.starts_with("nuxt.config")
            || name.starts_with("router.options")
            || name.starts_with("app.config")
            || matches!(name, ".npmrc" | "pnpm-workspace.yaml" | "app.html")
            || Path::new(name)
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|extension| {
                    matches!(
                        extension,
                        "vue" | "js" | "mjs" | "cjs" | "ts" | "mts" | "cts" | "jsx" | "tsx"
                    )
                })
        {
            return None;
        }
    }
    if directory == root && configs != 1 {
        return None;
    }
    Some(())
}

fn static_vue_name(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".vue") else {
        return false;
    };
    !stem.is_empty()
        && stem.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

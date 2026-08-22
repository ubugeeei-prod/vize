//! Rename-time rewriting of `paths`-aliased import specifiers (#3917).
//!
//! The manual scanner rewrote only relative specifiers, so moving a file left
//! every `@/…`-style import pointing at the old path — exactly the imports
//! that survive refactors longest. The alias map is anchored the way the
//! session anchors it (#3915): nearest `tsconfig.json`, following a
//! solution-style shell's `references` to the first config that declares
//! `paths`. Style is preserved per specifier: an aliased import stays aliased
//! when the moved file remains under the alias target, and falls back to a
//! relative spelling when it leaves the subtree — the oracle's behavior.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use std::path::{Path, PathBuf};

use super::manual::{
    RESOLVABLE_SCRIPT_EXTENSIONS, RenameTarget, RenderStyle, apply_all_path_renames,
    candidate_exists, is_index_file, normalize_path_buf, render_module_specifier,
    split_specifier_suffix, strip_extension,
};
use crate::server::ServerState;

/// One effective alias: the pattern (`@/*`), and the target's base directory
/// resolved against its declaring config (`<anchor>/src`).
struct AliasEntry {
    pattern_prefix: std::string::String,
    target_base: PathBuf,
}

/// Rewrite one specifier for the pending renames: relative specifiers via the
/// manual scanner's rules, aliased ones via the `paths` map. `None` leaves the
/// specifier untouched.
pub(super) fn rewrite_specifier(
    state: &ServerState,
    importer_path: &Path,
    future_importer_dir: &Path,
    specifier: &str,
    rename_targets: &[RenameTarget],
) -> Option<std::string::String> {
    super::manual::rewrite_relative_specifier(
        state,
        importer_path.parent()?,
        future_importer_dir,
        specifier,
        rename_targets,
    )
    .or_else(|| {
        rewrite_alias_specifier(
            state,
            importer_path,
            future_importer_dir,
            specifier,
            rename_targets,
        )
    })
}

/// Rewrite one non-relative specifier whose alias-resolved target is being
/// renamed. `None` leaves the specifier untouched.
fn rewrite_alias_specifier(
    state: &ServerState,
    importer_path: &Path,
    future_importer_dir: &Path,
    specifier: &str,
    rename_targets: &[RenameTarget],
) -> Option<std::string::String> {
    let (specifier_path, suffix) = split_specifier_suffix(specifier);
    if specifier_path.starts_with("./") || specifier_path.starts_with("../") {
        return None;
    }

    let mut aliases = project_aliases(importer_path)?;
    // Longest pattern wins, mirroring TypeScript's `paths` selection.
    aliases.sort_by_key(|alias| std::cmp::Reverse(alias.pattern_prefix.len()));

    for alias in &aliases {
        let Some(rest) = specifier_path.strip_prefix(alias.pattern_prefix.as_str()) else {
            continue;
        };
        let base = normalize_path_buf(&alias.target_base.join(rest));
        let Some((resolved, style)) = probe_alias_target(state, &base, rename_targets) else {
            continue;
        };
        let future = apply_all_path_renames(&resolved, rename_targets)?;

        if let Ok(remainder) = future.strip_prefix(&alias.target_base) {
            let mut rendered = alias.pattern_prefix.clone();
            rendered.push_str(&render_alias_remainder(remainder, style));
            rendered.push_str(suffix);
            return Some(rendered);
        }
        // The file left the alias subtree: no alias spelling exists any more,
        // so fall back to a relative path from the importer.
        let mut rendered = render_module_specifier(future_importer_dir, &future, style)?;
        rendered.push_str(suffix);
        return Some(rendered);
    }
    None
}

/// The alias-relative spelling of a renamed target, in the style the original
/// specifier used. A directory-index import keeps that spelling as long as the
/// moved file is still an `index.*`; `@/` alone is not a module, so a target
/// landing directly on the alias root degrades to the extensionless spelling.
fn render_alias_remainder(remainder: &Path, style: RenderStyle) -> std::string::String {
    let rendered = match style {
        RenderStyle::Explicit => normalize_path_buf(remainder),
        RenderStyle::Extensionless => strip_extension(remainder),
        RenderStyle::DirectoryIndex if is_index_file(remainder) => remainder
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map_or_else(|| strip_extension(remainder), normalize_path_buf),
        RenderStyle::DirectoryIndex => strip_extension(remainder),
    };
    rendered.to_string_lossy().replace('\\', "/")
}

/// Resolve the alias-substituted base to a real (or being-renamed) file: exact
/// when the specifier carries an extension, otherwise extension probing and
/// then directory-index probing, mirroring the relative scanner's candidate
/// order. The style decides how the rename is re-spelled.
fn probe_alias_target(
    state: &ServerState,
    base: &Path,
    rename_targets: &[RenameTarget],
) -> Option<(PathBuf, RenderStyle)> {
    let resolvable = |path: &Path| {
        candidate_exists(state, path) || apply_all_path_renames(path, rename_targets).is_some()
    };
    if base.extension().is_some() {
        return resolvable(base).then(|| (base.to_path_buf(), RenderStyle::Explicit));
    }
    for extension in RESOLVABLE_SCRIPT_EXTENSIONS {
        let candidate = base.with_extension(extension);
        if resolvable(&candidate) {
            return Some((candidate, RenderStyle::Extensionless));
        }
    }
    for extension in RESOLVABLE_SCRIPT_EXTENSIONS {
        let mut index_name = std::string::String::from("index.");
        index_name.push_str(extension);
        let candidate = base.join(index_name);
        if resolvable(&candidate) {
            return Some((candidate, RenderStyle::DirectoryIndex));
        }
    }
    None
}

/// The `paths` aliases governing `source_path`, resolved by the shared
/// reader (nearest tsconfig, references-follow, string-aware jsonc).
fn project_aliases(source_path: &Path) -> Option<Vec<AliasEntry>> {
    let mut aliases = Vec::new();
    if let Some(paths) = crate::ide::tsconfig_paths::project_paths(source_path) {
        for (pattern, target) in &paths.entries {
            // Exact-match aliases name one module; a rename cannot re-spell them.
            let (Some(pattern_prefix), Some(target)) =
                (pattern.strip_suffix('*'), target.strip_suffix('*'))
            else {
                continue;
            };
            aliases.push(AliasEntry {
                pattern_prefix: pattern_prefix.to_string(),
                target_base: normalize_path_buf(&paths.anchor.join(target)),
            });
        }
    }
    if let Some(root) = nearest_nuxt_root(source_path) {
        for pattern_prefix in ["~/", "@/"] {
            for target_base in [root.join("app"), root.clone(), root.join("src")] {
                aliases.push(AliasEntry {
                    pattern_prefix: pattern_prefix.to_string(),
                    target_base: normalize_path_buf(&target_base),
                });
            }
        }
    }
    (!aliases.is_empty()).then_some(aliases)
}

fn nearest_nuxt_root(file: &Path) -> Option<PathBuf> {
    file.ancestors()
        .skip(1)
        .find(|dir| {
            [
                "nuxt.config.ts",
                "nuxt.config.mts",
                "nuxt.config.js",
                "nuxt.config.mjs",
            ]
            .iter()
            .any(|config| dir.join(config).is_file())
        })
        .map(Path::to_path_buf)
}

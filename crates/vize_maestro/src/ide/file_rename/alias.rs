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
    RESOLVABLE_SCRIPT_EXTENSIONS, RenameTarget, apply_all_path_renames, candidate_exists,
    normalize_path_buf, relative_module_path, split_specifier_suffix, strip_extension,
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
        let Some((resolved, extensionless)) = probe_alias_target(state, &base, rename_targets)
        else {
            continue;
        };
        let future = apply_all_path_renames(&resolved, rename_targets)?;

        if let Ok(remainder) = future.strip_prefix(&alias.target_base) {
            let mut rendered = alias.pattern_prefix.clone();
            let remainder = if extensionless {
                strip_extension(remainder)
            } else {
                remainder.to_path_buf()
            };
            rendered.push_str(&remainder.to_string_lossy().replace('\\', "/"));
            rendered.push_str(suffix);
            return Some(rendered);
        }
        // The file left the alias subtree: no alias spelling exists any more,
        // so fall back to a relative path from the importer.
        let rendered_target = if extensionless {
            strip_extension(&future)
        } else {
            future
        };
        let mut rendered = relative_module_path(future_importer_dir, &rendered_target)?;
        rendered.push_str(suffix);
        return Some(rendered);
    }
    None
}

/// Resolve the alias-substituted base to a real (or being-renamed) file:
/// exact when the specifier carries an extension, extension probing otherwise.
/// The flag reports whether the specifier was extensionless, which decides the
/// rendered style.
fn probe_alias_target(
    state: &ServerState,
    base: &Path,
    rename_targets: &[RenameTarget],
) -> Option<(PathBuf, bool)> {
    let resolvable = |path: &Path| {
        candidate_exists(state, path) || apply_all_path_renames(path, rename_targets).is_some()
    };
    if base.extension().is_some() {
        return resolvable(base).then(|| (base.to_path_buf(), false));
    }
    for extension in RESOLVABLE_SCRIPT_EXTENSIONS {
        let candidate = base.with_extension(extension);
        if resolvable(&candidate) {
            return Some((candidate, true));
        }
    }
    None
}

/// The `paths` aliases governing `source_path`, resolved by the shared
/// reader (nearest tsconfig, references-follow, string-aware jsonc).
fn project_aliases(source_path: &Path) -> Option<Vec<AliasEntry>> {
    let paths = crate::ide::tsconfig_paths::project_paths(source_path)?;
    let mut aliases = Vec::new();
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
    (!aliases.is_empty()).then_some(aliases)
}

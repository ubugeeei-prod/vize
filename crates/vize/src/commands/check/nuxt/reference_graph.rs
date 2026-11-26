//! Installed declarations Nuxt's generated reference graph loads.
//!
//! `.nuxt/nuxt.d.ts` pulls part of the app's types in through triple-slash
//! references into installed packages. Nitro's `augments.d.mts` is one: it
//! types `useRequestEvent()`, `defineRouteRules()` and
//! `useRuntimeConfig().nitro`. Program input collection keeps installed
//! declarations out of the mirrored sources, because a rewritten copy of one
//! can shadow the package's own entry, so those augmentations never reached
//! the program and the names they declare reported TS2339 where tsc accepts
//! them. Each installed declaration of the graph the program does not hold
//! is loaded the way tsc loads it instead: by a reference to the installed
//! file, from the stub file every program file sees.

use std::path::{Path, PathBuf};

use vize_l0::{FxHashSet, String, cstr};

use super::super::tsconfig_inputs::normalize_input_path;

/// One `/// <reference path>` stub for each installed declaration in `graph`
/// that `program_files` does not already hold, in path order.
///
/// `shadowing` names the graph's declarations that would replace a Vue
/// package outright; input collection keeps those out of every program, and
/// they stay out here.
pub(super) fn installed_reference_stubs(
    graph: &FxHashSet<PathBuf>,
    shadowing: &FxHashSet<PathBuf>,
    program_files: &[PathBuf],
) -> Vec<String> {
    let mut candidates = graph
        .iter()
        .filter(|path| is_installed(path) && !shadowing.contains(*path))
        .map(|path| real_path(path))
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return Vec::new();
    }
    // Only installed inputs can match, so only those are canonicalized.
    let loaded = program_files
        .iter()
        .filter(|path| is_installed(path))
        .map(|path| real_path(path))
        .collect::<FxHashSet<_>>();
    candidates.sort();
    candidates.dedup();
    candidates
        .into_iter()
        .filter(|path| !loaded.contains(path))
        .filter_map(|path| {
            let path = path.to_str()?;
            // A path holding a quote cannot be spelled in the directive.
            (!path.contains('"')).then(|| cstr!("/// <reference path=\"{path}\" />"))
        })
        .collect()
}

pub(super) fn is_installed(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == "node_modules")
}

fn real_path(path: &Path) -> PathBuf {
    normalize_input_path(path)
}

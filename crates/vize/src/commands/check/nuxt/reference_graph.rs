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
    graph: &super::super::tsconfig_inputs::InstalledDeclarationGraph,
    shadowing: &FxHashSet<PathBuf>,
    program_files: &[PathBuf],
) -> Vec<String> {
    let mut candidates = graph
        .installed
        .iter()
        .filter(|path| !shadowing.contains(*path))
        .map(|path| real_path(path))
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return Vec::new();
    }
    // Canonical linked-package paths need their recorded installed membership.
    // Ordinary app inputs still never incur an extra canonicalization here.
    let loaded = program_files
        .iter()
        .filter(|path| graph.installed.contains(*path) || is_installed(path))
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

/// Keep a parent's declarations while preventing its nested references from
/// loading a bare Vue shim. Sources remain untouched; Canon registers these
/// snapshots in its owned mirror and routes references to the snapshots.
pub(super) fn safe_declaration_snapshots(
    graph: &super::super::tsconfig_inputs::InstalledDeclarationGraph,
    shadowing: &FxHashSet<PathBuf>,
) -> Result<Vec<(PathBuf, String)>, String> {
    if shadowing.is_empty() {
        return Ok(Vec::new());
    }
    let mut reverse = vize_l0::FxHashMap::<&PathBuf, Vec<&PathBuf>>::default();
    for (parent, targets) in &graph.references {
        for target in targets {
            reverse.entry(target).or_default().push(parent);
        }
    }
    let mut pending = shadowing.iter().collect::<Vec<_>>();
    let mut affected = FxHashSet::default();
    while let Some(target) = pending.pop() {
        for parent in reverse.get(target).into_iter().flatten() {
            if affected.insert(*parent) {
                pending.push(parent);
            }
        }
    }
    let mut affected = affected
        .into_iter()
        .filter(|path| !shadowing.contains(*path))
        .collect::<Vec<_>>();
    affected.sort();
    affected
        .into_iter()
        .map(|path| {
            let source = std::fs::read_to_string(path).map_err(|error| {
                cstr!(
                    "Cannot read declaration snapshot {}: {error}",
                    path.display()
                )
            })?;
            let targets = graph
                .references
                .get(path)
                .ok_or_else(|| cstr!("Missing declaration references for {}", path.display()))?;
            let mut snapshot = String::default();
            for target in targets.iter().filter(|target| !shadowing.contains(*target)) {
                snapshot.push_str(&snapshot_reference(target)?);
            }
            for line in source.split_inclusive('\n') {
                let trimmed = line.trim_start();
                let directive = trimmed.starts_with("///")
                    && trimmed.contains("<reference")
                    && (trimmed.contains("path=") || trimmed.contains("types="));
                if !directive {
                    snapshot.push_str(line);
                }
            }
            Ok((path.clone(), snapshot))
        })
        .collect()
}

fn snapshot_reference(target: &Path) -> Result<String, String> {
    let target = target.to_str().ok_or_else(|| {
        cstr!(
            "Declaration reference path is not UTF-8: {}",
            target.display()
        )
    })?;
    if target.contains(['\r', '\n', '\0', '\u{2028}', '\u{2029}']) {
        return Err(cstr!(
            "Declaration reference path contains an unrepresentable character: {target:?}"
        ));
    }
    let quote = if target.contains('"') { '\'' } else { '"' };
    if target.contains(quote) {
        return Err(cstr!(
            "Declaration reference path contains both quote delimiters: {target:?}"
        ));
    }
    Ok(cstr!("/// <reference path={quote}{target}{quote} />\n"))
}

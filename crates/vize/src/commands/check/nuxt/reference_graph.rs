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

use vize_l0::{FxHashMap, FxHashSet, String, cstr};

use super::super::tsconfig_inputs::{
    DeclarationReference, normalize_input_path, reference_attribute,
};

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
    snapshots: &[(PathBuf, String)],
) -> Vec<String> {
    let snapshots = snapshots
        .iter()
        .map(|(path, _)| path)
        .collect::<FxHashSet<_>>();
    let mut candidates = graph
        .installed
        .iter()
        .filter(|path| !shadowing.contains(*path))
        .map(|path| {
            let identity = real_path(path);
            let ordinary = !snapshots.contains(path);
            let emitted = if ordinary {
                identity.clone()
            } else {
                path.clone()
            };
            (identity, ordinary, emitted)
        })
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
    candidates.dedup_by(|left, right| left.0 == right.0);
    candidates
        .into_iter()
        .filter(|(identity, _, _)| !loaded.contains(identity))
        .filter_map(|(_, _, path)| {
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
            let directives = graph
                .directives
                .get(path)
                .into_iter()
                .flatten()
                .map(|reference| (reference.key(), reference))
                .collect::<FxHashMap<_, _>>();
            let mut snapshot = String::default();
            for line in source.split_inclusive('\n') {
                if let Some(rewritten) = snapshot_directive(line, &directives, shadowing)? {
                    snapshot.push_str(&rewritten);
                } else {
                    snapshot.push_str(line);
                }
            }
            Ok((path.clone(), snapshot))
        })
        .collect()
}

fn snapshot_directive(
    line: &str,
    directives: &FxHashMap<(&str, &str), &DeclarationReference>,
    shadowing: &FxHashSet<PathBuf>,
) -> Result<Option<String>, String> {
    for attribute in ["path", "types"] {
        let Some(specifier) = reference_attribute(line, attribute) else {
            continue;
        };
        let reference = directives
            .get(&(attribute, specifier))
            .ok_or_else(|| cstr!("Declaration reference changed while snapshotting: {line:?}"))?;
        return match reference {
            DeclarationReference::Path { target, .. } => {
                if shadowing.contains(target) {
                    return Ok(Some(String::default()));
                }
                let start = line
                    .find("path=")
                    .ok_or_else(|| cstr!("Missing path attribute: {line:?}"))?
                    + "path=".len();
                let end = start + specifier.len() + 2;
                let before = line
                    .get(..start)
                    .ok_or_else(|| cstr!("Invalid reference path boundary: {line:?}"))?;
                let after = line
                    .get(end..)
                    .ok_or_else(|| cstr!("Invalid reference path boundary: {line:?}"))?;
                let (quote, target) = quoted_reference_path(target)?;
                Ok(Some(cstr!("{before}{quote}{target}{quote}{after}")))
            }
            DeclarationReference::Types { targets, .. } => {
                if targets.is_empty() {
                    return Ok(None);
                }
                let mut rewritten = String::default();
                for target in targets.iter().filter(|target| !shadowing.contains(*target)) {
                    rewritten.push_str(&snapshot_reference(target)?);
                }
                Ok(Some(rewritten))
            }
        };
    }
    Ok(None)
}

fn snapshot_reference(target: &Path) -> Result<String, String> {
    let (quote, target) = quoted_reference_path(target)?;
    Ok(cstr!("/// <reference path={quote}{target}{quote} />\n"))
}

fn quoted_reference_path(target: &Path) -> Result<(char, &str), String> {
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
    Ok((quote, target))
}

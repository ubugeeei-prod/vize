//! Whether an editor project's declarations own the global `useTemplateRef`.
//!
//! `vize check` decides this from the program's declaration graph. Editor
//! projects load the workspace declarations the server discovered, so they
//! answer the same question from those files and everything they reference:
//! Nuxt's `.nuxt/types/imports.d.ts` is reached only through `nuxt.d.ts`'s
//! `/// <reference path>`. Without it, an auto-imported `useTemplateRef` kept
//! Vue's untyped signature in the editor while the CLI typed it.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use vize_carton::{FxHashMap, FxHashSet};

/// Declarations a project can reach before the walk stops looking.
const MAX_DECLARATIONS: usize = 4096;

/// What one declaration says, cached by its modification time: discovery hands
/// every rebuild the same files, and they rarely change.
#[derive(Clone)]
struct DeclarationFacts {
    modified: Option<SystemTime>,
    declares: bool,
    references: Vec<PathBuf>,
}

fn cache() -> &'static Mutex<FxHashMap<PathBuf, DeclarationFacts>> {
    static CACHE: OnceLock<Mutex<FxHashMap<PathBuf, DeclarationFacts>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// Whether a declaration the editor loads (`reference_paths`, open buffers
/// first), or one it references, declares the global as Vue's own export, or
/// a framework supplied the declaration as an auto-import stub.
pub(super) fn editor_declares_vue_template_ref(
    options: &crate::virtual_ts::VirtualTsOptions,
    overlays: &FxHashMap<PathBuf, &str>,
) -> bool {
    if options
        .auto_import_stubs
        .iter()
        .any(|stub| crate::batch::declares_vue_template_ref_global(stub))
    {
        return true;
    }
    let mut pending: Vec<PathBuf> = options
        .reference_paths
        .iter()
        .map(|path| PathBuf::from(path.as_str()))
        .collect();
    let mut seen: FxHashSet<PathBuf> = pending.iter().cloned().collect();
    while let Some(path) = pending.pop() {
        if seen.len() > MAX_DECLARATIONS {
            break;
        }
        let Some(facts) = declaration_facts(&path, overlays) else {
            continue;
        };
        if facts.declares {
            return true;
        }
        for reference in facts.references {
            if seen.insert(reference.clone()) {
                pending.push(reference);
            }
        }
    }
    false
}

fn declaration_facts(path: &Path, overlays: &FxHashMap<PathBuf, &str>) -> Option<DeclarationFacts> {
    if let Some(text) = overlays
        .get(path)
        .copied()
        .or_else(|| overlays.get(&vize_l0::path::physical_path(path)).copied())
    {
        // Unsaved bytes are authoritative and never cached.
        return Some(read_facts(path, text, None));
    }
    let modified = std::fs::metadata(path).ok()?.modified().ok();
    if let Ok(cache) = cache().lock()
        && let Some(facts) = cache.get(path)
        && facts.modified == modified
        && modified.is_some()
    {
        return Some(facts.clone());
    }
    let text = std::fs::read_to_string(path).ok()?;
    let facts = read_facts(path, &text, modified);
    if let Ok(mut cache) = cache().lock() {
        cache.insert(path.to_path_buf(), facts.clone());
    }
    Some(facts)
}

fn read_facts(path: &Path, text: &str, modified: Option<SystemTime>) -> DeclarationFacts {
    DeclarationFacts {
        modified,
        declares: crate::batch::declares_vue_template_ref_global(text),
        references: reference_paths(path, text),
    }
}

/// The `/// <reference path="…" />` targets of a declaration's header, which
/// TypeScript reads only before the first statement.
fn reference_paths(path: &Path, text: &str) -> Vec<PathBuf> {
    let directory = path.parent().unwrap_or(path);
    text.lines()
        .map(str::trim)
        // Comments of either kind may precede the directives.
        .take_while(|line| {
            line.is_empty()
                || line.starts_with("//")
                || line.starts_with("/*")
                || line.starts_with('*')
        })
        .filter_map(|line| {
            let directive = line.strip_prefix("///")?.trim_start();
            let rest = directive.strip_prefix("<reference")?;
            let (_, after) = rest.split_once("path")?;
            let after = after.trim_start().strip_prefix('=')?.trim_start();
            let quote = after
                .chars()
                .next()
                .filter(|quote| matches!(quote, '"' | '\''))?;
            let (target, _) = after.get(1..)?.split_once(quote)?;
            Some(directory.join(target))
        })
        .filter(|target| {
            target
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::reference_paths;
    use std::path::{Path, PathBuf};

    #[test]
    fn header_reference_paths_resolve_beside_the_declaration() {
        let text = "/* generated */\n/**\n * banner\n */\n/// <reference types=\"vue\" />\n/// <reference path=\"types/imports.d.ts\" />\n/// <reference path='./plugins.d.ts' />\n\nexport {}\n/// <reference path=\"late.d.ts\" />\n";
        assert_eq!(
            reference_paths(Path::new("/app/.nuxt/nuxt.d.ts"), text),
            vec![
                PathBuf::from("/app/.nuxt/types/imports.d.ts"),
                PathBuf::from("/app/.nuxt/./plugins.d.ts"),
            ]
        );
    }
}

//! Refuse new splitting when external module loads may carry ambient globals.

use std::path::Path;

use super::{VirtualFile, VirtualProject, normalize_join, resolve_virtual_import};
use vize_carton::FxHashMap;

pub(super) fn permits_sharing(
    project: &VirtualProject,
    files: &[&VirtualFile],
    index_by_virtual: &FxHashMap<&Path, usize>,
) -> bool {
    files.iter().all(|file| {
        let source = project
            .original_content_for_virtual(&file.virtual_path)
            .unwrap_or(&file.content);
        if source.contains("<reference") || source.contains('\\') {
            return false;
        }
        // Unlike the generated-code cost scan, accept trivia after keywords
        // and call parens. Over-matching comments/strings only declines an
        // optimization; no parsed module resolution or owner changes follow.
        for token in ["from", "import", "require"] {
            for (at, _) in source.match_indices(token) {
                let rest = skip_trivia(source.get(at + token.len()..).unwrap_or_default());
                let rest = rest.strip_prefix('(').map_or(rest, skip_trivia);
                let Some(quote) = rest
                    .chars()
                    .next()
                    .filter(|quote| matches!(quote, '\'' | '"'))
                else {
                    continue;
                };
                let Some((specifier, _)) = rest.get(1..).unwrap_or_default().split_once(quote)
                else {
                    return false;
                };
                if specifier.contains('\\') {
                    return false;
                }
                // The shared helper loaded by every checker program already
                // imports Vue. Any other bare/absolute dependency may augment
                // globals only in its importer's program, so decline splitting.
                if specifier == "vue" {
                    continue;
                }
                if !specifier.starts_with("./") && !specifier.starts_with("../") {
                    return false;
                }
                let Some(base) = file.virtual_path.parent() else {
                    return false;
                };
                let target = normalize_join(base, specifier);
                if resolve_virtual_import(&target, index_by_virtual).is_none() {
                    // Ambient and unregistered real-tree module visibility is
                    // unchanged by preserving the former component plan.
                    return false;
                }
            }
        }
        true
    })
}

fn skip_trivia(mut source: &str) -> &str {
    loop {
        source = source.trim_start();
        if let Some(rest) = source.strip_prefix("/*") {
            let Some((_, rest)) = rest.split_once("*/") else {
                return "";
            };
            source = rest;
        } else if let Some(rest) = source.strip_prefix("//") {
            let Some((_, rest)) = rest.split_once('\n') else {
                return "";
            };
            source = rest;
        } else {
            return source;
        }
    }
}

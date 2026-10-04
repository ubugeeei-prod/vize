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
        if source.contains("<reference")
            || source.contains('\\')
            || source.contains('&')
            || ["declare", "global", "namespace", "require"]
                .iter()
                .any(|syntax| source.contains(syntax))
        {
            // Template entities can decode into module loads or declarations
            // that an original-source lexical screen cannot prove harmless.
            return false;
        }
        if file
            .original_path
            .extension()
            .is_some_and(|extension| extension == "vue")
            && source.match_indices("export").any(|(at, _)| {
                !skip_trivia(source.get(at + "export".len()..).unwrap_or_default())
                    .strip_prefix("default")
                    .and_then(|rest| rest.chars().next())
                    .is_some_and(|next| {
                        next.is_ascii_whitespace()
                            || matches!(next, '(' | '{' | '[' | '\'' | '"' | '`')
                    })
            })
        {
            // Vue wrappers are modules, but parser recovery can retain a
            // malformed export's module load. Only normal default component
            // exports qualify; other exports retain the original cost plan.
            return false;
        }
        // Unlike the generated-code cost scan, accept trivia after keywords
        // and call parens. Over-matching comments/strings only declines an
        // optimization; no parsed module resolution or owner changes follow.
        for token in ["from", "import", "require"] {
            for (at, _) in source.match_indices(token) {
                let rest = skip_trivia(source.get(at + token.len()..).unwrap_or_default());
                if rest.starts_with(['.', '?', '<']) {
                    return false;
                }
                if token == "import" && (rest.starts_with("defer") || rest.starts_with("source")) {
                    return false;
                }
                let call = rest.starts_with('(');
                let rest = rest.strip_prefix('(').map_or(rest, skip_trivia);
                let Some(quote) = rest
                    .chars()
                    .next()
                    .filter(|quote| matches!(quote, '\'' | '"' | '`'))
                else {
                    if call {
                        // Computed, parenthesized or malformed operands are
                        // outside the statically proved module-load grammar.
                        return false;
                    }
                    if token == "import" {
                        // Native parser recovery can still load a module from
                        // malformed imports lacking `from`. Require the first
                        // quoted operand to follow the supported static form.
                        let Some(at) = rest.find(['\'', '"', '`']) else {
                            return false;
                        };
                        if !rest
                            .get(..at)
                            .unwrap_or_default()
                            .trim_end()
                            .ends_with("from")
                        {
                            return false;
                        }
                    }
                    continue;
                };
                let Some((specifier, _)) = rest.get(1..).unwrap_or_default().split_once(quote)
                else {
                    return false;
                };
                if specifier.contains('\\') || (quote == '`' && specifier.contains("${")) {
                    return false;
                }
                // The shared helper imports Vue from the virtual root. Prove
                // the same backend directory and module mode before sharing
                // that exemption: nested package/alias contexts can differ.
                if specifier == "vue" {
                    if file.virtual_path.parent() != Some(project.virtual_root())
                        || !file
                            .virtual_path
                            .extension()
                            .and_then(|extension| extension.to_str())
                            .is_some_and(|extension| matches!(extension, "ts" | "js"))
                        || call
                        || token == "require"
                        || source.contains("resolution-mode")
                    {
                        return false;
                    }
                    continue;
                }
                // Other bare/absolute dependencies may augment globals only
                // in their importer's program, so decline new splitting.
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

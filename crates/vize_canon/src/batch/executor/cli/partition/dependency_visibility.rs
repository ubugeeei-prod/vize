//! Refuse new splitting when external module loads may carry ambient globals.

use std::path::Path;

use super::{VirtualFile, VirtualProject, normalize_join, resolve_virtual_import};
use vize_carton::FxHashMap;

pub(super) fn permits_sharing(
    project: &VirtualProject,
    files: &[&VirtualFile],
    index_by_virtual: &FxHashMap<&Path, usize>,
) -> bool {
    if !permits_compiler_options(project)
        || project.virtual_files_sorted().iter().any(|file| {
            let Some(source) = project.original_content_for_virtual(&file.virtual_path) else {
                return true;
            };
            !within_supported_source_domain(file, source)
        })
    {
        return false;
    }
    files.iter().all(|file| {
        let Some(source) = project.original_content_for_virtual(&file.virtual_path) else {
            return false;
        };
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
                // `vue` is one module for every file mirrored into this project.
                // A nested SFC (`src/App.vue`) imports it the same way the
                // root helper does. Files outside the virtual root can resolve
                // a different copy, so they keep the single-program plan.
                if specifier == "vue" {
                    if !file.virtual_path.starts_with(project.virtual_root())
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

fn permits_compiler_options(project: &VirtualProject) -> bool {
    // The executor materializes this authoritative flattened config before
    // partitioning, while retaining the existing MaterializeLock.
    let Ok(source) = std::fs::read_to_string(project.generated_tsconfig_path()) else {
        return false;
    };
    let Ok(config) = serde_json::from_str::<serde_json::Value>(&source) else {
        return false;
    };
    let Some(options) = config
        .get("compilerOptions")
        .and_then(serde_json::Value::as_object)
    else {
        return false;
    };
    // `jsx: "preserve"` leaves the syntax for a later tool and does not
    // inject a runtime. Vue projects set it by default; treating it as a
    // JSX runtime forced every shared type leaf back onto one checker.
    if [
        "jsxImportSource",
        "jsxFactory",
        "jsxFragmentFactory",
        "reactNamespace",
    ]
    .iter()
    .any(|option| options.contains_key(*option))
    {
        return false;
    }
    matches!(
        options.get("jsx").and_then(serde_json::Value::as_str),
        None | Some("preserve")
    )
}

fn within_supported_source_domain(file: &VirtualFile, source: &str) -> bool {
    if file
        .original_path
        .to_str()
        .is_some_and(|path| path.ends_with(".d.ts"))
    {
        // Existing fixed ambient registration includes these in every shard.
        return true;
    }
    match file
        .original_path
        .extension()
        .and_then(|extension| extension.to_str())
    {
        // Same distinction as a Vue script: `Row<T>` is a type argument.
        // `<div>` in a `.ts` file under `jsx: "preserve"` still declines.
        Some("ts" | "js") => return !script_body_opens_jsx_tag(source),
        Some("vue") => {}
        _ => return false,
    }
    let folded = source.to_ascii_lowercase();
    if folded.contains("src") || folded.contains("jsx") {
        // Script src becomes a synthetic module load; JSX may load an
        // implicit runtime. Neither is proved by the original import scan.
        return false;
    }
    for (at, _) in folded.match_indices("lang") {
        let Some(start) = source.get(..at).and_then(|prefix| prefix.rfind('<')) else {
            return false;
        };
        let prefix = source.get(start..at).unwrap_or_default();
        let languages: &[&str] = if prefix.starts_with("<script ") {
            &["ts", "js"]
        } else if prefix.starts_with("<template ") {
            &["html"]
        } else {
            return false;
        };
        let rest = source.get(at + "lang".len()..).unwrap_or_default();
        if prefix.contains('>')
            || source.get(at..at + "lang".len()) != Some("lang")
            || !languages.iter().any(|language| {
                rest.strip_prefix("=\"")
                    .and_then(|rest| rest.strip_prefix(language))
                    .is_some_and(|rest| rest.starts_with('"'))
                    || rest
                        .strip_prefix("='")
                        .and_then(|rest| rest.strip_prefix(language))
                        .is_some_and(|rest| rest.starts_with('\''))
            })
        {
            return false;
        }
    }
    for (at, _) in source.match_indices("<script") {
        let Some((_, rest)) = source.get(at..).unwrap_or_default().split_once('>') else {
            return false;
        };
        let Some((body, _)) = rest.split_once("</script>") else {
            return false;
        };
        // lang is already ts/js, and a JSX factory would have failed
        // `permits_compiler_options`. Type arguments (`defineProps<{`) are
        // the common case; they are not a runtime import.
        if script_body_opens_jsx_tag(body) {
            return false;
        }
    }
    true
}

/// `<div>` in a TS script can be recovered as JSX. `ref<T>` and `defineProps<{`
/// cannot: a type argument follows an identifier, a tag does not.
fn script_body_opens_jsx_tag(body: &str) -> bool {
    let mut rest = body;
    while let Some(at) = rest.find('<') {
        let previous = rest.get(..at).unwrap_or_default();
        let after_identifier = previous
            .chars()
            .rev()
            .find(|ch| !ch.is_whitespace())
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$' | '.'));
        let next = rest.get(at + 1..).unwrap_or_default().trim_start();
        let opens_tag = next.starts_with(|ch: char| ch.is_ascii_alphabetic() || ch == '/');
        if opens_tag && !after_identifier {
            return true;
        }
        rest = rest.get(at + 1..).unwrap_or_default();
    }
    false
}

fn skip_trivia(mut source: &str) -> &str {
    loop {
        // The pinned native scanner also treats BOM and zero-width space as
        // single-line trivia; Rust's Unicode White_Space excludes both.
        source = source.trim_start_matches(|ch: char| {
            ch.is_whitespace() || matches!(ch, '\u{feff}' | '\u{200b}')
        });
        if let Some(rest) = source.strip_prefix("/*") {
            let Some((_, rest)) = rest.split_once("*/") else {
                return "";
            };
            source = rest;
        } else if let Some(rest) = source.strip_prefix("//") {
            // Only these four native line endings terminate a line comment.
            // BOM, zero-width space and NEL remain inside the comment.
            let Some((_, rest)) = rest.split_once(['\n', '\r', '\u{2028}', '\u{2029}']) else {
                return "";
            };
            source = rest;
        } else {
            return source;
        }
    }
}

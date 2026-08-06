//! Go-to-definition through an imported name (#3893).
//!
//! The checker answers definition at an import site with the local alias
//! declaration — the import statement itself — so the response never left the
//! file, for any import shape. Editors expect the jump to unwrap the alias
//! and land on the exported declaration, the way tsserver does.
//!
//! This follows the import by hand: resolve the specifier (relative, bare
//! package, or tsconfig `paths` alias), then locate the named binding in the
//! target — following `export … from` barrels a bounded number of hops.

use std::fs;
use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};
use vize_carton::cstr;

use crate::ide::IdeContext;
use crate::ide::definition::{helpers, module_specifier, script};

/// Barrels can chain; three hops covers a package barrel re-exporting a
/// directory barrel re-exporting the module, without risking a cycle walk.
const MAX_REEXPORT_HOPS: usize = 3;

pub(super) fn definition(ctx: &IdeContext<'_>) -> Option<GotoDefinitionResponse> {
    let word = helpers::get_word_at_offset(&ctx.content, ctx.offset)?;
    let specifier = importing_specifier(&ctx.content, ctx.offset, &word)?;
    let target = resolve_import_specifier(ctx.uri, &specifier)?;
    locate_export(&target, &word, MAX_REEXPORT_HOPS).map(GotoDefinitionResponse::Scalar)
}

/// The module specifier of the import statement that both contains `offset`
/// and binds `word`. `None` when the cursor is not on an imported name.
fn importing_specifier(content: &str, offset: usize, word: &str) -> Option<String> {
    let mut search_start = 0;
    while let Some(position) = content[search_start..].find("import") {
        let statement_start = search_start + position;
        search_start = statement_start + "import".len();
        let rest = &content[statement_start..];
        let from_quote = find_specifier_span(rest)?;
        let statement_end = statement_start + from_quote.1;
        if offset < statement_start || offset > statement_end {
            continue;
        }
        let clause = &rest[..from_quote.0];
        if !binds_name(clause, word) {
            return None;
        }
        return Some(rest[from_quote.0 + 1..from_quote.1].to_owned());
    }
    None
}

/// `(opening_quote_index, closing_quote_index)` of the statement's specifier,
/// relative to the statement start.
fn find_specifier_span(rest: &str) -> Option<(usize, usize)> {
    let line_end = rest.find('\n').map_or(rest.len(), |index| {
        // Multi-line named-import lists keep scanning to the closing quote.
        if rest[..index].contains('{') && !rest[..index].contains('}') {
            rest.len()
        } else {
            index
        }
    });
    let segment = &rest[..line_end];
    let quote_start = segment
        .find('"')
        .into_iter()
        .chain(segment.find('\''))
        .min()?;
    let quote = segment.as_bytes()[quote_start] as char;
    let quote_end = segment[quote_start + 1..].find(quote)? + quote_start + 1;
    Some((quote_start, quote_end))
}

/// Whether the import clause binds `word` — as a named import (respecting
/// `as` renames), a default import, or a namespace import.
fn binds_name(clause: &str, word: &str) -> bool {
    if let (Some(open), Some(close)) = (clause.find('{'), clause.find('}')) {
        for part in clause[open + 1..close].split(',') {
            let bound = part
                .rsplit(" as ")
                .next()
                .unwrap_or(part)
                .trim()
                .trim_start_matches("type ")
                .trim();
            if bound == word {
                return true;
            }
        }
    }
    let head = clause.split('{').next().unwrap_or(clause);
    head.split(',').any(|part| {
        let part = part
            .trim()
            .trim_start_matches("import")
            .trim()
            .trim_end_matches("from")
            .trim();
        part == word || part.strip_prefix("* as ").map(str::trim) == Some(word)
    })
}

/// Resolve relative and bare specifiers through the shared resolver, and
/// tsconfig `paths` aliases through the nearest tsconfig — the shapes the
/// audit measured (`../composables/useCounter`, `#ui`, `@/lib/format`).
fn resolve_import_specifier(uri: &Url, specifier: &str) -> Option<PathBuf> {
    if let Some(path) = module_specifier::resolve_specifier(uri, specifier) {
        return Some(path);
    }
    let file = uri.to_file_path().ok()?;
    let tsconfig_dir = file
        .ancestors()
        .skip(1)
        .find(|dir| dir.join("tsconfig.json").is_file())?;
    let manifest = fs::read_to_string(tsconfig_dir.join("tsconfig.json")).ok()?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest)
        .ok()
        .or_else(|| serde_json::from_str(&strip_jsonc_comments(&manifest)).ok())?;
    let paths = manifest.get("compilerOptions")?.get("paths")?.as_object()?;
    let mut best: Option<(usize, PathBuf)> = None;
    for (pattern, targets) in paths {
        let substituted = if let Some(prefix) = pattern.strip_suffix('*') {
            specifier.strip_prefix(prefix).and_then(|rest| {
                targets.as_array()?.iter().find_map(|target| {
                    let target = target.as_str()?.strip_suffix('*')?;
                    Some(cstr!("{target}{rest}").to_string())
                })
            })
        } else if specifier == pattern {
            targets
                .as_array()
                .and_then(|targets| targets.first())
                .and_then(|target| target.as_str())
                .map(str::to_owned)
        } else {
            None
        };
        let Some(substituted) = substituted else {
            continue;
        };
        let base = tsconfig_dir.join(substituted);
        if let Some(resolved) = probe(&base)
            && best.as_ref().is_none_or(|(len, _)| pattern.len() > *len)
        {
            best = Some((pattern.len(), resolved));
        }
    }
    best.map(|(_, path)| path)
}

/// Line and block comments stripped, so real-world tsconfigs parse. Naive
/// about comment markers inside strings, which tsconfig paths do not contain
/// in practice; a failed parse simply skips alias resolution.
fn strip_jsonc_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '/' && chars.peek() == Some(&'/') {
            for skipped in chars.by_ref() {
                if skipped == '\n' {
                    out.push('\n');
                    break;
                }
            }
        } else if ch == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut previous = ' ';
            for skipped in chars.by_ref() {
                if previous == '*' && skipped == '/' {
                    break;
                }
                previous = skipped;
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn probe(base: &Path) -> Option<PathBuf> {
    if base.extension().is_some() && base.is_file() {
        return Some(base.to_path_buf());
    }
    for extension in ["ts", "tsx", "d.ts", "vue"] {
        let candidate = PathBuf::from(cstr!("{}.{extension}", base.display()).as_str());
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    ["index.ts", "index.tsx"]
        .iter()
        .map(|index| base.join(index))
        .find(|candidate| candidate.is_file())
}

/// The declaration of `word` inside `target`, following re-export barrels.
fn locate_export(target: &Path, word: &str, hops: usize) -> Option<Location> {
    if target
        .extension()
        .is_some_and(|extension| extension == "vue")
    {
        // A `.vue` module's meaningful position is the file itself, matching
        // how component-tag definition resolves today.
        return Some(Location {
            uri: Url::from_file_path(target).ok()?,
            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
        });
    }
    let content = fs::read_to_string(target).ok()?;

    // A barrel both names the word and points elsewhere; the re-export hop
    // comes first so the jump lands on the real declaration, not the alias.
    if hops > 0
        && let Some(specifier) = reexport_specifier(&content, word)
        && let Some(uri) = Url::from_file_path(target).ok()
        && let Some(next) = resolve_import_specifier(&uri, &specifier)
        && let Some(location) = locate_export(&next, word, hops - 1)
    {
        return Some(location);
    }

    if let Some(binding) = script::find_binding_location_raw(&content, word) {
        let (line, character) = helpers::offset_to_position(&content, binding.offset);
        let uri = Url::from_file_path(target).ok()?;
        let position = Position::new(line, character);
        let end = Position::new(line, character + word.len() as u32);
        return Some(Location {
            uri,
            range: Range::new(position, end),
        });
    }

    None
}

/// The source specifier of an `export … from` clause covering `word`.
fn reexport_specifier(content: &str, word: &str) -> Option<String> {
    let mut search_start = 0;
    while let Some(position) = content[search_start..].find("export") {
        let statement_start = search_start + position;
        search_start = statement_start + "export".len();
        let rest = &content[statement_start..];
        let Some((quote_start, quote_end)) = find_specifier_span(rest) else {
            continue;
        };
        let clause = &rest[..quote_start];
        if !clause.contains("from") {
            continue;
        }
        let covers = if let (Some(open), Some(close)) = (clause.find('{'), clause.find('}')) {
            clause[open + 1..close].split(',').any(|part| {
                let part = part.trim();
                let exported = part.rsplit(" as ").next().unwrap_or(part).trim();
                exported == word
            })
        } else {
            clause.contains('*')
        };
        if covers {
            return Some(rest[quote_start + 1..quote_end].to_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_importing_statement_is_matched_by_offset_and_binding() {
        let content = "import { a } from \"./a\";\nimport { useCounter, type User } from \"../composables/useCounter\";\n";
        let offset = content.find("useCounter").unwrap() + 2;
        assert_eq!(
            super::importing_specifier(content, offset, "useCounter").as_deref(),
            Some("../composables/useCounter"),
        );
        // Same statement, different word: `type User` binds too.
        let offset = content.find("User").unwrap();
        assert_eq!(
            super::importing_specifier(content, offset, "User").as_deref(),
            Some("../composables/useCounter"),
        );
        // Outside any import statement: no match.
        assert_eq!(
            super::importing_specifier(content, content.len() - 1, "useCounter"),
            None
        );
    }

    #[test]
    fn renamed_default_and_namespace_imports_bind() {
        assert!(super::binds_name("import { long as short } from", "short"));
        assert!(!super::binds_name("import { long as short } from", "long"));
        assert!(super::binds_name("import Default, { x } from", "Default"));
        assert!(super::binds_name("import * as ns from", "ns"));
    }

    #[test]
    fn reexports_cover_named_renames_and_stars() {
        let barrel = "export { default as UiButton } from \"./UiButton.vue\";\nexport * from \"./tokens\";\n";
        assert_eq!(
            super::reexport_specifier(barrel, "UiButton").as_deref(),
            Some("./UiButton.vue"),
        );
        assert_eq!(
            super::reexport_specifier(barrel, "anything").as_deref(),
            Some("./tokens")
        );
    }
}

//! Deduplicating the per-component runtime-helper preambles of one module.

use vize_s0::{FxHashSet, String};

/// Merge a sequence of per-component preambles into one deduplicated preamble.
///
/// Each VDOM preamble is a line-oriented block — typically a single
/// `import { name as _alias, … } from "vue"` statement (default JSX options emit
/// no hoists, but any extra lines are preserved verbatim). Concatenating several
/// components' preambles as-is would redeclare the same `_alias` bindings, which
/// is an ESM error, so this collapses every `import … from "<src>"` line into a
/// single import per source carrying the union of its specifiers in first-seen
/// order. Non-import lines (e.g. static hoists) are kept verbatim, deduplicated,
/// and appended after the merged imports.
pub(super) fn merge_preambles<'a>(preambles: impl Iterator<Item = &'a str>) -> String {
    // Imports grouped by source module, each preserving first-seen specifier
    // order; sources themselves preserve first-seen order via `import_sources`.
    let mut import_sources: Vec<&str> = Vec::new();
    let mut import_specifiers: Vec<Vec<&str>> = Vec::new();
    let mut seen_specifiers: FxHashSet<(&str, &str)> = FxHashSet::default();
    let mut extra_lines: Vec<&str> = Vec::new();
    let mut seen_extra: FxHashSet<&str> = FxHashSet::default();

    for preamble in preambles {
        for line in preamble.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            match parse_named_import(trimmed) {
                Some((specifiers, source)) => {
                    let group = match import_sources.iter().position(|s| *s == source) {
                        Some(index) => index,
                        None => {
                            import_sources.push(source);
                            import_specifiers.push(Vec::new());
                            import_sources.len() - 1
                        }
                    };
                    for specifier in specifiers.split(',') {
                        let specifier = specifier.trim();
                        if specifier.is_empty() {
                            continue;
                        }
                        if seen_specifiers.insert((source, specifier)) {
                            import_specifiers[group].push(specifier);
                        }
                    }
                }
                None => {
                    if seen_extra.insert(trimmed) {
                        extra_lines.push(trimmed);
                    }
                }
            }
        }
    }

    let mut merged = String::default();
    for (source, specifiers) in import_sources.iter().zip(import_specifiers.iter()) {
        merged.push_str("import { ");
        for (i, specifier) in specifiers.iter().enumerate() {
            if i > 0 {
                merged.push_str(", ");
            }
            merged.push_str(specifier);
        }
        merged.push_str(" } from \"");
        merged.push_str(source);
        merged.push_str("\"\n");
    }
    for line in extra_lines {
        merged.push_str(line);
        merged.push('\n');
    }
    merged
}

/// Parse a `import { a as _a, b as _b } from "src"` line into its
/// specifier list (the text between the braces) and source module. Returns
/// `None` for any line that is not a brace-style named import (so it is kept
/// verbatim by [`merge_preambles`]).
fn parse_named_import(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("import")?;
    let open = rest.find('{')?;
    let close = rest.find('}')?;
    if close < open {
        return None;
    }
    let specifiers = &rest[open + 1..close];

    let after = &rest[close + 1..];
    let from = after.find("from")?;
    let quoted = after[from + "from".len()..].trim();
    let bytes = quoted.as_bytes();
    let quote = *bytes.first()?;
    if quote != b'"' && quote != b'\'' {
        return None;
    }
    let inner = &quoted[1..];
    let end = inner.find(quote as char)?;
    Some((specifiers, &inner[..end]))
}

#[cfg(test)]
mod tests {
    use super::{merge_preambles, parse_named_import};

    #[test]
    fn parses_named_import_specifiers_and_source() {
        assert_eq!(
            parse_named_import("import { a as _a, b as _b } from \"vue\""),
            Some((" a as _a, b as _b ", "vue"))
        );
        // Single-quoted source is accepted too.
        assert_eq!(
            parse_named_import("import { x } from 'vue'"),
            Some((" x ", "vue"))
        );
        // Non-imports and namespace/default imports are not brace-named imports.
        assert_eq!(parse_named_import("const _hoisted = 1"), None);
        assert_eq!(parse_named_import("import Foo from \"bar\""), None);
    }

    #[test]
    fn merge_preambles_dedups_overlapping_vue_imports() {
        // Two components importing overlapping helpers from "vue" must collapse to
        // one import with each binding declared exactly once (concatenating the
        // raw lines would redeclare `_createElementBlock`, an ESM error).
        let merged = merge_preambles(
            [
                "import { createElementBlock as _createElementBlock } from \"vue\"\n",
                "import { createElementBlock as _createElementBlock, toDisplayString as _toDisplayString } from \"vue\"\n",
            ]
            .into_iter(),
        );
        assert_eq!(
            merged,
            "import { createElementBlock as _createElementBlock, toDisplayString as _toDisplayString } from \"vue\"\n"
        );
    }

    #[test]
    fn merge_preambles_keeps_distinct_sources_and_hoists() {
        // Distinct sources each get their own import (first-seen order), and a
        // non-import hoist line is preserved verbatim after the imports.
        let merged = merge_preambles(
            [
                "import { a as _a } from \"vue\"\nconst _hoisted = 1\n",
                "import { b as _b } from \"other\"\n",
            ]
            .into_iter(),
        );
        assert_eq!(
            merged,
            "import { a as _a } from \"vue\"\nimport { b as _b } from \"other\"\nconst _hoisted = 1\n"
        );
    }
}

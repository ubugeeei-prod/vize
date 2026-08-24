//! The `v-for` value split: Vue's text grammar, applied **before** any
//! parse — the P2-5b decision, consumed here rather than re-derived.
//!
//! # Why the split is textual (the `a in b in c` disagreement)
//!
//! JS `in` associates left; Vue's v-for grammar splits at the **first
//! viable** `in`/`of`. On `a in b in c` they genuinely disagree — Vue
//! reads alias `a`, source `b in c`; a retained AST of the whole value
//! would read `(a in b) in c`. That is the P1-6-recorded reason a v-for
//! value's retained AST must never be consumed naively, and why
//! [`OpaqueReason::ForValue`] exists: the whole value is **not a JS
//! expression**. The lowering therefore splits the authored text with
//! the same grammar the shipped splitter uses
//! (`crates/vize_atelier_core/src/transforms/v_for.rs`,
//! `find_for_separator` / `split_top_level_aliases`, strict mode — no
//! `template_syntax_quirks`), then admits each **sub-slice** through the
//! shared rule ([`super::expr`]); a value that cannot split at all rides
//! whole as `Opaque(ForValue)` with pessimal semantics.

use vize_s0::SmallVec;

/// Vue gives `v-for` exactly three binding positions: value, key, and
/// index. Keep that overwhelmingly common shape inline while retaining
/// every additional authored alias so malformed or future syntax is not
/// silently truncated.
const FOR_ALIAS_INLINE_CAPACITY: usize = 3;

type ForAliases<'a> = SmallVec<[&'a str; FOR_ALIAS_INLINE_CAPACITY]>;

/// The split of a v-for value: byte ranges of the alias part and the
/// source part inside the value text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ForSplit {
    /// End of the alias text (before the separator's whitespace).
    pub alias_end: usize,
    /// Start of the source text (after the separator's whitespace).
    pub source_start: usize,
}

/// Find the first viable ` in ` / ` of ` separator: the keyword with
/// whitespace on both sides, exactly the shipped grammar.
pub(crate) fn split_for(content: &str) -> Option<ForSplit> {
    for (keyword_start, first) in content.char_indices() {
        let keyword = match first {
            'i' if content[keyword_start..].starts_with("in") => "in",
            'o' if content[keyword_start..].starts_with("of") => "of",
            _ => continue,
        };
        let has_space_before = content[..keyword_start]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace);
        let after_keyword = keyword_start + keyword.len();
        let has_space_after = content[after_keyword..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace);
        if !(has_space_before && has_space_after) {
            continue;
        }

        let alias_end = content[..keyword_start].trim_end().len();
        let source = content[after_keyword..].trim_start();
        if source.is_empty() {
            return None;
        }
        let source_start = content.len() - source.len();
        return Some(ForSplit {
            alias_end,
            source_start,
        });
    }
    None
}

/// Split the alias part into its top-level positions (value, key, index).
/// Strict grammar: parentheses strip only as a matched pair; a lone
/// paren is malformed (`None`), matching the shipped splitter's default
/// (the `template_syntax_quirks` compatibility path is deliberately not
/// offered here).
pub(crate) fn split_aliases(alias: &str) -> Option<ForAliases<'_>> {
    let trimmed = alias.trim();
    if trimmed.is_empty() {
        return Some(ForAliases::new());
    }
    let starts = trimmed.starts_with('(');
    let ends = trimmed.ends_with(')');
    let inner = if starts && ends {
        if trimmed.len() < 2 {
            return None;
        }
        &trimmed[1..trimmed.len() - 1]
    } else if starts || ends {
        return None;
    } else {
        trimmed
    };
    Some(split_top_level(inner.trim()))
}

/// Split on top-level commas, string- and bracket-aware.
fn split_top_level(input: &str) -> ForAliases<'_> {
    let bytes = input.as_bytes();
    let mut aliases = ForAliases::new();
    let mut start = 0usize;
    let mut paren = 0u32;
    let mut brace = 0u32;
    let mut bracket = 0u32;
    let mut in_string: Option<u8> = None;
    let mut escaped = false;
    for (idx, &byte) in bytes.iter().enumerate() {
        if let Some(quote) = in_string {
            if escaped {
                escaped = false;
                continue;
            }
            if byte == b'\\' {
                escaped = true;
                continue;
            }
            if byte == quote {
                in_string = None;
            }
            continue;
        }
        match byte {
            b'\'' | b'"' | b'`' => in_string = Some(byte),
            b'(' => paren += 1,
            b')' => paren = paren.saturating_sub(1),
            b'{' => brace += 1,
            b'}' => brace = brace.saturating_sub(1),
            b'[' => bracket += 1,
            b']' => bracket = bracket.saturating_sub(1),
            b',' if paren == 0 && brace == 0 && bracket == 0 => {
                aliases.push(input[start..idx].trim());
                start = idx + 1;
            }
            _ => {}
        }
    }
    aliases.push(input[start..].trim());
    aliases
}

#[cfg(test)]
mod tests {
    use super::{split_aliases, split_for};

    fn split(content: &str) -> (&str, &str) {
        let s = split_for(content).expect("the value splits");
        (&content[..s.alias_end], &content[s.source_start..])
    }

    #[test]
    fn splits_at_the_first_viable_keyword() {
        assert_eq!(split("item in items"), ("item", "items"));
        assert_eq!(split("item of items"), ("item", "items"));
        // The recorded disagreement: Vue reads alias `a`, source
        // `b in c`; JS `in` associates left and would read `(a in b)`.
        assert_eq!(split("a in b in c"), ("a", "b in c"));
    }

    #[test]
    fn keywords_need_whitespace_on_both_sides() {
        assert_eq!(split_for("items"), None);
        assert_eq!(split_for("in items"), None);
        assert_eq!(split_for("kind of"), None);
        assert_eq!(split_for("item in   "), None);
        assert_eq!(split("index in indexes"), ("index", "indexes"));
    }

    #[test]
    fn separator_scan_preserves_unicode_boundaries_and_whitespace() {
        assert_eq!(split("値\u{2003}in\u{2003}一覧"), ("値", "一覧"));
    }

    #[test]
    fn aliases_split_at_top_level_commas_only() {
        assert_eq!(
            split_aliases("(item, key, index)").as_deref(),
            Some(["item", "key", "index"].as_slice())
        );
        assert_eq!(
            split_aliases("item, index").as_deref(),
            Some(["item", "index"].as_slice())
        );
        assert_eq!(
            split_aliases("({ a, b }, i)").as_deref(),
            Some(["{ a, b }", "i"].as_slice())
        );
        assert_eq!(
            split_aliases("[a, b]").as_deref(),
            Some(["[a, b]"].as_slice())
        );
        assert_eq!(split_aliases("").as_deref(), Some([].as_slice()));
    }

    #[test]
    fn contract_aliases_stay_inline() {
        let aliases = split_aliases("(value, key, index)").expect("aliases split");

        assert_eq!(aliases.as_slice(), ["value", "key", "index"]);
        assert!(!aliases.spilled());
    }

    #[test]
    fn additional_authored_aliases_spill_without_truncation() {
        let aliases = split_aliases("(value, key, index, extra)").expect("aliases split");

        assert_eq!(aliases.as_slice(), ["value", "key", "index", "extra"]);
        assert!(aliases.spilled());
    }

    #[test]
    fn a_lone_paren_is_malformed_in_strict_grammar() {
        assert_eq!(split_aliases("(item"), None);
        assert_eq!(split_aliases("item)"), None);
    }
}

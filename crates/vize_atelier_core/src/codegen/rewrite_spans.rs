//! Identifier spans for context-rewritten template expressions (P3-9).
//!
//! Template expressions reach the emitters already rewritten: the transform
//! and the codegen scope pass splice context accessors around identifier
//! references (`msg` → `_ctx.msg`, `$setup.msg`, `_unref(msg)`,
//! `msg.value`) and keep every other byte verbatim. The rewrite edits are
//! pure insertions from a closed alphabet, so the emitted text is the authored
//! text with alphabet tokens inserted.
//!
//! [`rewritten_identifier_spans`] recovers those insertion points *exactly*:
//! it only answers when the emitted text decomposes into the authored bytes in
//! order plus alphabet tokens, and it prefers keeping an authored byte over
//! treating it as an insertion, so a literal `_ctx.` the author wrote is never
//! mistaken for a rewrite. Each returned pair anchors the first byte of a
//! rewritten identifier (its accessor prefix) to the identifier's first
//! authored byte, which is the span shape TS-31 requires to be byte-exact.
//! Anything else — TypeScript stripping, comment rewriting, props-alias
//! renames — makes the decomposition fail and yields no identifier spans; the
//! expression keeps its whole-expression anchor.

/// Accessor prefixes the rewrite inserts before an identifier reference.
const PREFIXES: [&str; 7] = [
    "_ctx.",
    "$setup.",
    "$props.",
    "$data.",
    "$options.",
    "__props.",
    "_unref(",
];

/// Tokens the rewrite inserts after an identifier reference.
const SUFFIXES: [&str; 2] = [")", ".value"];

/// Largest alignment table (authored bytes × inserted bytes) worth building.
/// Beyond it the expression keeps only its whole-expression anchor.
const MAX_CELLS: usize = 1 << 22;

/// One rewritten identifier: its offset in the emitted text, its offset in the
/// authored text, its authored name, and the length of the accessor prefix
/// the rewrite inserted before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RewrittenIdentifier<'a> {
    pub emitted: usize,
    pub authored: usize,
    pub name: &'a str,
    pub prefix_len: usize,
}

impl RewrittenIdentifier<'_> {
    /// Length of the rewritten reference in the emitted text: the accessor
    /// prefix plus the name.
    pub fn emitted_len(&self) -> usize {
        self.prefix_len + self.name.len()
    }
}

/// Decompose `emitted` into `authored` plus rewrite insertions and return the
/// rewritten identifiers, or `None` when no exact decomposition exists.
pub fn rewritten_identifier_spans<'a>(
    authored: &'a str,
    emitted: &str,
) -> Option<Vec<RewrittenIdentifier<'a>>> {
    let (original, output) = (authored.as_bytes(), emitted.as_bytes());
    let inserted = output.len().checked_sub(original.len())?;
    let width = inserted + 1;
    if (original.len() + 1).saturating_mul(width) > MAX_CELLS {
        return None;
    }
    let tokens = || PREFIXES.iter().chain(SUFFIXES.iter()).map(|t| t.as_bytes());
    // feasible[i * width + k]: authored[i..] aligns with emitted[i + k..].
    let mut feasible = vec![false; (original.len() + 1) * width];
    for i in (0..=original.len()).rev() {
        for k in (0..=inserted).rev() {
            let at = &output[i + k..];
            let keeps =
                i < original.len() && original[i] == output[i + k] && feasible[(i + 1) * width + k];
            let ends = i == original.len() && k == inserted;
            let inserts = || {
                tokens().any(|t| {
                    k + t.len() <= inserted
                        && at.starts_with(t)
                        && feasible[i * width + k + t.len()]
                })
            };
            feasible[i * width + k] = keeps || ends || inserts();
        }
    }
    if !feasible[0] {
        return None;
    }

    let mut spans = Vec::new();
    let (mut i, mut k) = (0usize, 0usize);
    while i < original.len() || k < inserted {
        if i < original.len() && original[i] == output[i + k] && feasible[(i + 1) * width + k] {
            i += 1;
            continue;
        }
        let at = &output[i + k..];
        let token = tokens()
            .find(|t| {
                k + t.len() <= inserted && at.starts_with(t) && feasible[i * width + k + t.len()]
            })
            .expect("a feasible state keeps a byte or inserts a token");
        let is_prefix = PREFIXES.iter().any(|p| p.as_bytes() == token);
        let name = identifier_at(authored, i);
        if is_prefix && !name.is_empty() {
            spans.push(RewrittenIdentifier {
                emitted: i + k,
                authored: i,
                name,
                prefix_len: token.len(),
            });
        }
        k += token.len();
    }
    Some(spans)
}

/// The identifier starting at `offset` in `text`, or `""`.
fn identifier_at(text: &str, offset: usize) -> &str {
    let rest = &text[offset..];
    let starts = rest
        .chars()
        .next()
        .is_some_and(|ch| ch.is_alphabetic() || ch == '_' || ch == '$');
    if !starts {
        return "";
    }
    let end = rest
        .find(|ch: char| !(ch.is_alphanumeric() || ch == '_' || ch == '$'))
        .unwrap_or(rest.len());
    &rest[..end]
}

#[cfg(test)]
mod tests {
    use super::{RewrittenIdentifier, rewritten_identifier_spans};

    fn spans<'a>(authored: &'a str, emitted: &str) -> Option<Vec<(usize, usize, &'a str)>> {
        rewritten_identifier_spans(authored, emitted).map(|spans| {
            spans
                .into_iter()
                .map(
                    |RewrittenIdentifier {
                         emitted,
                         authored,
                         name,
                         ..
                     }| (emitted, authored, name),
                )
                .collect()
        })
    }

    #[test]
    fn every_context_prefix_is_anchored_to_its_identifier() {
        assert_eq!(
            spans("count + offset", "_ctx.count + _ctx.offset"),
            Some(vec![(0, 0, "count"), (13, 8, "offset")])
        );
        assert_eq!(
            spans("a(b)", "$setup.a(_unref(b))"),
            Some(vec![(0, 0, "a"), (9, 2, "b")])
        );
        assert_eq!(spans("n = n + 1", "n.value = n.value + 1"), Some(vec![]));
    }

    #[test]
    fn authored_bytes_win_over_insertions() {
        // `_ctx` authored verbatim is kept; only the inserted prefix anchors.
        assert_eq!(spans("_ctx.a", "_ctx.a"), Some(vec![]));
        assert_eq!(spans("_x", "_ctx._x"), Some(vec![(0, 0, "_x")]));
        assert_eq!(spans("c", "_ctx.c"), Some(vec![(0, 0, "c")]));
    }

    #[test]
    fn non_insertion_rewrites_yield_no_spans() {
        assert_eq!(spans("a as B", "_ctx.a"), None);
        assert_eq!(spans("foo", "__props.bar"), None);
        assert_eq!(spans("ab", "a"), None);
    }
}

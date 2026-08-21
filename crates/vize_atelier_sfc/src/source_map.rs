//! Source-map emission for the assembled SFC module (#3399).
//!
//! # Why the map is recovered rather than recorded
//!
//! `compile_sfc` does not edit the `.vue` source into its
//! output the way a MagicString-based compiler does. It *concatenates*
//! independently produced chunks: hoisted template imports, the rewritten
//! `<script>`/`<script setup>` body, a synthesized render function, an
//! `export default`. Several of those chunks come from passes that only ever see
//! strings (the script sections are split into lines, filtered and re-joined),
//! and a `lang="ts"` SFC is re-printed wholesale by oxc before it crosses the
//! napi boundary. There is no edit script to turn into mappings, and no offset
//! that survives the pipeline end to end.
//!
//! What does survive is the *content* of what the author wrote. This module
//! recovers the mapping from that, by indexing the authored
//! `<script>`/`<script setup>` lines and matching emitted lines back to them.
//!
//! # The anchor rules
//!
//! A generated line is mapped only when it matches **exactly one** authored
//! script line under the first of these keys that resolves. Every key is
//! uniqueness-gated on its own: a key shared by two authored lines anchors
//! nothing, because there would be no single origin to name.
//!
//! 1. **Exact text** — the trimmed line is byte-identical. This is the common
//!    case for plain JavaScript SFCs, whose script statements are copied out
//!    verbatim and only re-indented.
//! 2. **Printer-normalised text** — whitespace runs collapsed, `'` folded to
//!    `"`, a trailing `;` dropped. This survives oxc re-printing a TypeScript
//!    module, which changes exactly those things and nothing else about a
//!    statement it did not otherwise transform.
//! 3. **Declared binding** — the `(kind, name)` a declaration introduces, e.g.
//!    `const msg`. This is what survives when the printer *did* change the line:
//!    `const msg: string = 'x'` is emitted as `const msg = "x"`, and
//!    `const props = defineProps(...)` as `const props = __props`. Mapping the
//!    generated declaration of a binding to the authored declaration of that
//!    same binding is right by construction, not by coincidence.
//!
//! Anything that matches no key — synthesized render code, `_sfc_main`
//! plumbing — is left unmapped rather than guessed at. Unmapped lines degrade to
//! the nearest preceding mapping, which is still inside the authored `.vue`
//! file: the behaviour the issue asks for, instead of the virtual `.vue.ts`
//! module.
//!
//! Fidelity is line-level: a mapping anchors the first non-whitespace column of
//! the generated line to the first non-whitespace column of the authored line.
//! Widening it to expression level requires the emitter to record offsets and is
//! tracked separately; see the crate-level codegen docs and #1533.

use vize_atelier_core::codegen::source_map::SourceMapBuilder;
use vize_carton::{FxHashMap, String};

use crate::types::SfcDescriptor;

#[cfg(test)]
mod anchor_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

/// Shortest trimmed line worth anchoring.
///
/// Below this every candidate is structural punctuation (`}`, `})`, `};`) that
/// the emitter also produces on its own, so an anchor would be noise even when
/// the text happens to be unique.
const MIN_ANCHOR_LEN: usize = 3;

/// Keywords that introduce a binding, for anchor rule 3.
const DECLARATION_KEYWORDS: [&str; 5] = ["const", "let", "var", "function", "class"];

/// Modifiers that may precede a declaration without changing what it declares.
const DECLARATION_MODIFIERS: [&str; 3] = ["export", "default", "async"];

/// Where a key was found in the authored script blocks.
#[derive(Debug, Clone, Copy)]
enum Anchor {
    /// The key occurs exactly once; the payload is the byte offset of the first
    /// non-whitespace character of that line in the SFC source.
    Unique(u32),
    /// The key occurs more than once, so no single origin can be chosen.
    Ambiguous,
}

/// The authored script lines, indexed under each anchor key.
#[derive(Default)]
struct AnchorIndex<'a> {
    exact: FxHashMap<&'a str, Anchor>,
    normalized: FxHashMap<String, Anchor>,
    declared: FxHashMap<String, Anchor>,
}

impl<'a> AnchorIndex<'a> {
    fn is_empty(&self) -> bool {
        self.exact.is_empty()
    }

    fn insert(&mut self, trimmed: &'a str, offset: u32) {
        record(&mut self.exact, trimmed, offset);
        record(&mut self.normalized, normalized_key(trimmed), offset);
        if let Some(key) = declaration_index_key(trimmed) {
            record(&mut self.declared, key, offset);
        }
    }

    /// The authored offset `trimmed` unambiguously came from, by rule order.
    fn lookup(&self, trimmed: &str) -> Option<u32> {
        unique(self.exact.get(trimmed))
            .or_else(|| unique(self.normalized.get(&normalized_key(trimmed))))
            .or_else(|| {
                declaration_index_key(trimmed).and_then(|key| unique(self.declared.get(&key)))
            })
    }
}

fn record<K: std::hash::Hash + Eq>(index: &mut FxHashMap<K, Anchor>, key: K, offset: u32) {
    index
        .entry(key)
        .and_modify(|anchor| *anchor = Anchor::Ambiguous)
        .or_insert(Anchor::Unique(offset));
}

fn unique(anchor: Option<&Anchor>) -> Option<u32> {
    match anchor {
        Some(Anchor::Unique(offset)) => Some(*offset),
        _ => None,
    }
}

/// Whether a trimmed line carries enough signal to be an anchor.
///
/// Requires real identifier content, which rules out brace/paren-only lines
/// without needing a list of them.
fn is_anchorable(trimmed: &str) -> bool {
    trimmed.len() >= MIN_ANCHOR_LEN
        && trimmed
            .chars()
            .any(|ch| ch.is_alphanumeric() || ch == '_' || ch == '$')
}

/// Reduce a line to what a JavaScript printer cannot change about it.
fn normalized_key(trimmed: &str) -> String {
    let mut out = String::with_capacity(trimmed.len());
    let mut pending_space = false;
    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            pending_space = true;
            continue;
        }
        if pending_space && !out.is_empty() {
            out.push(' ');
        }
        pending_space = false;
        out.push(if ch == '\'' { '"' } else { ch });
    }
    // The space before a dropped `;` has to go with it, or `const a = 'x' ;`
    // and `const a = "x";` would normalise differently.
    while out.ends_with(';') || out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Strip a leading keyword and the separator after it.
fn strip_keyword<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = text.strip_prefix(keyword)?;
    rest.starts_with(|ch: char| ch.is_whitespace() || ch == '*')
        .then(|| rest.trim_start())
}

/// The declaration a line introduces, rendered as the index key `"kind name"`.
fn declaration_index_key(trimmed: &str) -> Option<String> {
    let (kind, name) = declaration_key(trimmed)?;
    let mut key = String::with_capacity(kind.len() + name.len() + 1);
    key.push_str(kind);
    key.push(' ');
    key.push_str(name);
    Some(key)
}

/// The `(kind, name)` a declaration line introduces, or `None` when the line is
/// not a declaration.
fn declaration_key(trimmed: &str) -> Option<(&str, &str)> {
    let mut rest = trimmed;
    while let Some(stripped) = DECLARATION_MODIFIERS
        .iter()
        .find_map(|modifier| strip_keyword(rest, modifier))
    {
        rest = stripped;
    }

    let (kind, after) = DECLARATION_KEYWORDS
        .into_iter()
        .find_map(|kind| strip_keyword(rest, kind).map(|after| (kind, after)))?;
    // `function* gen()` declares `gen`, same as `function gen()`.
    let after = after.strip_prefix('*').map_or(after, str::trim_start);
    let end = after
        .find(|ch: char| !(ch.is_alphanumeric() || ch == '_' || ch == '$'))
        .unwrap_or(after.len());
    (end > 0).then(|| (kind, &after[..end]))
}

/// Iterate `(byte offset of the line, line without its terminator)`.
///
/// `split_inclusive` keeps the terminator so the running offset stays exact for
/// both `\n` and `\r\n` inputs; the terminator is then trimmed off the yielded
/// text so callers compare line bodies.
fn lines_with_offsets(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0usize;
    text.split_inclusive('\n').map(move |raw| {
        let start = offset;
        offset += raw.len();
        let body = raw
            .strip_suffix('\n')
            .map_or(raw, |line| line.strip_suffix('\r').unwrap_or(line));
        (start, body)
    })
}

/// Split a line into `(leading whitespace length, trimmed body)`.
fn split_indent(line: &str) -> (usize, &str) {
    let without_indent = line.trim_start();
    (line.len() - without_indent.len(), without_indent.trim_end())
}

/// Byte ranges of the authored `<script>` and `<script setup>` block contents.
fn script_ranges(descriptor: &SfcDescriptor<'_>) -> Vec<(usize, usize)> {
    [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
        .into_iter()
        .flatten()
        .map(|block| (block.loc.start, block.loc.end))
        .filter(|(start, end)| end > start)
        .collect()
}

/// Index the authored script lines under every anchor key.
///
/// A line participates when it overlaps a script block's content range, so the
/// `<script>`/`</script>` tag lines and everything in `<template>`/`<style>` are
/// excluded: only text the emitter could have derived from is a candidate.
fn index_script_lines<'a>(source: &'a str, ranges: &[(usize, usize)]) -> AnchorIndex<'a> {
    let mut index = AnchorIndex::default();

    for (start, line) in lines_with_offsets(source) {
        let end = start + line.len();
        if !ranges.iter().any(|&(lo, hi)| start < hi && end > lo) {
            continue;
        }
        let (indent, trimmed) = split_indent(line);
        if !is_anchorable(trimmed) {
            continue;
        }
        index.insert(trimmed, (start + indent) as u32);
    }

    index
}

/// Build a Source Map v3 document for `generated`, the exact emitted module.
///
/// `filename` becomes the map's single `sources` entry and the SFC source is
/// embedded as `sourcesContent`, so a consumer resolves a frame to the authored
/// `.vue` file without a second fetch.
///
/// Returns `None` when the SFC has no script block, or when no generated line
/// could be anchored — an empty map is worse than none, because it claims to
/// describe the module while resolving every position to nothing.
///
/// `generated` must be the bytes the caller actually hands on. The emitted
/// module is re-printed when TypeScript is stripped at the napi boundary, so a
/// map built before that pass does not describe the module after it.
pub fn build_sfc_source_map(
    generated: &str,
    descriptor: &SfcDescriptor<'_>,
    filename: &str,
) -> Option<String> {
    let source = descriptor.source.as_ref();
    let ranges = script_ranges(descriptor);
    if ranges.is_empty() {
        return None;
    }

    let index = index_script_lines(source, &ranges);
    if index.is_empty() {
        return None;
    }

    let mut builder = SourceMapBuilder::new();
    let mut mapped = 0usize;
    for (start, line) in lines_with_offsets(generated) {
        let (indent, trimmed) = split_indent(line);
        if !is_anchorable(trimmed) {
            continue;
        }
        if let Some(source_offset) = index.lookup(trimmed) {
            builder.add_raw(start + indent, source_offset);
            mapped += 1;
        }
    }

    if mapped == 0 {
        return None;
    }
    Some(builder.finish(generated, filename, source))
}

/// [`build_sfc_source_map`] gated on the codegen `source_map` flag, as the
/// `SfcCompileResult.map` field wants it.
pub(crate) fn sfc_source_map(
    generated: &str,
    descriptor: &SfcDescriptor<'_>,
    filename: &str,
    codegen_options: &vize_atelier_core::CodegenOptions,
) -> Option<serde_json::Value> {
    if !codegen_options.source_map {
        return None;
    }
    let json = build_sfc_source_map(generated, descriptor, filename)?;
    serde_json::from_str(&json).ok()
}

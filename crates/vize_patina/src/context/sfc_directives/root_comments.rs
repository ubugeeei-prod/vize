//! Root-level `<!-- eslint-… -->` comments of an SFC.
//!
//! eslint-plugin-vue applies a comment written between the blocks to the
//! top-level element that follows it: an `eslint-disable` left open is
//! cleared at that element's end. This is how a component silences a template
//! rule from the line above `<template>`.

use super::super::eslint_directive::{
    EslintDisableDirective, EslintDisableKind, InlineSuppressionState, parse_eslint_disable_comment,
};

/// Scan the comments of `source` that sit outside every block.
///
/// `blocks` holds the `(start, end)` of each block including its tags.
///
/// Returns `None` when no such comment carries a suppression pragma.
pub(super) fn scan(
    source: &str,
    mut blocks: Vec<(usize, usize)>,
    offset_to_line: impl Fn(usize) -> u32,
) -> Option<InlineSuppressionState> {
    blocks.sort_unstable();
    let mut state = InlineSuppressionState::default();
    let mut found = false;
    let mut at = 0;

    while let Some(open) = source.get(at..).and_then(|rest| rest.find("<!--")) {
        let start = at + open;
        // `<!--` inside a block is that block's own content.
        if let Some(&(_, block_end)) = blocks
            .iter()
            .find(|(block_start, block_end)| *block_start <= start && start < *block_end)
        {
            at = block_end.max(start + 1);
            continue;
        }
        let end = source
            .get(start..)
            .and_then(|rest| rest.find("-->"))
            .map_or(source.len(), |close| start + close + 3);
        at = end.max(start + 1);

        let Some(directive) = source
            .get(start..end)
            .and_then(parse_eslint_disable_comment)
        else {
            continue;
        };
        found = true;
        let line = match directive.kind {
            // "Next line" counts from where the comment ends.
            EslintDisableKind::DisableNextLine => offset_to_line(end.saturating_sub(1)),
            _ => offset_to_line(start),
        };
        let open_rules =
            matches!(directive.kind, EslintDisableKind::Disable).then(|| directive.rules.clone());
        state.record(directive, line);

        // An open region ends with the next top-level element.
        if let Some(rules) = open_rules
            && let Some(&(_, block_end)) =
                blocks.iter().find(|(block_start, _)| *block_start >= end)
        {
            let enable = EslintDisableDirective {
                kind: EslintDisableKind::Enable,
                rules,
            };
            state.record(enable, offset_to_line(block_end) + 1);
        }
    }

    found.then_some(state)
}

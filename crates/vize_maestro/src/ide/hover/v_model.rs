//! Hover range helpers for component `v-model` attributes.
#![cfg(feature = "native")]

use super::HoverService;

pub(super) fn argument_token_span(content: &str, offset: usize) -> Option<(usize, usize)> {
    let (token_start, token_end) =
        crate::ide::token_span_at_offset(content, offset, HoverService::is_word_char)?;
    let token = content.get(token_start..token_end)?;
    let argument = token.strip_prefix("v-model:")?;
    if argument.starts_with('[') {
        return None;
    }

    let argument_len = argument
        .split_once('.')
        .map_or(argument.len(), |(name, _)| name.len());
    if argument_len == 0 {
        return None;
    }

    let start = token_start + "v-model:".len();
    Some((start, start + argument_len))
}

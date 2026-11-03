//! The current strict Vue for-head text split, before any language parse.

use vize_l0::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForKeyword {
    In,
    Of,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Parts {
    pub aliases: Span,
    pub collection: Span,
    pub keyword: ForKeyword,
    pub separator: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SplitError {
    MissingSeparator,
    MissingCollection,
    UnpairedAliasParentheses,
    SourceTooLarge,
}

/// First whitespace-delimited in/of, matching the repository's strict grammar.
/// Untrimmed input matters: ` in xs` authors an empty alias position. Alias
/// commas are never scanned here; only the retained parameter AST determines
/// dense positions. Older repeat grammar and independent-paren quirks are not
/// admitted by this dialect entry point.
pub(crate) fn split(input: &str) -> Result<Parts, SplitError> {
    for (at, first) in input.char_indices() {
        if !matches!(first, 'i' | 'o') {
            continue;
        }
        let Some((before, from)) = input.split_at_checked(at) else {
            continue;
        };
        let (keyword, after) = if let Some(after) = from.strip_prefix("in") {
            (ForKeyword::In, after)
        } else if let Some(after) = from.strip_prefix("of") {
            (ForKeyword::Of, after)
        } else {
            continue;
        };
        if !before.chars().next_back().is_some_and(char::is_whitespace)
            || !after.chars().next().is_some_and(char::is_whitespace)
        {
            continue;
        }
        let collection = after.trim_start();
        if collection.trim_end().is_empty() {
            return Err(SplitError::MissingCollection);
        }
        let collection_start = input.len() - after.trim_start().len();
        let mut aliases = before.trim();
        let mut alias_start = before.len() - before.trim_start().len();
        let starts = aliases.starts_with('(');
        let ends = aliases.ends_with(')');
        if starts != ends {
            return Err(SplitError::UnpairedAliasParentheses);
        }
        if starts {
            let Some(inner) = aliases
                .strip_prefix('(')
                .and_then(|text| text.strip_suffix(')'))
            else {
                return Err(SplitError::UnpairedAliasParentheses);
            };
            alias_start += 1 + inner.len() - inner.trim_start().len();
            aliases = inner.trim();
        }
        return Ok(Parts {
            aliases: checked_span(alias_start, alias_start + aliases.len())?,
            collection: checked_span(collection_start, collection_start + collection.len())?,
            keyword,
            separator: checked_span(at, at + 2)?,
        });
    }
    Err(SplitError::MissingSeparator)
}

fn checked_span(start: usize, end: usize) -> Result<Span, SplitError> {
    Ok(Span::new(
        start.try_into().map_err(|_| SplitError::SourceTooLarge)?,
        end.try_into().map_err(|_| SplitError::SourceTooLarge)?,
    ))
}

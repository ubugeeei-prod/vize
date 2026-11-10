//! Punctuator lookup between AST nodes, skipping comments like ESLint's token store.

use oxc_span::Span;

/// The first `token` in `source[start..end]` that is not inside a comment.
pub(super) fn find_token(source: &str, start: u32, end: u32, token: &str) -> Option<Span> {
    let end = (end as usize).min(source.len());
    let mut at = start as usize;
    while at < end {
        let rest = source.get(at..end)?;
        let step = if let Some(comment) = rest.strip_prefix("/*") {
            comment.find("*/").map_or(rest.len(), |close| close + 4)
        } else if rest.starts_with("//") {
            rest.find(['\n', '\r', '\u{2028}', '\u{2029}'])
                .unwrap_or(rest.len())
        } else if rest.starts_with(token) {
            let token_end = at.checked_add(token.len())?;
            return Some(Span::new(
                u32::try_from(at).ok()?,
                u32::try_from(token_end).ok()?,
            ));
        } else {
            rest.chars().next().map_or(1, char::len_utf8)
        };
        at = at.checked_add(step)?;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::find_token;

    #[test]
    fn skips_comments_and_parentheses() {
        let source = "(a) /* == */ != (b)";
        let span = find_token(source, 3, 16, "!=");
        assert_eq!(span.map(|span| (span.start, span.end)), Some((13, 15)));
        let source = "a // ,\n , b";
        let span = find_token(source, 1, 11, ",");
        assert_eq!(span.map(|span| span.start), Some(8));
    }
}

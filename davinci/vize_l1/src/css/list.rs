//! Class lists from the same original rule tokens, without another parse.

use super::{StyleIssue, StyleIssueCode, StyleSyntax};
use cssparser::Token;

/// Unforgeable receipt for a complete comma-separated simple-class selector.
/// Every insertion is the end of an actual original classname token.
///
/// ```compile_fail
/// use vize_l1::css::SimpleClassListStyle;
/// let _ = SimpleClassListStyle { count: 2, insertions: [1, 2] };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SimpleClassListStyle<'o, 'a> {
    syntax: &'o StyleSyntax<'a>,
    count: usize,
}

impl<'o, 'a> SimpleClassListStyle<'o, 'a> {
    #[must_use]
    pub const fn syntax(self) -> &'o StyleSyntax<'a> {
        self.syntax
    }
    #[must_use]
    pub const fn class_count(self) -> usize {
        self.count
    }
    /// These offsets borrow the validated actual token sequence; no caller
    /// list, reparsed selector or normalized spelling supplies authority.
    pub fn insertions(self) -> impl Iterator<Item = u32> + 'o {
        self.syntax
            .rule()
            .into_iter()
            .flat_map(|rule| rule.prelude().iter())
            .filter_map(|token| {
                matches!(token.token(), Token::Ident(_)).then_some(token.span().end)
            })
    }
}

pub(super) fn observe<'o, 'a>(
    syntax: &'o StyleSyntax<'a>,
) -> Result<SimpleClassListStyle<'o, 'a>, StyleIssue> {
    if let Some(issue) = syntax.issue() {
        return Err(issue);
    }
    let unsupported = || StyleIssue {
        code: StyleIssueCode::UnsupportedSelector,
        span: syntax.source().span(),
    };
    let rule = syntax.rule().ok_or_else(unsupported)?;
    let mut tokens = rule.prelude().iter().peekable();
    while tokens
        .peek()
        .is_some_and(|token| matches!(token.token(), Token::WhiteSpace(_) | Token::Comment(_)))
    {
        tokens.next();
    }
    let mut count = 0;
    loop {
        let dot = tokens.next().ok_or_else(unsupported)?;
        let name = tokens.next().ok_or_else(unsupported)?;
        let Token::Ident(decoded) = name.token() else {
            return Err(unsupported());
        };
        if !matches!(dot.token(), Token::Delim('.'))
            || syntax
                .source()
                .root_source()
                .get(name.span().start as usize..name.span().end as usize)
                != Some(decoded.as_ref())
        {
            return Err(unsupported());
        }
        count += 1;
        let mut whitespace = false;
        while tokens
            .peek()
            .is_some_and(|token| matches!(token.token(), Token::WhiteSpace(_)))
        {
            whitespace = true;
            tokens.next();
        }
        match tokens.next() {
            None if count >= 2 => return Ok(SimpleClassListStyle { syntax, count }),
            Some(comma) if !whitespace && matches!(comma.token(), Token::Comma) => {
                while tokens
                    .peek()
                    .is_some_and(|token| matches!(token.token(), Token::WhiteSpace(_)))
                {
                    tokens.next();
                }
            }
            _ => return Err(unsupported()),
        }
    }
}

#[cfg(test)]
mod tests;

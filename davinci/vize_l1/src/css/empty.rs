//! The fixed empty-pseudo window borrows actual original parser tokens.

use super::{CssToken, StyleIssue, StyleIssueCode, StyleSyntax};
use cssparser::Token;
use vize_l0::Span;

/// Original classname, colon and literal empty-pseudo tokens from one rule.
/// No caller-supplied selector, offsets or pseudo kind grants authority.
///
/// ```compile_fail
/// use vize_l1::css::EmptyClassStyle;
/// let _ = EmptyClassStyle { insertion: 2, pseudo: "empty" };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct EmptyClassStyle<'o, 'a> {
    syntax: &'o StyleSyntax<'a>,
    name: &'o CssToken<'a>,
    colon: &'o CssToken<'a>,
    pseudo: &'o CssToken<'a>,
}
impl<'o, 'a> EmptyClassStyle<'o, 'a> {
    #[must_use]
    pub const fn syntax(self) -> &'o StyleSyntax<'a> {
        self.syntax
    }
    #[must_use]
    pub const fn class_token(self) -> &'o CssToken<'a> {
        self.name
    }
    #[must_use]
    pub const fn pseudo_tokens(self) -> (&'o CssToken<'a>, &'o CssToken<'a>) {
        (self.colon, self.pseudo)
    }
    #[must_use]
    pub const fn insertion(self) -> u32 {
        self.name.span().end
    }
    #[must_use]
    pub const fn pseudo_span(self) -> Span {
        Span::new(self.colon.span().start, self.pseudo.span().end)
    }
}

pub(super) fn observe<'o, 'a>(
    syntax: &'o StyleSyntax<'a>,
) -> Result<EmptyClassStyle<'o, 'a>, StyleIssue> {
    if let Some(issue) = syntax.issue() {
        return Err(issue);
    }
    let unsupported = || StyleIssue {
        code: StyleIssueCode::UnsupportedSelector,
        span: syntax.source().span(),
    };
    let rule = syntax.rule().ok_or_else(unsupported)?;
    let [dot, name, colon, pseudo, tail @ ..] = rule.prelude() else {
        return Err(unsupported());
    };
    let Token::Ident(classname) = name.token() else {
        return Err(unsupported());
    };
    let literal = |token: &CssToken<'_>, expected: &str| {
        syntax
            .source()
            .root_source()
            .get(token.span().start as usize..token.span().end as usize)
            == Some(expected)
    };
    if !matches!(dot.token(), Token::Delim('.'))
        || !matches!(colon.token(), Token::Colon)
        || !matches!(pseudo.token(), Token::Ident(value) if value.as_ref() == "empty")
        || !literal(name, classname.as_ref())
        || !literal(pseudo, "empty")
        || tail.len() > 1
        || tail
            .first()
            .is_some_and(|token| !matches!(token.token(), Token::WhiteSpace(_)))
    {
        return Err(unsupported());
    }
    Ok(EmptyClassStyle {
        syntax,
        name,
        colon,
        pseudo,
    })
}

#[cfg(test)]
mod tests;

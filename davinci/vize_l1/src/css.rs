//! One production CSS parse over an original source-checked SFC style block.
//!
//! This bounded owner retains actual cssparser tokens and errors. It does not
//! normalize CSS or claim property semantics, nested rules or Vue bindings.

use crate::container::vue::StyleView;
use alloc::vec::Vec;
use cssparser::{CowRcStr, ParseError, ParseErrorKind, Token};
use vize_l0::{SourceBlock, Span};

mod parser;

/// Why this original syntax cannot lend the simple-class scoped family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleIssueCode {
    UnsupportedLanguage,
    Syntax,
    UnsupportedSelector,
    UnsupportedValue,
    MissingBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyleIssue {
    pub code: StyleIssueCode,
    pub span: Span,
}

/// An actual token returned by the pinned parser, with whole-file byte bounds.
#[derive(Debug)]
pub struct CssToken<'a> {
    token: Token<'a>,
    span: Span,
}
impl<'a> CssToken<'a> {
    #[must_use]
    pub fn token(&self) -> &Token<'a> {
        &self.token
    }
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }
}

/// Actual declaration callback data, retained even if its value is unavailable.
#[derive(Debug)]
pub struct CssDeclaration<'a> {
    name: CowRcStr<'a>,
    span: Span,
    value: Vec<CssToken<'a>>,
}
impl<'a> CssDeclaration<'a> {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }
    #[must_use]
    pub fn value(&self) -> &[CssToken<'a>] {
        &self.value
    }
}

/// The qualified-rule callbacks from the same parser over the original block.
#[derive(Debug)]
pub struct CssRule<'a> {
    prelude: Vec<CssToken<'a>>,
    declarations: Vec<CssDeclaration<'a>>,
}
impl<'a> CssRule<'a> {
    #[must_use]
    pub fn prelude(&self) -> &[CssToken<'a>] {
        &self.prelude
    }
    #[must_use]
    pub fn declarations(&self) -> &[CssDeclaration<'a>] {
        &self.declarations
    }
}

/// Complete original source beside one actual parse and any retained refusal.
/// Private construction requires a genuine original descriptor StyleView.
///
/// ```compile_fail
/// use vize_l1::css::StyleSyntax;
/// let _ = StyleSyntax { source: "replacement", container_index: 0 };
/// ```
#[derive(Debug)]
pub struct StyleSyntax<'a> {
    source: SourceBlock<'a>,
    container_index: usize,
    rule: Option<CssRule<'a>>,
    error: Option<ParseError<'a, StyleIssue>>,
    issue: Option<StyleIssue>,
}

impl<'a> StyleSyntax<'a> {
    /// Parse the original CSS slice once; non-CSS languages stay unparsed.
    #[must_use]
    pub fn observe(original: StyleView<'_, 'a>) -> Self {
        let source = original.block();
        let mut syntax = Self {
            source,
            container_index: original.container_index(),
            rule: None,
            error: None,
            issue: None,
        };
        if original
            .attrs()
            .iter()
            .any(|attr| attr.name == "lang" && attr.value != Some("css"))
        {
            syntax.issue = Some(StyleIssue {
                code: StyleIssueCode::UnsupportedLanguage,
                span: source.span(),
            });
            return syntax;
        }
        parser::observe(&mut syntax);
        syntax
    }
    #[must_use]
    pub const fn source(&self) -> SourceBlock<'a> {
        self.source
    }
    #[must_use]
    pub const fn container_index(&self) -> usize {
        self.container_index
    }
    #[must_use]
    pub fn rule(&self) -> Option<&CssRule<'a>> {
        self.rule.as_ref()
    }
    /// Actual parser error, including local CSS UTF-16 line/column and token.
    #[must_use]
    pub fn parser_error(&self) -> Option<&ParseError<'a, StyleIssue>> {
        self.error.as_ref()
    }
    #[must_use]
    pub fn issue(&self) -> Option<StyleIssue> {
        self.issue.or_else(|| {
            self.error.as_ref().map(|error| match &error.kind {
                ParseErrorKind::Custom(issue) => *issue,
                ParseErrorKind::Basic(_) => StyleIssue {
                    code: StyleIssueCode::Syntax,
                    span: self.source.span(),
                },
            })
        })
    }
    /// Lend exactly one complete `.class` qualified rule, with flat values.
    /// No replacement token list, source text or caller status is accepted.
    pub fn simple_class(&self) -> Result<SimpleClassStyle<'_, 'a>, StyleIssue> {
        if let Some(issue) = self.issue() {
            return Err(issue);
        }
        let unsupported = || StyleIssue {
            code: StyleIssueCode::UnsupportedSelector,
            span: self.source.span(),
        };
        let rule = self.rule.as_ref().ok_or_else(unsupported)?;
        let mut tokens = rule
            .prelude
            .iter()
            .filter(|token| !matches!(token.token, Token::Comment(_)))
            .skip_while(|token| matches!(token.token, Token::WhiteSpace(_)));
        let dot = tokens.next().ok_or_else(unsupported)?;
        let name = tokens.next().ok_or_else(unsupported)?;
        if !matches!(dot.token, Token::Delim('.'))
            || !matches!(name.token, Token::Ident(_))
            || tokens.any(|token| !matches!(token.token, Token::WhiteSpace(_)))
        {
            return Err(unsupported());
        }
        Ok(SimpleClassStyle {
            syntax: self,
            insertion: name.span.end,
        })
    }
}

/// Unforgeable whole-style receipt for the parser-proven insertion position.
#[derive(Debug, Clone, Copy)]
pub struct SimpleClassStyle<'o, 'a> {
    syntax: &'o StyleSyntax<'a>,
    insertion: u32,
}
impl<'o, 'a> SimpleClassStyle<'o, 'a> {
    #[must_use]
    pub const fn syntax(self) -> &'o StyleSyntax<'a> {
        self.syntax
    }
    #[must_use]
    pub const fn insertion(self) -> u32 {
        self.insertion
    }
}

#[cfg(test)]
mod tests;

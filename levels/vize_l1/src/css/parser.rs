//! The pinned CSS library supplies rule, declaration and token events directly.

use super::{CssDeclaration, CssRule, CssToken, StyleIssue, StyleIssueCode, StyleSyntax};
use alloc::vec::Vec;
use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserInput, ParserState,
    QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, Token, parse_one_rule,
};
use vize_l0::{SourceBlock, Span};

pub(super) fn observe(syntax: &mut StyleSyntax<'_>) {
    let mut input = ParserInput::new(syntax.source.source());
    let mut parser = Parser::new(&mut input);
    let mut callbacks = Callbacks {
        source: syntax.source,
        rule: None,
    };
    syntax.error = parse_one_rule(&mut parser, &mut callbacks).err();
    syntax.rule = callbacks.rule;
}

struct Callbacks<'a> {
    source: SourceBlock<'a>,
    rule: Option<CssRule<'a>>,
}
impl Callbacks<'_> {
    fn span(&self, start: usize, end: usize) -> Span {
        Span::new(
            self.source.start() + start as u32,
            self.source.start() + end as u32,
        )
    }
}

impl<'a> QualifiedRuleParser<'a> for Callbacks<'a> {
    type Prelude = ();
    type QualifiedRule = ();
    type Error = StyleIssue;
    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'a, 't>,
    ) -> Result<(), ParseError<'a, StyleIssue>> {
        let mut prelude = Vec::new();
        let result = tokens(self.source, input, &mut prelude);
        self.rule = Some(CssRule {
            prelude,
            declarations: Vec::new(),
        });
        result
    }
    fn parse_block<'t>(
        &mut self,
        _: (),
        _: &ParserState,
        input: &mut Parser<'a, 't>,
    ) -> Result<(), ParseError<'a, StyleIssue>> {
        for declaration in RuleBodyParser::new(input, self) {
            declaration.map_err(|(error, _)| error)?;
        }
        // CSS Syntax permits EOF recovery. This scoped family requires the
        // genuine parser stop position to be an authored closing brace.
        let end = input.position().byte_index();
        if self.source.source().get(end..end + 1) != Some("}") {
            return Err(input.new_custom_error(StyleIssue {
                code: StyleIssueCode::MissingBoundary,
                span: self.span(end, end),
            }));
        }
        Ok(())
    }
}
impl<'a> AtRuleParser<'a> for Callbacks<'a> {
    type Prelude = ();
    type AtRule = ();
    type Error = StyleIssue;
}
impl<'a> DeclarationParser<'a> for Callbacks<'a> {
    type Declaration = ();
    type Error = StyleIssue;
    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'a>,
        input: &mut Parser<'a, 't>,
        start: &ParserState,
    ) -> Result<(), ParseError<'a, StyleIssue>> {
        let mut value = Vec::new();
        let result = tokens(self.source, input, &mut value);
        let span = self.span(start.position().byte_index(), input.position().byte_index());
        if let Some(rule) = self.rule.as_mut() {
            rule.declarations.push(CssDeclaration { name, span, value });
        }
        result
    }
}
impl<'a> RuleBodyItemParser<'a, (), StyleIssue> for Callbacks<'a> {
    fn parse_declarations(&self) -> bool {
        true
    }
    fn parse_qualified(&self) -> bool {
        false
    }
}

fn tokens<'a>(
    source: SourceBlock<'a>,
    input: &mut Parser<'a, '_>,
    output: &mut Vec<CssToken<'a>>,
) -> Result<(), ParseError<'a, StyleIssue>> {
    loop {
        let start = input.position().byte_index();
        let token = match input.next_including_whitespace_and_comments() {
            Ok(token) => token.clone(),
            Err(error) if matches!(error.kind, cssparser::BasicParseErrorKind::EndOfInput) => {
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };
        let end = input.position().byte_index();
        let span = Span::new(source.start() + start as u32, source.start() + end as u32);
        let raw = source.source().get(start..end).unwrap_or_default();
        let issue = if token.is_parse_error() {
            Some(StyleIssueCode::Syntax)
        } else if matches!(
            token,
            Token::Function(_)
                | Token::ParenthesisBlock
                | Token::SquareBracketBlock
                | Token::CurlyBracketBlock
        ) {
            Some(StyleIssueCode::UnsupportedValue)
        } else if matches!(token, Token::Comment(_)) && !raw.ends_with("*/") {
            Some(StyleIssueCode::MissingBoundary)
        } else if let Token::QuotedString(ref decoded) = token {
            if raw.len() < 2 || raw.as_bytes().first() != raw.as_bytes().last() {
                Some(StyleIssueCode::MissingBoundary)
            } else if raw.get(1..raw.len() - 1) != Some(decoded.as_ref()) {
                Some(StyleIssueCode::UnsupportedValue)
            } else if decoded.contains("v-bind") || decoded.contains("/*") {
                // Vue observes binding spellings inside quoted declaration
                // tokens too; comment markers can bridge its CSS-var spelling.
                // The original returned token retains the refusal authority.
                Some(StyleIssueCode::UnsupportedValue)
            } else {
                None
            }
        } else {
            None
        };
        output.push(CssToken { token, span });
        if let Some(code) = issue {
            return Err(input.new_custom_error(StyleIssue { code, span }));
        }
    }
}

//! Authored token spans in declarations of an already accepted stylesheet.

use cssparser::{
    AtRuleParser, BasicParseError, CowRcStr, DeclarationParser, ParseError, Parser, ParserInput,
    ParserState, QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, StyleSheetParser, Token,
};

#[derive(Default)]
pub(super) struct ValueTokens {
    pub(super) bindings: Vec<(usize, usize)>,
    pub(super) important: Vec<(usize, usize)>,
    declarations: bool,
}

impl ValueTokens {
    pub(super) fn new(source: &str) -> Self {
        let mut input = ParserInput::new(source);
        let mut parser = Parser::new(&mut input);
        let mut tokens = Self::default();
        for _ in StyleSheetParser::new(&mut parser, &mut tokens) {}
        tokens.bindings.sort_unstable();
        tokens.important.sort_unstable();
        tokens
    }

    fn body<'i>(&mut self, input: &mut Parser<'i, '_>) {
        let previous = self.declarations;
        self.declarations = true;
        for _ in RuleBodyParser::new(input, self) {}
        self.declarations = previous;
    }
}

impl<'i> DeclarationParser<'i> for ValueTokens {
    type Declaration = ();
    type Error = ();

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        _start: &ParserState,
    ) -> Result<(), ParseError<'i, ()>> {
        // Stage the spans: a nested selector can initially look like a
        // declaration. Its rejected value must publish no findings.
        let mut bindings = Vec::new();
        let mut important = None;
        loop {
            let position = input.position();
            let start = position.byte_index();
            let Ok(token) = input.next_including_whitespace_and_comments().cloned() else {
                break;
            };
            match token {
                Token::Delim('!') => {
                    let flag = input.try_parse(|input| {
                        input.expect_ident_matching("important")?;
                        let end = input.position().byte_index();
                        input.expect_exhausted()?;
                        Ok::<_, BasicParseError<'i>>(end)
                    });
                    if let Ok(end) = flag {
                        important = Some((start, end));
                    }
                }
                Token::CurlyBracketBlock if !name.starts_with("--") => {
                    return Err(input.new_custom_error(()));
                }
                Token::Function(name) => {
                    let binding =
                        name.as_ref() == "v-bind" && input.slice_from(position) == "v-bind(";
                    input.parse_nested_block(|nested| collect_functions(nested, &mut bindings))?;
                    if binding {
                        bindings.push((start, input.position().byte_index()));
                    }
                }
                Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock => {
                    input.parse_nested_block(|nested| collect_functions(nested, &mut bindings))?;
                }
                _ => {}
            }
        }
        self.bindings.extend(bindings);
        self.important.extend(important);
        Ok(())
    }
}

impl<'i> AtRuleParser<'i> for ValueTokens {
    type Prelude = ();
    type AtRule = ();
    type Error = ();

    fn parse_prelude<'t>(
        &mut self,
        _name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<(), ParseError<'i, ()>> {
        while input.next().is_ok() {}
        Ok(())
    }

    fn parse_block<'t>(
        &mut self,
        (): (),
        _start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<(), ParseError<'i, ()>> {
        self.body(input);
        Ok(())
    }
}

impl<'i> QualifiedRuleParser<'i> for ValueTokens {
    type Prelude = ();
    type QualifiedRule = ();
    type Error = ();

    fn parse_prelude<'t>(&mut self, input: &mut Parser<'i, 't>) -> Result<(), ParseError<'i, ()>> {
        while input.next().is_ok() {}
        Ok(())
    }

    fn parse_block<'t>(
        &mut self,
        (): (),
        _start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<(), ParseError<'i, ()>> {
        self.body(input);
        Ok(())
    }
}

impl<'i> RuleBodyItemParser<'i, (), ()> for ValueTokens {
    fn parse_declarations(&self) -> bool {
        self.declarations
    }

    fn parse_qualified(&self) -> bool {
        true
    }
}

fn collect_functions<'i>(
    input: &mut Parser<'i, '_>,
    out: &mut Vec<(usize, usize)>,
) -> Result<(), ParseError<'i, ()>> {
    loop {
        let position = input.position();
        let start = position.byte_index();
        let Ok(token) = input.next_including_whitespace_and_comments().cloned() else {
            return Ok(());
        };
        match token {
            Token::Function(name) => {
                let binding = name.as_ref() == "v-bind" && input.slice_from(position) == "v-bind(";
                input.parse_nested_block(|nested| collect_functions(nested, out))?;
                if binding {
                    out.push((start, input.position().byte_index()));
                }
            }
            Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock => {
                input.parse_nested_block(|nested| collect_functions(nested, out))?;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;

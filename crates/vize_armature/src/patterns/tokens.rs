use oxc_ast::ast::Expression;
use oxc_span::{GetSpan, SourceType, Span};
use oxc_syntax::identifier::{is_identifier_part, is_identifier_start};
use vize_s0::{
    String,
    expression_guard::{expression_is_safe_to_parse, is_expression_trailing_trivia},
};

use super::{
    PatternBinding, PatternExpression,
    parser::{PatternParser, Result},
};

impl PatternParser<'_> {
    pub fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    pub fn space(&mut self) {
        while let Some(c) = self.peek() {
            if !c.is_whitespace() && c != '\u{feff}' {
                break;
            }
            self.pos += c.len_utf8();
        }
    }

    pub fn eat(&mut self, token: &str) -> bool {
        self.space();
        if !self.source[self.pos..].starts_with(token) {
            return false;
        }
        self.pos += token.len();
        true
    }

    pub fn word(&mut self, token: &str) -> bool {
        self.space();
        let Some(rest) = self.source[self.pos..].strip_prefix(token) else {
            return false;
        };
        if rest.chars().next().is_some_and(is_identifier_part) {
            return false;
        }
        self.pos += token.len();
        true
    }

    pub fn expect(&mut self, token: &str) -> Result<()> {
        if self.eat(token) {
            Ok(())
        } else {
            Err(self.error(&vize_s0::cstr!("Expected {token} in pattern.")))
        }
    }

    pub fn name(&mut self) -> Result<PatternBinding> {
        self.space();
        let start = self.pos;
        let Some(first) = self.peek().filter(|c| is_identifier_start(*c)) else {
            return Err(self.error("Expected an identifier."));
        };
        self.pos += first.len_utf8();
        while let Some(c) = self.peek().filter(|c| is_identifier_part(*c)) {
            self.pos += c.len_utf8();
        }
        Ok(PatternBinding {
            name: String::from(&self.source[start..self.pos]),
            span: Span::new(start as u32, self.pos as u32),
        })
    }

    pub fn string(&mut self) -> Result<(PatternExpression, Vec<u16>)> {
        let start = self.pos;
        let quote = self
            .peek()
            .ok_or_else(|| self.error("Expected a string."))?;
        self.pos += 1;
        while let Some(c) = self.peek() {
            self.pos += c.len_utf8();
            if c == quote {
                let text = self.expression(start);
                let parsed = oxc_parser::Parser::new(&self.js, &text.text, SourceType::mjs())
                    .parse_expression()
                    .map_err(|_| self.error("Invalid string escape in pattern."))?;
                let Expression::StringLiteral(literal) = parsed else {
                    return Err(self.error("Expected a string literal."));
                };
                return Ok((
                    text.clone(),
                    string_units(literal.value.as_str(), literal.lone_surrogates),
                ));
            }
            if c == '\n' || c == '\r' {
                return Err(self.error("Unterminated string in pattern."));
            }
            if c == '\\' {
                let Some(escaped) = self.peek() else {
                    break;
                };
                if escaped == '\n' || escaped == '\r' {
                    return Err(self.error("Invalid string escape in pattern."));
                }
                self.pos += escaped.len_utf8();
                if matches!(escaped, '1'..='9')
                    || (escaped == '0' && self.peek().is_some_and(|c| c.is_ascii_digit()))
                {
                    return Err(self.error("Legacy octal escapes are not supported."));
                }
            }
        }
        Err(self.error("Unterminated string in pattern."))
    }

    pub fn number(&mut self, signed: bool) -> Result<(PatternExpression, Option<f64>)> {
        let start = self.pos;
        if signed && matches!(self.peek(), Some('+' | '-')) {
            self.pos += 1;
        }
        let unsigned = self.pos;
        let radix = match self.source.as_bytes().get(unsigned..unsigned + 2) {
            Some(b"0x" | b"0X") => Some(16),
            Some(b"0b" | b"0B") => Some(2),
            Some(b"0o" | b"0O") => Some(8),
            _ => None,
        };
        if let Some(radix) = radix {
            self.pos += 2;
            let digits = self.pos;
            while self
                .peek()
                .is_some_and(|c| c.is_ascii() && c.is_digit(radix))
            {
                self.pos += 1;
            }
            if self.pos == digits {
                return Err(self.error("Expected digits in numeric literal."));
            }
            if self.peek() == Some('n') {
                self.pos += 1;
            }
        } else {
            self.digits();
            let mut integer = true;
            if self.peek() == Some('.') {
                integer = false;
                self.pos += 1;
                self.digits();
            }
            if self.pos == unsigned || &self.source[unsigned..self.pos] == "." {
                return Err(self.error("Expected a numeric literal."));
            }
            if matches!(self.peek(), Some('e' | 'E')) {
                integer = false;
                self.pos += 1;
                if matches!(self.peek(), Some('+' | '-')) {
                    self.pos += 1;
                }
                let exponent = self.pos;
                self.digits();
                if exponent == self.pos {
                    return Err(self.error("Expected exponent digits."));
                }
            }
            if integer && self.peek() == Some('n') {
                self.pos += 1;
            }
        }
        let text = self.expression(start);
        if text.text.starts_with('+') && text.text.ends_with('n') {
            return Err(self.error("Unary plus cannot be used with bigint."));
        }
        let parsed = oxc_parser::Parser::new(&self.js, &text.text, SourceType::mjs())
            .parse_expression()
            .map_err(|_| self.error("Invalid numeric literal."))?;
        let literal = match &parsed {
            Expression::NumericLiteral(_) | Expression::BigIntLiteral(_) => true,
            Expression::UnaryExpression(unary) => matches!(
                &unary.argument,
                Expression::NumericLiteral(_) | Expression::BigIntLiteral(_)
            ),
            _ => false,
        };
        if !literal || parsed.span().end as usize != text.text.len() {
            return Err(self.error("Invalid numeric literal."));
        }
        let value = if let Expression::NumericLiteral(literal) = parsed {
            Some(literal.value)
        } else {
            None
        };
        Ok((text.clone(), value))
    }

    fn digits(&mut self) {
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.pos += 1;
        }
    }

    pub fn validate_expression<'a>(
        &'a self,
        source: &'a str,
        kind: &str,
    ) -> Result<Expression<'a>> {
        if !expression_is_safe_to_parse(source) {
            return Err(self.error(&vize_s0::cstr!("Unsafe or unbalanced {kind} expression.")));
        }
        let expression = oxc_parser::Parser::new(&self.js, source, SourceType::ts())
            .parse_expression()
            .map_err(|_| self.error(&vize_s0::cstr!("Invalid {kind} expression.")))?;
        if !is_expression_trailing_trivia(&source[expression.span().end as usize..]) {
            return Err(self.error(&vize_s0::cstr!("Unexpected token after {kind} expression.")));
        }
        Ok(expression)
    }
}

// OXC escapes every replacement character when a string contains lone surrogates.
fn string_units(value: &str, escaped_surrogates: bool) -> Vec<u16> {
    if !escaped_surrogates {
        return value.encode_utf16().collect();
    }
    let mut result = Vec::new();
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c == '\u{fffd}' {
            let mut unit = 0u16;
            for _ in 0..4 {
                unit = unit * 16
                    + chars
                        .next()
                        .and_then(|c| c.to_digit(16))
                        .expect("OXC surrogate encoding") as u16;
            }
            result.push(unit);
        } else {
            let mut units = [0; 2];
            result.extend_from_slice(c.encode_utf16(&mut units));
        }
    }
    result
}

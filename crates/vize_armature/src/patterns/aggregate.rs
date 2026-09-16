use oxc_span::Span;
use oxc_syntax::number::ToJsString;
use vize_s0::{FxHashSet, cstr};

use super::{
    MatchPattern, PatternExpression, PatternKind, PatternProperty, PatternRest,
    parser::{PatternParser, Result},
};

impl PatternParser<'_> {
    pub fn object(&mut self) -> Result<PatternKind> {
        let mut properties = Vec::new();
        let mut keys = FxHashSet::default();
        let mut rest = None;
        while !self.eat("}") {
            if self.eat("...") {
                rest = Some(self.rest("}")?);
                self.expect("}")?;
                break;
            }
            self.space();
            let start = self.pos;
            let (key, key_value, pattern) = if self.word("const") {
                let binding = self.bind()?;
                let key = PatternExpression {
                    text: cstr!("\"{}\"", binding.name),
                    span: binding.span,
                };
                let key_value = binding.name.encode_utf16().collect();
                let pattern = MatchPattern {
                    span: binding.span,
                    kind: PatternKind::Binding(binding),
                };
                (key, key_value, pattern)
            } else {
                let (key, value) = self.key()?;
                self.expect(":")?;
                (key, value, self.pattern()?)
            };
            if !keys.insert(key_value.clone()) {
                return Err(self.error("Duplicate pattern property."));
            }
            properties.push(PatternProperty {
                key,
                key_value,
                pattern,
                span: Span::new(start as u32, self.pos as u32),
            });
            if self.eat("}") {
                break;
            }
            self.expect(",")?;
        }
        Ok(PatternKind::Object { properties, rest })
    }

    pub fn array(&mut self) -> Result<PatternKind> {
        let mut elements = Vec::new();
        let mut rest = None;
        while !self.eat("]") {
            if self.eat("...") {
                rest = Some(self.rest("]")?);
                self.expect("]")?;
                break;
            }
            elements.push(self.pattern()?);
            if self.eat("]") {
                break;
            }
            self.expect(",")?;
        }
        Ok(PatternKind::Array { elements, rest })
    }

    fn rest(&mut self, close: &str) -> Result<PatternRest> {
        let start = self.pos - 3;
        let binding = if self.word("const") {
            Some(self.bind()?)
        } else {
            None
        };
        self.space();
        if !self.source[self.pos..].starts_with(close) {
            return Err(self
                .error("Rest must be last, without a trailing comma; use ... or ...const name."));
        }
        Ok(PatternRest {
            binding,
            span: Span::new(start as u32, self.pos as u32),
        })
    }

    fn key(&mut self) -> Result<(PatternExpression, Vec<u16>)> {
        self.space();
        if matches!(self.peek(), Some('\'' | '"')) {
            return self.string();
        }
        if self.peek().is_some_and(|c| c.is_ascii_digit() || c == '.') {
            let (mut number, value) = self.number(false)?;
            let Some(value) = value else {
                return Err(self.error("Bigint object keys are not supported."));
            };
            let value = value.to_js_string();
            number.text = cstr!("\"{value}\"");
            return Ok((number, value.encode_utf16().collect()));
        }
        let binding = self.name()?;
        let value = binding.name.encode_utf16().collect();
        Ok((
            PatternExpression {
                text: cstr!("\"{}\"", binding.name),
                span: binding.span,
            },
            value,
        ))
    }
}

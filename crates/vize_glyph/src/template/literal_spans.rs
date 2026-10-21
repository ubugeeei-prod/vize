//! Raw-line ownership of a directive value from its parsed string and
//! template-literal spans.
//!
//! A line that starts inside a quasi or a continued string literal belongs to
//! the runtime value and is printed verbatim. When the value parses, the
//! parser's spans decide that exactly, so a backtick inside a string or comment
//! is never mistaken for a literal (#9583). Values that are not JavaScript on
//! their own (`v-for` aliases, partial or invalid code) keep the lexical
//! ownership of [`LiteralLineState`].

use oxc_allocator::Allocator;
use oxc_ast::ast::{StringLiteral, TemplateElement, TemplateLiteral};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l1::markup::entity::{EntityContext, decode_one};

use super::literal_lines::{LiteralLineState, Representation};

/// Line ownership for one value, by parsed spans when available.
pub(crate) enum ValueLines {
    /// `raw[n]` is whether value line `n` starts inside a literal; `next` is
    /// the line the next `line_is_raw` call answers for.
    Parsed {
        raw: Vec<bool>,
        next: usize,
    },
    Lexed(LiteralLineState),
}

impl ValueLines {
    /// Ownership for `value`, positioned before its first line.
    pub(crate) fn new(value: &str, representation: Representation) -> Self {
        parsed_raw_lines(value, representation).map_or_else(
            || Self::Lexed(LiteralLineState::from_line("", representation)),
            |raw| Self::Parsed { raw, next: 0 },
        )
    }

    /// Ownership for the lines after the first of a rendered `name="value"`.
    pub(crate) fn for_attribute(attr: &str) -> Self {
        let value = attr
            .split_once('=')
            .and_then(|(_, value)| {
                let quote = value.chars().next().filter(|c| matches!(c, '"' | '\''))?;
                let value = value.get(1..)?;
                Some(value.strip_suffix(quote).unwrap_or(value))
            })
            .unwrap_or_default();
        match parsed_raw_lines(value, Representation::Html) {
            Some(raw) => Self::Parsed { raw, next: 1 },
            None => Self::Lexed(LiteralLineState::from_attribute(
                attr.split('\n').next().unwrap_or_default(),
            )),
        }
    }

    pub(crate) fn line_is_raw(&self) -> bool {
        match self {
            Self::Parsed { raw, next } => raw.get(*next).copied().unwrap_or(false),
            Self::Lexed(state) => state.line_is_raw(),
        }
    }

    pub(crate) fn line_holds_code(&self, line: &str) -> bool {
        !self.line_is_raw() && !line.trim().is_empty()
    }

    pub(crate) fn advance_line(&mut self, line: &str) {
        match self {
            Self::Parsed { next, .. } => *next += 1,
            Self::Lexed(state) => state.advance_line(line),
        }
    }
}

/// Per-line "starts inside a string or quasi" flags for a value that parses
/// as one expression (or, outside interpolations, as handler statements).
pub(crate) fn parsed_raw_lines(value: &str, representation: Representation) -> Option<Vec<bool>> {
    if !value.contains('\n') {
        return None;
    }
    let decoded = match representation {
        Representation::Html => decode_attribute(value)?,
        Representation::JavaScript | Representation::JavaScriptInterpolation => value.into(),
    };
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    // Directive printers keep a whole literal, substitutions included, as the
    // expression formatter laid it out; interpolations own only the quasis.
    let mut spans = LiteralSpans {
        spans: Vec::new(),
        whole_templates: !matches!(representation, Representation::JavaScriptInterpolation),
    };
    match Parser::new(&allocator, &decoded, source_type).parse_expression() {
        Ok(expression) => spans.visit_expression(&expression),
        Err(_) if !matches!(representation, Representation::JavaScriptInterpolation) => {
            let parsed = Parser::new(&allocator, &decoded, source_type).parse();
            if !parsed.diagnostics.is_empty() || parsed.panicked {
                return None;
            }
            spans.visit_program(&parsed.program);
        }
        Err(_) => return None,
    }
    let mut raw = vec![false];
    for (index, byte) in decoded.bytes().enumerate() {
        if byte == b'\n' {
            let start = (index + 1) as u32;
            raw.push(
                spans
                    .spans
                    .iter()
                    .any(|&(span_start, span_end)| span_start < start && start <= span_end),
            );
        }
    }
    Some(raw)
}

/// Decode attribute character references; a reference that decodes to a line
/// terminator would change the line structure, so such values stay lexical.
fn decode_attribute(value: &str) -> Option<vize_l0::String> {
    let mut decoded = vize_l0::String::with_capacity(value.len());
    let mut rest = value;
    while let Some(amp) = rest.find('&') {
        decoded.push_str(rest.get(..amp)?);
        let tail = rest.get(amp..)?;
        match decode_one(tail.as_bytes(), EntityContext::Attribute) {
            Some((entity, width)) => {
                let mut newline = false;
                entity.for_each(|c| {
                    newline |= matches!(c, '\n' | '\r');
                    decoded.push(c);
                });
                if newline {
                    return None;
                }
                rest = tail.get(width..)?;
            }
            None => {
                decoded.push('&');
                rest = tail.get(1..)?;
            }
        }
    }
    decoded.push_str(rest);
    Some(decoded)
}

struct LiteralSpans {
    spans: Vec<(u32, u32)>,
    whole_templates: bool,
}

impl<'a> Visit<'a> for LiteralSpans {
    fn visit_template_literal(&mut self, it: &TemplateLiteral<'a>) {
        if self.whole_templates {
            self.spans
                .push((it.span.start, it.span.end.saturating_sub(1)));
        } else {
            oxc_ast_visit::walk::walk_template_literal(self, it);
        }
    }

    fn visit_template_element(&mut self, it: &TemplateElement<'a>) {
        self.spans.push((it.span.start, it.span.end));
    }

    fn visit_string_literal(&mut self, it: &StringLiteral<'a>) {
        self.spans
            .push((it.span.start, it.span.end.saturating_sub(1)));
    }
}

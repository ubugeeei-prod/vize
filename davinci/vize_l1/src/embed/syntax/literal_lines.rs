//! Which lines of an embedded JS/TS value start inside a literal.
//!
//! A line that starts inside a string (a line continuation) or a template
//! literal belongs to the runtime value: re-indenting it changes the value. A
//! printer that re-indents the lines of a multi-line value asks this provider
//! which lines it must keep verbatim. Parsed spans decide that exactly, so a
//! backtick inside a string or comment is never taken for a literal.
//!
//! The value is read as the expression it starts with, under the compiler
//! profile's TypeScript module parse. When it does not start with one and
//! statements are allowed, it is read as a whole Program instead. Syntax that
//! is not clean under either reading has no answer; the caller keeps its
//! lexical fallback.

use alloc::vec::Vec;
use oxc_allocator::Allocator;
use oxc_ast::ast::{StringLiteral, TemplateElement, TemplateLiteral};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;

/// How a printer reads the literals of a value.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LiteralLineOptions {
    /// The byte that ends a line: `\n`, or `\r` for lone-CR text.
    pub newline: u8,
    /// Own a whole template literal, substitutions included, rather than only
    /// its quasis. Directive printers keep the literal as one piece; text
    /// interpolations re-indent the code inside `${…}`.
    pub whole_templates: bool,
    /// Read a value that is not an expression as handler statements.
    pub statements: bool,
}

/// `lines[n]` is whether line `n` of `source` starts inside a string or
/// template literal; line 0 never does. `None` when `source` has no line break
/// or does not parse cleanly.
#[must_use]
#[inline]
pub fn literal_line_starts(source: &str, options: LiteralLineOptions) -> Option<Vec<bool>> {
    if !source.as_bytes().contains(&options.newline) {
        return None;
    }
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let mut spans = LiteralSpans {
        spans: Vec::new(),
        whole_templates: options.whole_templates,
    };
    match Parser::new(&allocator, source, source_type).parse_expression() {
        Ok(expression) => spans.visit_expression(&expression),
        Err(_) if options.statements => {
            let parsed = Parser::new(&allocator, source, source_type).parse();
            if !parsed.diagnostics.is_empty() || parsed.panicked {
                return None;
            }
            spans.visit_program(&parsed.program);
        }
        Err(_) => return None,
    }
    let mut lines = alloc::vec![false];
    for (index, byte) in source.bytes().enumerate() {
        if byte == options.newline {
            let start = u32::try_from(index + 1).ok()?;
            lines.push(
                spans
                    .spans
                    .iter()
                    .any(|&(span_start, span_end)| span_start < start && start <= span_end),
            );
        }
    }
    Some(lines)
}

struct LiteralSpans {
    spans: Vec<(u32, u32)>,
    whole_templates: bool,
}

impl<'a> Visit<'a> for LiteralSpans {
    #[inline]
    fn visit_template_literal(&mut self, it: &TemplateLiteral<'a>) {
        if self.whole_templates {
            self.spans
                .push((it.span.start, it.span.end.saturating_sub(1)));
        } else {
            oxc_ast_visit::walk::walk_template_literal(self, it);
        }
    }

    #[inline]
    fn visit_template_element(&mut self, it: &TemplateElement<'a>) {
        self.spans.push((it.span.start, it.span.end));
    }

    #[inline]
    fn visit_string_literal(&mut self, it: &StringLiteral<'a>) {
        self.spans
            .push((it.span.start, it.span.end.saturating_sub(1)));
    }
}

#[cfg(test)]
mod tests;

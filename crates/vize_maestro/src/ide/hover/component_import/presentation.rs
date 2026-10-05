//! Wrap only long generated component type fragments, never the source program.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "hover Markdown uses std String payloads"
)]

use std::borrow::Cow;

pub(super) fn field(name: &str, source: &str) -> String {
    let value = type_fragment(source, name.len() + 5);
    let space = if value.starts_with('\n') { "" } else { " " };
    format!("  {name}:{space}{value};")
}

fn type_fragment(source: &str, prefix_width: usize) -> Cow<'_, str> {
    // The usual short contract keeps its exact bytes without a parser or allocation.
    if source.len().saturating_add(prefix_width) <= 80 {
        return Cow::Borrowed(source);
    }
    #[cfg(feature = "glyph")]
    if let Some(value) = format_long_type(source) {
        return Cow::Owned(value);
    }
    // Feature-minimal builds and unsupported fragments preserve the entire type.
    Cow::Borrowed(source)
}

#[cfg(feature = "glyph")]
#[cold]
fn format_long_type(source: &str) -> Option<String> {
    use oxc_ast::ast::{Statement, StringLiteral, TSTemplateLiteralType};
    use oxc_ast_visit::Visit;
    use oxc_span::GetSpan;

    let input = format!("type __VizeHover = {source};");
    let options = vize_glyph::FormatOptions {
        print_width: 78,
        quote_props: vize_glyph::QuoteProps::Preserve,
        ..Default::default()
    };
    let formatted = vize_glyph::format_script(&input, &options).ok()?;
    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, &formatted, oxc_span::SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() || parsed.program.body.len() != 1 {
        return None;
    }
    let Statement::TSTypeAliasDeclaration(alias) = parsed.program.body.first()? else {
        return None;
    };
    if alias.id.name != "__VizeHover" {
        return None;
    }
    let span = alias.type_annotation.span();
    let value = formatted.get(span.start as usize..span.end as usize)?;
    #[derive(Default)]
    struct QuotedTypes(Vec<oxc_span::Span>);
    impl<'a> Visit<'a> for QuotedTypes {
        fn visit_string_literal(&mut self, literal: &StringLiteral<'a>) {
            self.0.push(literal.span);
        }
        fn visit_ts_template_literal_type(&mut self, literal: &TSTemplateLiteralType<'a>) {
            self.0.push(literal.span);
        }
    }
    let mut quoted = QuotedTypes::default();
    quoted.visit_ts_type(&alias.type_annotation);
    quoted.0.sort_unstable_by_key(|span| span.start);
    let before = formatted.get(..span.start as usize)?;
    let preceding = before.rsplit('\n').next()?;
    let mut output = String::new();
    // Restore the formatter's RHS line break/indent and optional leading union/intersection
    // separator outside the AST span. No delimiter inside the type is searched.
    if before.contains('\n')
        && preceding
            .bytes()
            .all(|byte| matches!(byte, b' ' | b'|' | b'&'))
    {
        output.push_str("\n  ");
        output.push_str(preceding);
    }
    // Raw newlines inside literal types are part of the type's value. Only
    // syntax whitespace gains the enclosing field's two-space indentation.
    append_indented(&mut output, value, span.start, &quoted.0)?;
    Some(output)
}

#[cfg(feature = "glyph")]
fn append_indented(
    output: &mut String,
    value: &str,
    offset: u32,
    quoted: &[oxc_span::Span],
) -> Option<()> {
    let mut start = 0;
    let mut literal = 0;
    for (index, byte) in value.bytes().enumerate() {
        let position = offset as usize + index;
        while quoted
            .get(literal)
            .is_some_and(|span| span.end as usize <= position)
        {
            literal += 1;
        }
        let inside = quoted
            .get(literal)
            .is_some_and(|span| span.start as usize <= position);
        if byte == b'\n' && !inside {
            output.push_str(value.get(start..=index)?);
            output.push_str("  ");
            start = index + 1;
        }
    }
    output.push_str(value.get(start..)?);
    Some(())
}

#[cfg(test)]
mod tests {
    use super::{field, type_fragment};
    use std::borrow::Cow;

    #[test]
    fn short_contract_is_borrowed_and_byte_identical() {
        let source = "{ label: string; count?: number }";
        assert!(matches!(type_fragment(source, 10), Cow::Borrowed(value) if value == source));
        assert_eq!(
            field("props", source),
            "  props: { label: string; count?: number };"
        );
    }

    #[test]
    fn unsupported_long_type_is_preserved_without_truncation() {
        let source = format!("{{ {}", "invalid type ".repeat(12));
        assert_eq!(field("props", &source), format!("  props: {source};"));
    }

    #[cfg(feature = "glyph")]
    #[test]
    fn wrapping_a_union_preserves_every_quoted_punctuation_token() {
        let source = "\"first;=value\" | \"second,<value>\" | \"third{}[]\" | \"fourth(value)\" | \"fifth\\\"quoted\"";
        let value = field("props", source);
        assert_eq!(
            value,
            "  props:\n    | \"first;=value\"\n    | \"second,<value>\"\n    | \"third{}[]\"\n    | \"fourth(value)\"\n    | 'fifth\"quoted';"
        );
    }

    #[cfg(feature = "glyph")]
    #[test]
    fn structural_indentation_preserves_raw_lf_crlf_and_continued_literals() {
        for literal in [
            "`first\nsecond`",
            "`first\r\nsecond`",
            "`outer${`inner\nraw`}`",
            "\"first\\\ncontinued\"",
            "\"first\\\r\ncontinued\"",
        ] {
            let source = format!("{{\n  value: {literal};\n  ready: boolean;\n}}");
            let start = source.find(literal).expect("authored literal span") as u32;
            let span = oxc_span::Span::new(start, start + literal.len() as u32);
            let mut actual = String::new();
            super::append_indented(&mut actual, &source, 0, &[span]).expect("complete fragment");
            assert_eq!(
                actual,
                format!("{{\n    value: {literal};\n    ready: boolean;\n  }}")
            );
        }
    }

    #[cfg(not(feature = "glyph"))]
    #[test]
    fn no_glyph_preserves_complete_long_contract() {
        let source = "Record<string, { selected?: ReadonlyArray<{ id: string; value: number; ready: boolean }> }>";
        assert!(matches!(type_fragment(source, 10), Cow::Borrowed(value) if value == source));
    }
}

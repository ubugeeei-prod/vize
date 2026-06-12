//! Parsing `.jsx`/`.tsx` source into an OXC program.
//!
//! This is a thin wrapper over `oxc_parser` that selects the right
//! [`SourceType`](oxc_span::SourceType) for the [`JsxLang`] and surfaces parse
//! errors as Vize [`JsxDiagnostic`]s with byte ranges.

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_parser::Parser;

use vize_carton::ToCompactString;

use crate::diagnostics::JsxDiagnostic;
use crate::lang::JsxLang;

/// The result of parsing a JSX/TSX module.
///
/// The [`Program`] borrows the supplied [`Allocator`], so the allocator must
/// outlive the parsed module. Callers typically lower the program into Vize IR
/// (which copies out owned strings) before dropping the allocator.
pub struct ParsedModule<'a> {
    /// The parsed program. Empty body if parsing panicked.
    pub program: Program<'a>,
    /// Parse diagnostics, already mapped to Vize byte ranges.
    pub diagnostics: Vec<JsxDiagnostic>,
    /// Whether the parser bailed out unrecoverably.
    pub panicked: bool,
}

impl<'a> ParsedModule<'a> {
    /// Whether parsing produced any error-severity diagnostics.
    pub fn has_errors(&self) -> bool {
        self.panicked || self.diagnostics.iter().any(JsxDiagnostic::is_error)
    }
}

/// Parse `source` as a JSX/TSX module using `lang` to select the dialect.
pub fn parse_module<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    lang: JsxLang,
) -> ParsedModule<'a> {
    let ret = Parser::new(allocator, source, lang.source_type()).parse();
    let source_len = source.len();
    let diagnostics = ret
        .errors
        .iter()
        .map(|error| oxc_error_to_diagnostic(error, source_len))
        .collect();
    ParsedModule {
        program: ret.program,
        diagnostics,
        panicked: ret.panicked,
    }
}

/// Convert an OXC diagnostic into a Vize [`JsxDiagnostic`] with a byte range.
///
/// Mirrors the primary-label extraction used by the maestro diagnostics
/// collectors so JSX parse errors point at the same offsets the editor uses.
fn oxc_error_to_diagnostic(
    error: &oxc_diagnostics::OxcDiagnostic,
    source_len: usize,
) -> JsxDiagnostic {
    let (start, end) = error
        .labels
        .as_ref()
        .and_then(|labels| labels.iter().find(|label| label.primary()))
        .or_else(|| error.labels.as_ref().and_then(|labels| labels.first()))
        .map(|label| {
            let start = label.offset().min(source_len);
            let end = start
                .saturating_add(label.len().max(1))
                .min(source_len)
                .max(start + 1);
            (start as u32, end as u32)
        })
        .unwrap_or((0, source_len.max(1) as u32));
    JsxDiagnostic::error(error.message.to_compact_string(), start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_jsx() {
        let allocator = Allocator::default();
        let parsed = parse_module(&allocator, "const a = <div/>;", JsxLang::Jsx);
        assert!(!parsed.panicked);
        assert!(!parsed.has_errors());
        assert_eq!(parsed.program.body.len(), 1);
    }

    #[test]
    fn parses_tsx_with_type_annotations() {
        let allocator = Allocator::default();
        let src = "const a = (x: number): JSX.Element => <p>{x}</p>;";
        let parsed = parse_module(&allocator, src, JsxLang::Tsx);
        assert!(!parsed.has_errors(), "{:?}", parsed.diagnostics);
    }

    #[test]
    fn reports_syntax_error_with_range() {
        let allocator = Allocator::default();
        let parsed = parse_module(&allocator, "const a = <div>;", JsxLang::Jsx);
        assert!(parsed.has_errors());
        let diag = &parsed.diagnostics[0];
        assert!(diag.end > diag.start);
        assert!(diag.end <= "const a = <div>;".len() as u32);
    }

    #[test]
    fn type_annotation_is_error_in_plain_jsx() {
        // `.jsx` should not accept TS type annotations.
        let allocator = Allocator::default();
        let parsed = parse_module(&allocator, "const a = (x: number) => x;", JsxLang::Jsx);
        assert!(parsed.has_errors());
    }
}

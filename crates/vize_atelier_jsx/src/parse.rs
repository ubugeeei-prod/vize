//! Parsing `.jsx`/`.tsx` source into an OXC program.
//!
//! This is a thin wrapper over `oxc_parser` that selects the right
//! [`SourceType`](oxc_span::SourceType) for the [`JsxLang`] and surfaces parse
//! errors as Vize [`JsxDiagnostic`]s with byte ranges.

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_parser::Parser;

use std::borrow::Cow;

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

/// Return parser input that keeps Vue's TSX-only attribute spellings parseable.
///
/// OXC follows TypeScript's JSX grammar, where `ns:local` accepts an identifier
/// as the local part. Vue users also write update listeners such as
/// `onUpdate:current-step-index={...}`. The hyphenated local name is valid Vue
/// JSX, but it trips TSX parsing before Vize can lower it. Replace only those
/// hyphens with `_` while preserving byte length; lowering still reads names
/// from the original source spans, so the emitted IR keeps the authored name.
pub fn prepare_source_for_parse(source: &str, _lang: JsxLang) -> Cow<'_, str> {
    let bytes = source.as_bytes();
    if !bytes.contains(&b':') || !bytes.contains(&b'-') {
        return Cow::Borrowed(source);
    }

    let mut output = None;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' | b'"' | b'`' => index = skip_quoted(bytes, index, bytes[index]),
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = skip_line_comment(bytes, index + 2);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = skip_block_comment(bytes, index + 2);
            }
            b'<' if let Some(name_start) = jsx_tag_name_start(bytes, index) => {
                index = sanitize_jsx_opening_tag(bytes, name_start, &mut output);
            }
            _ => index += 1,
        }
    }

    match output {
        Some(bytes) => Cow::Owned(
            std::str::from_utf8(bytes.as_slice())
                .expect("source stays utf-8")
                .to_owned(),
        ),
        None => Cow::Borrowed(source),
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

fn jsx_tag_name_start(bytes: &[u8], lt: usize) -> Option<usize> {
    let mut index = lt + 1;
    if bytes.get(index) == Some(&b'/') {
        index += 1;
    }
    bytes
        .get(index)
        .is_some_and(|byte| is_jsx_ident_start(*byte))
        .then_some(index)
}

fn sanitize_jsx_opening_tag(
    bytes: &[u8],
    name_start: usize,
    output: &mut Option<Vec<u8>>,
) -> usize {
    let mut index = skip_jsx_tag_name(bytes, name_start);
    let mut braces = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if braces > 0 {
            index = skip_js_expression_byte(bytes, index, &mut braces);
            continue;
        }
        match byte {
            b'>' => return index + 1,
            b'\'' | b'"' => index = skip_quoted(bytes, index, byte),
            b'{' => {
                braces = 1;
                index += 1;
            }
            _ if is_jsx_ident_start(byte) => {
                index = sanitize_possible_namespaced_attr(bytes, index, output);
            }
            _ => index += 1,
        }
    }
    bytes.len()
}

fn skip_js_expression_byte(bytes: &[u8], index: usize, braces: &mut usize) -> usize {
    match bytes[index] {
        b'{' => {
            *braces += 1;
            index + 1
        }
        b'}' => {
            *braces = braces.saturating_sub(1);
            index + 1
        }
        b'\'' | b'"' => skip_quoted(bytes, index, bytes[index]),
        b'`' => skip_quoted(bytes, index, b'`'),
        b'/' if bytes.get(index + 1) == Some(&b'/') => skip_line_comment(bytes, index + 2),
        b'/' if bytes.get(index + 1) == Some(&b'*') => skip_block_comment(bytes, index + 2),
        _ => index + 1,
    }
}

fn sanitize_possible_namespaced_attr(
    bytes: &[u8],
    start: usize,
    output: &mut Option<Vec<u8>>,
) -> usize {
    let namespace_end = skip_jsx_attr_name_part(bytes, start);
    if bytes.get(namespace_end) != Some(&b':') {
        return namespace_end;
    }

    let local_start = namespace_end + 1;
    let local_end = skip_jsx_attr_name_part(bytes, local_start);
    for index in local_start..local_end {
        if bytes[index] == b'-' {
            output.get_or_insert_with(|| bytes.to_vec()).as_mut_slice()[index] = b'_';
        }
    }
    local_end
}

fn skip_jsx_tag_name(bytes: &[u8], mut index: usize) -> usize {
    while bytes
        .get(index)
        .is_some_and(|byte| is_jsx_attr_name_part(*byte) || matches!(byte, b':' | b'.'))
    {
        index += 1;
    }
    index
}

fn skip_jsx_attr_name_part(bytes: &[u8], mut index: usize) -> usize {
    while bytes
        .get(index)
        .is_some_and(|byte| is_jsx_attr_name_part(*byte))
    {
        index += 1;
    }
    index
}

fn is_jsx_ident_start(byte: u8) -> bool {
    byte == b'_' || byte == b'$' || byte.is_ascii_alphabetic()
}

fn is_jsx_attr_name_part(byte: u8) -> bool {
    is_jsx_ident_start(byte) || byte.is_ascii_digit() || byte == b'-'
}

fn skip_quoted(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            byte if byte == quote => return index + 1,
            _ => index += 1,
        }
    }
    bytes.len()
}

fn skip_line_comment(bytes: &[u8], mut index: usize) -> usize {
    while bytes.get(index).is_some_and(|byte| *byte != b'\n') {
        index += 1;
    }
    index
}

fn skip_block_comment(bytes: &[u8], mut index: usize) -> usize {
    while index + 1 < bytes.len() {
        if bytes[index] == b'*' && bytes[index + 1] == b'/' {
            return index + 2;
        }
        index += 1;
    }
    bytes.len()
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

    #[test]
    fn prepare_source_sanitizes_kebab_namespaced_attr_local_names() {
        let src = "const a = <Comp onUpdate:current-step-index={h} />;";
        let prepared = prepare_source_for_parse(src, JsxLang::Tsx);
        assert_eq!(prepared.len(), src.len());
        assert_eq!(
            prepared.as_ref(),
            "const a = <Comp onUpdate:current_step_index={h} />;"
        );
    }

    #[test]
    fn prepare_source_ignores_hyphens_inside_jsx_expressions() {
        let src = "const a = <Comp value={current-step-index} />;";
        let prepared = prepare_source_for_parse(src, JsxLang::Tsx);
        assert!(matches!(prepared, Cow::Borrowed(_)));
    }

    #[test]
    fn prepare_source_ignores_jsx_like_text_inside_strings() {
        let src = r#"const text = "<Comp onUpdate:current-step-index={h} />";"#;
        let prepared = prepare_source_for_parse(src, JsxLang::Tsx);
        assert!(matches!(prepared, Cow::Borrowed(_)));
    }
}

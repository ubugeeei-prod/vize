//! Emit a genuine source-owned plain CSS family; no legacy compiler helper.

use vize_l0::Span;
use vize_l1::container::vue::{AdmittedDescriptor, StyleView};
use vize_l4::write::{EmitDocument, LinkSink};

use super::NativeSfcCompileError;

/// Private evidence that this original style needs no unavailable transform.
struct PlainStyle<'o, 'a> {
    original: StyleView<'o, 'a>,
}

impl<'o, 'a> PlainStyle<'o, 'a> {
    fn observe(original: StyleView<'o, 'a>) -> Result<Self, NativeSfcCompileError> {
        if original
            .attrs()
            .iter()
            .any(|attr| attr.name != "lang" || attr.value != Some("css"))
        {
            return Err(NativeSfcCompileError::StyleCompilationUnavailable {
                container_index: original.container_index(),
                span: original.block().span(),
            });
        }
        if let Some(span) = unproven_bind(original.block().source(), original.block().start()) {
            return Err(NativeSfcCompileError::StyleBindSyntaxUnproven {
                container_index: original.container_index(),
                span,
            });
        }
        Ok(Self { original })
    }

    fn append(&self, output: &mut EmitDocument, trim: bool) {
        let block = self.original.block();
        let raw = block.source();
        let (text, prefix) = if trim {
            (raw.trim(), raw.len() - raw.trim_start().len())
        } else {
            (raw, 0)
        };
        // Match the complete SFC CSS join, including an empty trailing block.
        if !output.is_empty() {
            output.push_str("\n");
        }
        let mut at = block.start() + prefix as u32;
        for line in text.split_inclusive('\n') {
            let end = at + line.len() as u32;
            output.push_linked(line, Span::new(at, end));
            at = end;
        }
    }
}

pub(super) fn emit<L: LinkSink>(
    descriptor: AdmittedDescriptor<'_, '_>,
    trim: bool,
) -> Result<Option<EmitDocument>, NativeSfcCompileError> {
    let mut output = EmitDocument::new(L::RECORDING);
    for original in descriptor.styles() {
        PlainStyle::observe(original)?.append(&mut output, trim);
    }
    Ok((!output.is_empty()).then_some(output))
}

/// This conservative absence proof is not a CSS parser. Escapes, raw binding
/// spelling even in strings/comments, comment-bridged spelling and unfinished
/// comments are unproven. No source bytes are decoded, copied or normalized.
fn unproven_bind(source: &str, start: u32) -> Option<Span> {
    let bytes = source.as_bytes();
    let marker = b"v-bind";
    let (mut at, mut matched, mut token_start) = (0, 0, 0);
    let mut comment_start = None;
    while let Some(&byte) = bytes.get(at) {
        if byte == b'\\' {
            return Some(Span::new(start + at as u32, start + at as u32 + 1));
        }
        if bytes
            .get(at..at + marker.len())
            .is_some_and(|raw| raw.eq_ignore_ascii_case(marker))
        {
            return Some(Span::new(
                start + at as u32,
                start + at as u32 + marker.len() as u32,
            ));
        }
        if comment_start.is_some() {
            if byte == b'*' && bytes.get(at + 1) == Some(&b'/') {
                comment_start = None;
                at += 2;
            } else {
                at += 1;
            }
            continue;
        }
        if byte == b'/' && bytes.get(at + 1) == Some(&b'*') {
            comment_start = Some(at);
            at += 2;
            continue;
        }
        if marker
            .get(matched)
            .is_some_and(|expected| byte.eq_ignore_ascii_case(expected))
        {
            if matched == 0 {
                token_start = at;
            }
            matched += 1;
            if matched == marker.len() {
                return Some(Span::new(start + token_start as u32, start + at as u32 + 1));
            }
        } else if byte.eq_ignore_ascii_case(&b'v') {
            matched = 1;
            token_start = at;
        } else {
            matched = 0;
        }
        at += 1;
    }
    comment_start.map(|at| Span::new(start + at as u32, start + source.len() as u32))
}

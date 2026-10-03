//! Original Vue 1 escaped-text expressions, not generated runtime strings.

pub(super) mod sink;

use crate::embed::source::{DecodeSegmentKind, prepare_text_value};
use crate::embed::syntax::{NativeSyntax, parse_once};
use crate::embed::{Embed, EmbedSource, Grammar, Lang, Shape};
use vize_l0::{Allocator, SourceBlock, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextBoundary {
    pub span: Span,
    pub kind: TextBoundaryKind,
}

/// Conservative limits of this literal-delimiter expression-only provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextBoundaryKind {
    RawInterpolation,
    IncompleteDelimiter,
    EmptyInterpolation,
    HistoricalLineSeparator,
    OneTimeInterpolation,
    /// Every pipe is deferred, including `||` and pipes inside literals. This
    /// does not substitute Vue 2's filter scanner for Vue 1's distinct grammar.
    PipeSyntax,
    EncodedDelimiter,
    SourcePreparation,
}

/// An immutable observation created only at the original lexer callback.
/// Raw AST inspection carries no original CST occurrence authority.
#[derive(Debug)]
pub struct TextBinding<'a> {
    block: SourceBlock<'a>,
    span: Span,
    content_span: Span,
    raw_content: &'a str,
    cst_start: u32,
    source: Option<EmbedSource<'a>>,
    syntax: Option<NativeSyntax<'a>>,
    boundary: Option<TextBoundaryKind>,
}

impl<'a> TextBinding<'a> {
    pub const fn span(&self) -> Span {
        self.span
    }
    pub const fn content_span(&self) -> Span {
        self.content_span
    }
    pub const fn raw_content(&self) -> &'a str {
        self.raw_content
    }
    pub const fn source(&self) -> Option<EmbedSource<'a>> {
        self.source
    }
    pub fn syntax(&self) -> Option<&NativeSyntax<'a>> {
        self.syntax.as_ref()
    }
    pub const fn boundary(&self) -> Option<TextBoundaryKind> {
        self.boundary
    }
    pub(super) const fn cst_start(&self) -> u32 {
        self.cst_start
    }
    pub(super) fn matches(&self, block: SourceBlock<'a>, raw: &str) -> bool {
        self.block == block
            && self.raw_content.as_ptr() == raw.as_ptr()
            && self.raw_content.len() == raw.len()
            && block.span_of(raw) == Some(self.content_span)
    }
}

pub(super) fn observe<'a>(
    allocator: &'a Allocator,
    block: SourceBlock<'a>,
    start: usize,
    end: usize,
    raw: bool,
) -> TextBinding<'a> {
    // The existing lexer supplies checked UTF-8 source boundaries.
    let raw_content = &block.source()[start..end];
    let content_span = block.span_of(raw_content).expect("original callback bytes");
    let full_raw = raw && block.source().get(end..end + 3) == Some("}}}");
    let span = Span::new(
        content_span.start - if raw { 3 } else { 2 },
        (content_span.end + if full_raw { 3 } else { 2 }).min(block.end()),
    );
    let mut binding = TextBinding {
        block,
        span,
        content_span,
        raw_content,
        // The unchanged Vue 1 policy's raw recovery retains the third opening
        // brace in its escaped CST token. It can only confer a typed refusal.
        cst_start: content_span.start - u32::from(raw && (!full_raw || raw_content.is_empty())),
        source: None,
        syntax: None,
        boundary: None,
    };
    let prepared = match prepare_text_value(allocator, block.root_source(), content_span) {
        Ok(source) => source,
        Err(_) => {
            binding.boundary = Some(TextBoundaryKind::SourcePreparation);
            return binding;
        }
    };
    binding.source = Some(prepared);
    let text = prepared.text();
    let boundary = if raw {
        Some(TextBoundaryKind::RawInterpolation)
    } else if block.source().get(end..end + 2) != Some("}}") {
        Some(TextBoundaryKind::IncompleteDelimiter)
    } else if text.contains(['\r', '\u{2028}', '\u{2029}']) {
        // Historical `( . | \n )` excludes every CR, even in a CRLF pair.
        Some(TextBoundaryKind::HistoricalLineSeparator)
    } else if encoded_brace(prepared) {
        Some(TextBoundaryKind::EncodedDelimiter)
    } else if text.starts_with('*') {
        // Vue 1 tests the first character before trimming, unlike a scanner
        // that would silently turn leading-space `*` into once-only syntax.
        Some(TextBoundaryKind::OneTimeInterpolation)
    } else if text.contains('|') {
        Some(TextBoundaryKind::PipeSyntax)
    } else {
        None
    };
    if let Some(kind) = boundary {
        binding.boundary = Some(kind);
        return binding;
    }
    let Some(source) = trim(allocator, prepared) else {
        binding.boundary = Some(TextBoundaryKind::SourcePreparation);
        return binding;
    };
    binding.source = Some(source);
    if source.text().is_empty() {
        binding.boundary = Some(TextBoundaryKind::EmptyInterpolation);
    } else {
        binding.syntax = Some(parse_once(
            allocator,
            Embed {
                source,
                grammar: Grammar {
                    shape: Shape::Expr,
                    lang: Lang::Js,
                },
            },
        ));
    }
    binding
}

fn encoded_brace(source: EmbedSource<'_>) -> bool {
    source.decode_map().is_some_and(|map| {
        map.segments().iter().any(|segment| {
            let span = segment.decoded();
            segment.kind() == DecodeSegmentKind::Entity
                && source.text()[span.start as usize..span.end as usize].contains(['{', '}'])
        })
    })
}

fn trim<'a>(allocator: &'a Allocator, source: EmbedSource<'a>) -> Option<EmbedSource<'a>> {
    // ECMAScript String.trim on the once-decoded complete callback window.
    let whitespace = |ch: char| {
        matches!(ch, '\u{0009}'..='\u{000d}' | ' ' |
        '\u{00a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' |
        '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
    };
    let text = source.text();
    let trimmed = text.trim_matches(whitespace);
    let start = trimmed.as_ptr() as usize - text.as_ptr() as usize;
    source
        .slice_in(
            allocator,
            Span::new(start as u32, (start + trimmed.len()) as u32),
        )
        .ok()
}

//! Vue 2.7.16 text filters, measured against their original decoded source.
//!
//! The historical scanner selects pieces; the existing language provider parses
//! each original expression exactly once. Generated `_f`/`_s` text is never input.

mod scan;
use scan::scan;

use alloc::vec::Vec;
use vize_l0::{Allocator, SourceBlock, Span};

use crate::embed::syntax::{NativeSyntax, parse_once};
use crate::embed::{Embed, EmbedSource, Grammar, Lang, Shape, prepare_vue_interpolation_in};

/// A retained authored refusal; it does not discard later bindings or diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextBoundary {
    pub span: Span,
    pub kind: TextBoundaryKind,
}

/// Explicit limits of this default-delimiter text family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextBoundaryKind {
    EmptyInterpolation,
    HistoricalLineSeparator,
    IncompleteDelimiter,
    SourcePreparation,
    UnbalancedFilterSyntax,
    CommentSyntax,
    RegexCharacterClass,
    UnsupportedFilterName,
    UnsupportedArgumentList,
    /// Vue 2 decodes text before delimiter recognition. This bounded provider
    /// handles literal authored delimiters; entity-produced brace framing waits.
    EncodedDelimiter,
}

/// One filter registry name and the actual once-parsed original arguments.
#[derive(Debug)]
pub struct FilterInvocation<'a> {
    span: Span,
    name: EmbedSource<'a>,
    arguments: Vec<NativeSyntax<'a>>,
}

impl<'a> FilterInvocation<'a> {
    pub const fn span(&self) -> Span {
        self.span
    }
    pub const fn name(&self) -> EmbedSource<'a> {
        self.name
    }
    pub fn arguments(&self) -> &[NativeSyntax<'a>] {
        &self.arguments
    }
}

/// Ordered filter application syntax. No registry resolution or lowering is implied.
#[derive(Debug)]
pub struct FilterChain<'a> {
    base: NativeSyntax<'a>,
    filters: Vec<FilterInvocation<'a>>,
}

impl<'a> FilterChain<'a> {
    pub fn base(&self) -> &NativeSyntax<'a> {
        &self.base
    }
    pub fn filters(&self) -> &[FilterInvocation<'a>] {
        &self.filters
    }
    fn is_admitted(&self) -> bool {
        self.base.hole().is_none()
            && self.filters.iter().all(|filter| {
                filter
                    .arguments
                    .iter()
                    .all(|argument| argument.hole().is_none())
            })
    }
}

/// An authentic callback observation, including unadmitted original syntax.
#[derive(Debug)]
pub struct TextBinding<'a> {
    span: Span,
    source: EmbedSource<'a>,
    chain: Option<FilterChain<'a>>,
    boundaries: Vec<TextBoundary>,
}

impl<'a> TextBinding<'a> {
    pub const fn span(&self) -> Span {
        self.span
    }
    pub const fn source(&self) -> EmbedSource<'a> {
        self.source
    }
    /// Retained pieces, including local language holes or a partial chain before
    /// a later typed boundary. Only `admitted()` confers complete-chain syntax.
    pub fn chain(&self) -> Option<&FilterChain<'a>> {
        self.chain.as_ref()
    }
    pub fn boundaries(&self) -> &[TextBoundary] {
        &self.boundaries
    }
    pub fn admitted(&self) -> Option<&FilterChain<'a>> {
        self.chain
            .as_ref()
            .filter(|chain| self.boundaries.is_empty() && chain.is_admitted())
    }
}

pub(super) fn observe<'a>(
    allocator: &'a Allocator,
    block: SourceBlock<'a>,
    start: usize,
    end: usize,
) -> Option<TextBinding<'a>> {
    let content = block.source().get(start..end)?;
    let content_span = block.span_of(content)?;
    let span = Span::new(
        content_span.start.saturating_sub(2),
        (content_span.end + 2).min(block.end()),
    );
    let prepared =
        prepare_vue_interpolation_in(allocator, block.root_source(), content_span).ok()?;
    let source = trim(allocator, prepared)?;
    let mut binding = TextBinding {
        span,
        source,
        chain: None,
        boundaries: Vec::new(),
    };
    let boundary = if block.source().get(end..end + 2) != Some("}}") {
        Some(TextBoundaryKind::IncompleteDelimiter)
    } else if source.text().is_empty() {
        Some(TextBoundaryKind::EmptyInterpolation)
    } else if historical_separator(content) {
        Some(TextBoundaryKind::HistoricalLineSeparator)
    } else {
        None
    };
    if let Some(kind) = boundary {
        binding.boundaries.push(TextBoundary { span, kind });
        return Some(binding);
    }
    match chain(allocator, source) {
        Ok((chain, boundary)) => {
            binding.chain = Some(chain);
            if let Some(kind) = boundary {
                binding.boundaries.push(TextBoundary {
                    span: source.span(),
                    kind,
                });
            }
        }
        Err(kind) => binding.boundaries.push(TextBoundary {
            span: source.span(),
            kind,
        }),
    }
    Some(binding)
}

fn historical_separator(text: &str) -> bool {
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{2028}' || ch == '\u{2029}' || (ch == '\r' && chars.peek() != Some(&'\n')) {
            return true;
        }
    }
    false
}

fn trim<'a>(allocator: &'a Allocator, source: EmbedSource<'a>) -> Option<EmbedSource<'a>> {
    // ECMAScript String.trim, after HTML text entity decoding, as in Vue 2.
    let whitespace = |ch: char| {
        matches!(ch, '\u{0009}'..='\u{000d}' | ' ' | '\u{00a0}' |
        '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' |
        '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
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

fn piece<'a>(
    allocator: &'a Allocator,
    source: EmbedSource<'a>,
    start: usize,
    end: usize,
) -> Result<EmbedSource<'a>, TextBoundaryKind> {
    let slice = source
        .slice_in(allocator, Span::new(start as u32, end as u32))
        .map_err(|_| TextBoundaryKind::SourcePreparation)?;
    trim(allocator, slice).ok_or(TextBoundaryKind::SourcePreparation)
}

fn expression<'a>(allocator: &'a Allocator, source: EmbedSource<'a>) -> NativeSyntax<'a> {
    parse_once(
        allocator,
        Embed {
            source,
            grammar: Grammar {
                shape: Shape::Expr,
                lang: Lang::Js,
            },
        },
    )
}

fn chain<'a>(
    allocator: &'a Allocator,
    source: EmbedSource<'a>,
) -> Result<(FilterChain<'a>, Option<TextBoundaryKind>), TextBoundaryKind> {
    let separators = scan(source.text(), b'|')?;
    let base_end = separators.first().copied().unwrap_or(source.text().len());
    let base = expression(allocator, piece(allocator, source, 0, base_end)?);
    let mut chain = FilterChain {
        base,
        filters: Vec::new(),
    };
    for (index, separator) in separators.iter().enumerate() {
        let end = separators
            .get(index + 1)
            .copied()
            .unwrap_or(source.text().len());
        let filter = piece(allocator, source, separator + 1, end)
            .and_then(|filter| invocation(allocator, filter));
        match filter {
            Ok(filter) => chain.filters.push(filter),
            Err(kind) => return Ok((chain, Some(kind))),
        }
    }
    Ok((chain, None))
}

fn invocation<'a>(
    allocator: &'a Allocator,
    filter: EmbedSource<'a>,
) -> Result<FilterInvocation<'a>, TextBoundaryKind> {
    let text = filter.text();
    let open = text.find('(');
    let name_end = open.unwrap_or(text.len());
    let name = filter
        .slice_in(allocator, Span::new(0, name_end as u32))
        .map_err(|_| TextBoundaryKind::SourcePreparation)?;
    // Upstream uses this exact name as a JS string registry key. Refuse
    // spellings that its unescaped generated string cannot safely represent.
    if name.text().is_empty()
        || name
            .text()
            .chars()
            .any(|ch| matches!(ch, '"' | '\\' | '\r' | '\n' | '\u{2028}' | '\u{2029}'))
    {
        return Err(TextBoundaryKind::UnsupportedFilterName);
    }
    let mut arguments = Vec::new();
    if let Some(open) = open {
        if !text.ends_with(')') {
            return Err(TextBoundaryKind::UnsupportedArgumentList);
        }
        let args = filter
            .slice_in(
                allocator,
                Span::new((open + 1) as u32, (text.len() - 1) as u32),
            )
            .map_err(|_| TextBoundaryKind::SourcePreparation)?;
        if !args.text().is_empty() {
            let commas = scan(args.text(), b',')?;
            let mut start = 0;
            for end in commas
                .into_iter()
                .chain(core::iter::once(args.text().len()))
            {
                let argument = piece(allocator, args, start, end)?;
                if argument.text().is_empty() || argument.text().starts_with("...") {
                    return Err(TextBoundaryKind::UnsupportedArgumentList);
                }
                arguments.push(argument);
                start = end + 1;
            }
        }
    }
    Ok(FilterInvocation {
        span: filter.span(),
        name,
        // Validate the complete original list before starting language
        // parses, so a structural refusal cannot drop an observed AST.
        arguments: arguments
            .into_iter()
            .map(|source| expression(allocator, source))
            .collect(),
    })
}

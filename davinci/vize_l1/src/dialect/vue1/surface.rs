//! Native Vue 1 text syntax with an intrinsic, source-retaining carrier.
//!
//! Tag and attribute recovery shares the existing construction. This bounded
//! component profile does not claim the browser document profile, attribute
//! interpolation, filter/directive embeds, or runtime raw/one-time semantics.

use alloc::vec::Vec as OwnedVec;
use vize_l0::{Allocator, SourceBlock, SourceRoot, Span, Vec};

mod body;
use super::text::sink::TextSink;
use super::text::{TextBinding, TextBoundary};
pub use body::{TextChild, TextChildren, TextRefusal, TextView};

pub use vize_l0::SourceFrameError as SourceError;

use crate::dialect::LegacyVueVersion;
use crate::dialect::vue::surface::{SurfacePolicy, sink::VueSink};
use crate::event::Recorder;
use crate::markup::{Component, LexOptions, Lexer, Sink};
use crate::parse::{SurfaceError, construct};
use crate::surface::SurfaceTree;

/// A retained interpolation that this bounded provider does not admit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxBoundary {
    pub span: Span,
    pub kind: SyntaxBoundaryKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxBoundaryKind {
    /// The shared lexer callback cannot prove the complete unsafe delimiters.
    RawDelimiterRecovery,
    /// The historical regex does not recognize an empty interpolation.
    EmptyInterpolation,
    /// Historical JS dot excludes CR and Unicode line/paragraph separators.
    HistoricalLineSeparator,
    /// The authored leading `*` carries unfinished once-only semantics.
    OneTimeInterpolation,
}

/// Parser-owned Vue 1 source, distinct from the Vue 3 component carrier.
/// No conversion can confer Vue 3 factory or runtime admission.
///
/// ```compile_fail
/// use vize_l1::{dialect::vue1::surface, markup};
/// fn modern<'a>(parsed: surface::ComponentParse<'a>) -> markup::ComponentParse<'a> {
///     parsed.into()
/// }
/// ```
#[derive(Debug)]
pub struct ComponentParse<'a> {
    block: SourceBlock<'a>,
    tree: SurfaceTree<'a>,
    authored: Option<SurfaceTree<'a>>,
    errors: Vec<'a, SurfaceError>,
    unsupported: Vec<'a, SyntaxBoundary>,
    // These observations own ordinary parser diagnostics; arena storage would
    // skip their destructors. Source maps and actual ASTs still use the arena.
    bindings: OwnedVec<TextBinding<'a>>,
    text_boundaries: OwnedVec<TextBoundary>,
}

impl<'a> ComponentParse<'a> {
    pub const fn version(&self) -> LegacyVueVersion {
        LegacyVueVersion::V1
    }

    /// The authentic complete-file frame retained by this original parse.
    pub const fn block(&self) -> SourceBlock<'a> {
        self.block
    }

    pub fn tree(&self) -> &SurfaceTree<'a> {
        &self.tree
    }

    pub fn authored(&self) -> Option<&SurfaceTree<'a>> {
        self.authored.as_ref()
    }

    pub fn errors(&self) -> &[SurfaceError] {
        &self.errors
    }

    pub fn unsupported(&self) -> &[SyntaxBoundary] {
        &self.unsupported
    }

    /// Original callback observations, including syntax holes. Inspection alone
    /// does not join an observation to an original CST child.
    pub fn bindings(&self) -> &[TextBinding<'a>] {
        &self.bindings
    }

    pub fn text_boundaries(&self) -> &[TextBoundary] {
        &self.text_boundaries
    }
}

/// Construct native Vue 1 text syntax in the component markup profile.
/// Complete nonempty triple delimiters retain an unescaped lexical fact.
/// Raw/empty/one-time recoveries retain bytes and explicit admission boundaries.
pub fn parse_component<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> Result<ComponentParse<'a>, SourceError> {
    Ok(parse_component_block(
        allocator,
        SourceRoot::new(source)?.whole_block(),
    ))
}

pub fn parse_component_with_authored<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> Result<ComponentParse<'a>, SourceError> {
    Ok(parse_component_with_authored_block(
        allocator,
        SourceRoot::new(source)?.whole_block(),
    ))
}

/// Parse this original block once, retaining its complete authored root.
///
/// The carrier cannot outlive either its original root or syntax arena.
///
/// ```compile_fail
/// use vize_l0::{Allocator, SourceRoot};
/// use vize_l1::dialect::vue1::surface::parse_component_block;
/// let arena = Allocator::default();
/// let parsed = {
///     let source = String::from("{{{ x }}}");
///     let block = SourceRoot::new(&source).unwrap().whole_block();
///     parse_component_block(&arena, block)
/// };
/// println!("{:?}", parsed.block());
/// ```
///
/// ```compile_fail
/// use vize_l0::{Allocator, SourceRoot};
/// use vize_l1::dialect::vue1::surface::parse_component_block;
/// let block = SourceRoot::new("{{{ x }}}").unwrap().whole_block();
/// let parsed = {
///     let arena = Allocator::default();
///     parse_component_block(&arena, block)
/// };
/// println!("{:?}", parsed.tree());
/// ```
pub fn parse_component_block<'a>(
    allocator: &'a Allocator,
    block: SourceBlock<'a>,
) -> ComponentParse<'a> {
    projection(allocator, block, false)
}

/// Retain both projections of the same original block construction.
pub fn parse_component_with_authored_block<'a>(
    allocator: &'a Allocator,
    block: SourceBlock<'a>,
) -> ComponentParse<'a> {
    projection(allocator, block, true)
}

fn projection<'a>(
    allocator: &'a Allocator,
    block: SourceBlock<'a>,
    authored: bool,
) -> ComponentParse<'a> {
    let source = block.source();
    let mut unsupported = Vec::new_in(&allocator);
    let mut bindings = OwnedVec::new();
    let mut text_boundaries = OwnedVec::new();
    let (tree, authored, mut errors) =
        construct::<true>(allocator, source, authored, |events, errors| {
            let recorder = Recorder { events, errors };
            let inner = VueSink::<Vue1Policy>::new(allocator, source, recorder, &mut unsupported);
            let sink = TextSink {
                allocator,
                block,
                inner,
                bindings: &mut bindings,
                boundaries: &mut text_boundaries,
            };
            Lexer::<Component, _>::new(
                source,
                sink,
                LexOptions {
                    raw_interpolation: true,
                    ..LexOptions::default()
                },
            )
            .run();
        });
    // Rebase only observations already produced by the single construction;
    // neither the original source, events nor either CST is visited again.
    for error in &mut errors {
        error.offset += block.start();
    }
    for boundary in &mut unsupported {
        boundary.span.start += block.start();
        boundary.span.end += block.start();
    }
    ComponentParse {
        block,
        tree,
        authored,
        errors,
        unsupported,
        bindings,
        text_boundaries,
    }
}

pub(super) struct Vue1Policy;

impl SurfacePolicy for Vue1Policy {
    type Boundary = SyntaxBoundary;

    fn pre(raw: &str, _offset: u32, _source: &str) -> Result<bool, Self::Boundary> {
        // Vue 1.0.28 checks getAttr(el, 'v-pre') before ordinary directives.
        // Argument/modifier forms are not terminal controls in that compiler.
        Ok(raw == "v-pre")
    }

    fn interpolation<'a>(
        source: &'a str,
        recorder: &mut Recorder<'a, '_>,
        unsupported: &mut Vec<'a, Self::Boundary>,
        start: usize,
        end: usize,
        raw: bool,
    ) {
        let content = crate::slice::range(source, start, end);
        let full_raw = raw && source.get(end..end.saturating_add(3)) == Some("}}}");
        let width = if full_raw { 3 } else { 2 };
        let span = Span::new(
            start.saturating_sub(if raw { 3 } else { 2 }) as u32,
            end.saturating_add(width).min(source.len()) as u32,
        );
        let kind = if raw && (!full_raw || content.is_empty()) {
            Some(SyntaxBoundaryKind::RawDelimiterRecovery)
        } else if content.is_empty() {
            Some(SyntaxBoundaryKind::EmptyInterpolation)
        } else if content.contains(['\r', '\u{2028}', '\u{2029}']) {
            Some(SyntaxBoundaryKind::HistoricalLineSeparator)
        } else if content.starts_with('*') {
            Some(SyntaxBoundaryKind::OneTimeInterpolation)
        } else {
            None
        };
        if let Some(kind) = kind {
            unsupported.push(SyntaxBoundary { span, kind });
        }
        if full_raw && !content.is_empty() {
            recorder.raw_interpolation(start, end);
        } else {
            // Preserve the escaped fallback's authored third opening brace.
            recorder.on_interpolation(start - usize::from(raw), end);
            if full_raw {
                recorder.on_text(end + 2, end + 3);
            }
        }
    }
}

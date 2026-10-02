//! Native Vue 1 text syntax with an intrinsic, source-retaining carrier.
//!
//! Tag and attribute recovery shares the existing construction. This bounded
//! component profile does not claim the browser document profile, attribute
//! interpolation, filter/directive embeds, or runtime raw/one-time semantics.

use vize_l0::{Allocator, Span, Vec};

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
    tree: SurfaceTree<'a>,
    authored: Option<SurfaceTree<'a>>,
    errors: Vec<'a, SurfaceError>,
    unsupported: Vec<'a, SyntaxBoundary>,
}

impl<'a> ComponentParse<'a> {
    pub const fn version(&self) -> LegacyVueVersion {
        LegacyVueVersion::V1
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceError {
    SourceTooLarge,
}

/// Construct native Vue 1 text syntax in the component markup profile.
/// Complete nonempty triple delimiters retain an unescaped lexical fact.
/// Raw/empty/one-time recoveries retain bytes and explicit admission boundaries.
pub fn parse_component<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> Result<ComponentParse<'a>, SourceError> {
    projection(allocator, source, false)
}

pub fn parse_component_with_authored<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> Result<ComponentParse<'a>, SourceError> {
    projection(allocator, source, true)
}

fn projection<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    authored: bool,
) -> Result<ComponentParse<'a>, SourceError> {
    if u32::try_from(source.len()).is_err() {
        return Err(SourceError::SourceTooLarge);
    }
    let mut unsupported = Vec::new_in(&allocator);
    let (tree, authored, errors) =
        construct::<true>(allocator, source, authored, |events, errors| {
            let recorder = Recorder { events, errors };
            let sink = VueSink::<Vue1Policy>::new(allocator, source, recorder, &mut unsupported);
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
    Ok(ComponentParse {
        tree,
        authored,
        errors,
        unsupported,
    })
}

struct Vue1Policy;

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

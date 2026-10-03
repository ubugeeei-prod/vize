//! Distinct Vue 2 component syntax, using the existing native lexer/scope visit.

use alloc::vec::Vec as OwnedVec;
use vize_l0::{Allocator, SourceBlock, SourceFrameError, SourceRoot, Vec};

use super::text::{self, TextBinding, TextBoundary, TextBoundaryKind};
use crate::dialect::LegacyVueVersion;
use crate::dialect::vue::surface::{SurfacePolicy, sink::VueSink};
use crate::event::Recorder;
use crate::markup::entity::DecodedEntity;
use crate::markup::token::{LexErrorCode, LexMode, QuoteType};
use crate::markup::{Component, LexOptions, Lexer, Sink};
use crate::parse::{SurfaceError, construct};
use crate::surface::SurfaceTree;

/// An original Vue 2 owner. Captures and modern carriers cannot construct it.
///
/// ```compile_fail
/// use vize_l1::{dialect::vue2::surface, markup};
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
    // NativeSyntax owns parser diagnostics. Ordinary Vec runs their destructors;
    // an arena Vec of these observations would leak their owned storage.
    bindings: OwnedVec<TextBinding<'a>>,
    unsupported: OwnedVec<TextBoundary>,
}

impl<'a> ComponentParse<'a> {
    pub const fn version(&self) -> LegacyVueVersion {
        LegacyVueVersion::V2
    }
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
    pub fn bindings(&self) -> &[TextBinding<'a>] {
        &self.bindings
    }
    pub fn unsupported(&self) -> &[TextBoundary] {
        &self.unsupported
    }
}

/// Default-delimiter, component-profile Vue 2 text syntax, with JS filter pieces.
pub fn parse_component<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> Result<ComponentParse<'a>, SourceFrameError> {
    Ok(parse_component_block(
        allocator,
        SourceRoot::new(source)?.whole_block(),
    ))
}

/// Keep the authentic complete-file frame while parsing just this block once.
pub fn parse_component_block<'a>(
    allocator: &'a Allocator,
    block: SourceBlock<'a>,
) -> ComponentParse<'a> {
    projection(allocator, block, false)
}

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
    let mut boundaries = Vec::new_in(&allocator);
    let mut bindings = OwnedVec::new();
    let mut unsupported = OwnedVec::new();
    let (tree, authored, mut errors) =
        construct::<false>(allocator, source, authored, |events, errors| {
            let recorder = Recorder { events, errors };
            let inner = VueSink::<Vue2Policy>::new(allocator, source, recorder, &mut boundaries);
            let sink = TextSink {
                allocator,
                block,
                inner,
                bindings: &mut bindings,
                unsupported: &mut unsupported,
            };
            Lexer::<Component, _>::new(source, sink, LexOptions::default()).run();
        });
    for error in &mut errors {
        error.offset += block.start();
    }
    ComponentParse {
        block,
        tree,
        authored,
        errors,
        bindings,
        unsupported,
    }
}

struct Vue2Policy;
impl SurfacePolicy for Vue2Policy {
    type Boundary = core::convert::Infallible;
    fn pre(raw: &str, _offset: u32, _source: &str) -> Result<bool, Self::Boundary> {
        // Vue 2's getAndRemoveAttr(el, 'v-pre') selects only this exact spelling.
        Ok(raw == "v-pre")
    }
}

struct TextSink<'a, 'v> {
    allocator: &'a Allocator,
    block: SourceBlock<'a>,
    inner: VueSink<'a, 'v, Vue2Policy>,
    bindings: &'v mut OwnedVec<TextBinding<'a>>,
    unsupported: &'v mut OwnedVec<TextBoundary>,
}

macro_rules! forward {
    ($($name:ident($($arg:ident: $ty:ty),*);)*) => {
        $(fn $name(&mut self, $($arg: $ty),*) { self.inner.$name($($arg),*); })*
    };
}

impl Sink for TextSink<'_, '_> {
    forward! {
        on_text(start: usize, end: usize);
        on_attrib_data(start: usize, end: usize);
        on_attrib_entity(ch: char, start: usize, end: usize);
        on_attrib_entity_value(value: DecodedEntity, start: usize, end: usize);
        on_attrib_end(quote: QuoteType, end: usize);
        on_attrib_name(start: usize, end: usize);
        on_attrib_name_end(end: usize);
        on_dir_name(start: usize, end: usize);
        on_dir_arg(start: usize, end: usize);
        on_dir_modifier(start: usize, end: usize);
        on_comment(start: usize, end: usize);
        on_cdata(start: usize, end: usize);
        on_processing_instruction(start: usize, end: usize);
        on_open_tag_name(start: usize, end: usize);
        on_open_tag_end(end: usize);
        on_self_closing_tag(end: usize);
        on_close_tag(start: usize, end: usize);
        on_end();
        on_error(code: LexErrorCode, index: usize);
    }
    fn on_text_entity(&mut self, ch: char, start: usize, end: usize) {
        if matches!(ch, '{' | '}') {
            self.unsupported.push(TextBoundary {
                span: vize_l0::Span::new(
                    self.block.start() + start as u32,
                    self.block.start() + end as u32,
                ),
                kind: TextBoundaryKind::EncodedDelimiter,
            });
        }
        self.inner.on_text_entity(ch, start, end);
    }
    fn on_text_entity_value(&mut self, value: DecodedEntity, start: usize, end: usize) {
        let mut brace = false;
        value.for_each(|ch| brace |= matches!(ch, '{' | '}'));
        if brace {
            self.unsupported.push(TextBoundary {
                span: vize_l0::Span::new(
                    self.block.start() + start as u32,
                    self.block.start() + end as u32,
                ),
                kind: TextBoundaryKind::EncodedDelimiter,
            });
        }
        self.inner.on_text_entity_value(value, start, end);
    }
    fn on_interpolation(&mut self, start: usize, end: usize) {
        if let Some(binding) = text::observe(self.allocator, self.block, start, end) {
            self.bindings.push(binding);
        }
        self.inner.on_interpolation(start, end);
    }
    fn mode(&self) -> LexMode {
        self.inner.mode()
    }
}

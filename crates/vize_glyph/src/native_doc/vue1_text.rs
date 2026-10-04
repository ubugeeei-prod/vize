//! One authentic Vue 1 escaped-text callback with checked historical printing.

use vize_l0::{Allocator, SourceBlock, Span, String, Vec};
use vize_l1::dialect::vue1::surface::TextView;

use super::{Doc, ExpressionRefusal, LineEnding, PrintOptions, expression::borrowed_document};

/// Whole-document refusal. The component retains its original observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vue1TextDocumentRefusal {
    SourceMismatch {
        span: Span,
    },
    Operand {
        span: Span,
        error: ExpressionRefusal,
    },
}

/// Historical Vue 1 framing rejects every physical CR, including CRLF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vue1TextPrintRefusal {
    CrLf,
}

/// A document retains its authentic callback, expression and child borrow.
/// Only checked printing is public; a bare Doc cannot bypass its CRLF policy.
/// A successful owned String may outlive the original owner, source and arenas.
///
/// ```
/// use vize_glyph::native_doc::{PrintOptions, vue1_text_document};
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue1::surface;
/// let printed = {
///     let arena = Allocator::default();
///     let source = String::from("{{ left+right }}");
///     let owner = surface::parse_component(&arena, &source).unwrap();
///     let view = owner.text_for(owner.children().next().unwrap()).unwrap();
///     let document = vue1_text_document(view, &arena).unwrap();
///     assert!(core::ptr::eq(document.original().child().component(), &owner));
///     document.print(&PrintOptions::default()).unwrap()
/// };
/// assert_eq!(printed.as_str(), "{{ left + right }}");
/// ```
///
/// ```compile_fail,E0451
/// use vize_glyph::native_doc::{Doc, Vue1TextDocument};
/// use vize_l1::dialect::vue1::surface::TextView;
/// fn forge<'o, 'a>(original: TextView<'o, 'a>, document: Doc<'a>) -> Vue1TextDocument<'o, 'a> {
///     Vue1TextDocument { original, document }
/// }
/// ```
/// ```compile_fail
/// use vize_glyph::native_doc::vue1_text_document;
/// use vize_l0::Allocator;
/// let arena = Allocator::default();
/// let document = vue1_text_document("{{ item }}", &arena);
/// ```
/// ```compile_fail,E0515
/// use vize_glyph::native_doc::{Vue1TextDocument, vue1_text_document};
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue1::surface::ComponentParse;
/// fn escape<'a>(owner: ComponentParse<'a>, arena: &'a Allocator) -> Vue1TextDocument<'a, 'a> {
///     vue1_text_document(owner.text_for(owner.children().next().unwrap()).unwrap(), arena).unwrap()
/// }
/// ```
/// ```compile_fail,E0505
/// use vize_glyph::native_doc::{PrintOptions, vue1_text_document};
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue1::surface;
/// let arena = Allocator::default();
/// let owner = surface::parse_component(&arena, "{{ item }}").unwrap();
/// let document = vue1_text_document(owner.text_for(owner.children().next().unwrap()).unwrap(), &arena).unwrap();
/// drop(owner);
/// println!("{}", document.print(&PrintOptions::default()).unwrap());
/// ```
/// ```compile_fail,E0505
/// use vize_glyph::native_doc::vue1_text_document;
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue1::surface;
/// let arena = Allocator::default();
/// let owner = surface::parse_component(&arena, "{{ item }}").unwrap();
/// let document = vue1_text_document(owner.text_for(owner.children().next().unwrap()).unwrap(), &arena).unwrap();
/// let moved = owner;
/// println!("{:?}", document.original());
/// drop(moved);
/// ```
/// ```compile_fail
/// use vize_glyph::native_doc::{PrintOptions, vue1_text_document};
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue1::surface;
/// let arena = Allocator::default();
/// let owner = {
///     let source = String::from("{{ item }}");
///     surface::parse_component(&arena, &source).unwrap()
/// };
/// let document = vue1_text_document(owner.text_for(owner.children().next().unwrap()).unwrap(), &arena).unwrap();
/// println!("{}", document.print(&PrintOptions::default()).unwrap());
/// ```
/// ```compile_fail
/// use vize_glyph::native_doc::{PrintOptions, vue1_text_document};
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue1::surface;
/// let doc_arena = Allocator::default();
/// let owner = {
///     let original_arena = Allocator::default();
///     surface::parse_component(&original_arena, "{{ item }}").unwrap()
/// };
/// let document = vue1_text_document(owner.text_for(owner.children().next().unwrap()).unwrap(), &doc_arena).unwrap();
/// println!("{}", document.print(&PrintOptions::default()).unwrap());
/// ```
/// ```compile_fail
/// use vize_glyph::native_doc::{PrintOptions, vue1_text_document};
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue1::surface;
/// let arena = Allocator::default();
/// let owner = surface::parse_component(&arena, "{{ item }}").unwrap();
/// let document = {
///     let doc_arena = Allocator::default();
///     vue1_text_document(owner.text_for(owner.children().next().unwrap()).unwrap(), &doc_arena).unwrap()
/// };
/// println!("{}", document.print(&PrintOptions::default()).unwrap());
/// ```
/// ```compile_fail,E0599
/// use vize_glyph::native_doc::{LineEnding, PrintOptions, print, vue1_text_document};
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue1::surface;
/// let arena = Allocator::default();
/// let owner = surface::parse_component(&arena, "{{ left+right }}").unwrap();
/// let document = vue1_text_document(owner.text_for(owner.children().next().unwrap()).unwrap(), &arena).unwrap();
/// let options = PrintOptions { width: 1, line_ending: LineEnding::CrLf, ..PrintOptions::default() };
/// let bypass = print(document.document(), &options);
/// ```
#[derive(Debug)]
pub struct Vue1TextDocument<'o, 'a> {
    original: TextView<'o, 'a>,
    document: Doc<'a>,
}

impl<'o, 'a> Vue1TextDocument<'o, 'a> {
    pub fn original(&self) -> &TextView<'o, 'a> {
        &self.original
    }

    /// Reject CRLF before output, even when this particular Doc might stay flat.
    pub fn print(&self, options: &PrintOptions) -> Result<String, Vue1TextPrintRefusal> {
        if options.line_ending == LineEnding::CrLf {
            return Err(Vue1TextPrintRefusal::CrLf);
        }
        Ok(super::print(&self.document, options))
    }
}

/// Project only the genuine admitted original callback through the unchanged
/// shared Glyph expression visitor. Authored delimiters and trim gaps remain
/// complete original source slices. This adds no parse/decode/AST pass, Vue 2
/// grammar, classic-runtime admission, historical File or default route.
pub fn vue1_text_document<'o, 'a>(
    original: TextView<'o, 'a>,
    allocator: &'a Allocator,
) -> Result<Vue1TextDocument<'o, 'a>, Vue1TextDocumentRefusal> {
    let block = original.child().component().block();
    let binding = original.binding().span();
    if !block.contains_block_span(binding) {
        return Err(Vue1TextDocumentRefusal::SourceMismatch { span: binding });
    }
    let syntax = original
        .binding()
        .syntax()
        .ok_or(Vue1TextDocumentRefusal::SourceMismatch { span: binding })?;
    let span = syntax.source().span();
    let content = original.binding().content_span();
    if span.start < content.start || span.start >= span.end || span.end > content.end {
        return Err(Vue1TextDocumentRefusal::SourceMismatch { span });
    }
    let view = syntax
        .borrow_expression()
        .ok_or(Vue1TextDocumentRefusal::Operand {
            span,
            error: ExpressionRefusal::Unadmitted {
                hole: syntax.hole(),
            },
        })?;
    if !core::ptr::eq(view.expression(), original.expression())
        || !core::ptr::eq(view.source().authored_root(), block.root_source())
    {
        return Err(Vue1TextDocumentRefusal::SourceMismatch { span });
    }
    let projected = borrowed_document(&view, block, allocator)
        .map_err(|error| Vue1TextDocumentRefusal::Operand { span, error })?;
    let mut parts = Vec::new_in(&allocator);
    parts.push(Doc::text(authored(
        block,
        Span::new(binding.start, span.start),
    )?));
    parts.push(projected);
    parts.push(Doc::text(authored(
        block,
        Span::new(span.end, binding.end),
    )?));
    Ok(Vue1TextDocument {
        original,
        document: Doc::concat(parts),
    })
}

fn authored<'a>(block: SourceBlock<'a>, span: Span) -> Result<&'a str, Vue1TextDocumentRefusal> {
    if !block.contains_block_span(span) {
        return Err(Vue1TextDocumentRefusal::SourceMismatch { span });
    }
    block
        .root_source()
        .get(span.start as usize..span.end as usize)
        .ok_or(Vue1TextDocumentRefusal::SourceMismatch { span })
}

#[cfg(test)]
#[path = "vue1_text/tests.rs"]
mod tests;

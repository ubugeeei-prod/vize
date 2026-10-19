//! One original Vue 2 interpolation, with authored filter gaps kept verbatim.

use vize_l0::{Allocator, SourceBlock, Span, Vec};
use vize_l1::{dialect::vue2::surface::TextView, embed::syntax::NativeSyntax};

use super::{Doc, ExpressionRefusal, expression::borrowed_document};

/// A refusal of the whole document; the original component still owns all facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vue2TextDocumentRefusal {
    SourceMismatch {
        span: Span,
    },
    Operand {
        span: Span,
        error: ExpressionRefusal,
    },
}

/// The document retains its authentic original child/chain borrow.
/// Doc access is borrowed; source slices do not grant independent AST authority.
///
/// ```
/// use vize_glyph::native_doc::{PrintOptions, print, vue2_text_document};
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue2::surface;
/// let arena = Allocator::default();
/// let owner = surface::parse_component(&arena, "{{ left+right | upper () }}").unwrap();
/// let view = owner.text_for(owner.children().next().unwrap()).unwrap();
/// let document = vue2_text_document(view, &arena).unwrap();
/// assert_eq!(print(document.document(), &PrintOptions::default()), "{{ left + right | upper () }}");
/// assert!(core::ptr::eq(document.original().child().component(), &owner));
/// ```
///
/// ```compile_fail,E0451
/// use vize_glyph::native_doc::{Doc, Vue2TextDocument};
/// use vize_l1::dialect::vue2::surface::TextView;
/// fn forge<'o, 'a>(original: TextView<'o, 'a>, document: Doc<'a>) -> Vue2TextDocument<'o, 'a> {
///     Vue2TextDocument { original, document }
/// }
/// ```
/// ```compile_fail
/// use vize_glyph::native_doc::vue2_text_document;
/// use vize_l0::Allocator;
/// let arena = Allocator::default();
/// let document = vue2_text_document("{{ item }}", &arena);
/// ```
/// ```compile_fail,E0515
/// use vize_glyph::native_doc::{Vue2TextDocument, vue2_text_document};
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue2::surface::ComponentParse;
/// fn escape<'a>(owner: ComponentParse<'a>, arena: &'a Allocator) -> Vue2TextDocument<'a, 'a> {
///     vue2_text_document(owner.text_for(owner.children().next().unwrap()).unwrap(), arena).unwrap()
/// }
/// ```
/// ```compile_fail,E0505
/// use vize_glyph::native_doc::{PrintOptions, print, vue2_text_document};
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue2::surface;
/// let arena = Allocator::default();
/// let owner = surface::parse_component(&arena, "{{ item }}").unwrap();
/// let document = vue2_text_document(owner.text_for(owner.children().next().unwrap()).unwrap(), &arena).unwrap();
/// drop(owner);
/// println!("{}", print(document.document(), &PrintOptions::default()));
/// ```
/// ```compile_fail
/// use vize_glyph::native_doc::{PrintOptions, print, vue2_text_document};
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue2::surface;
/// let doc_arena = Allocator::default();
/// let owner = {
///     let original_arena = Allocator::default();
///     surface::parse_component(&original_arena, "{{ item }}").unwrap()
/// };
/// let document = vue2_text_document(owner.text_for(owner.children().next().unwrap()).unwrap(), &doc_arena).unwrap();
/// println!("{}", print(document.document(), &PrintOptions::default()));
/// ```
/// ```compile_fail,E0505
/// use vize_glyph::native_doc::vue2_text_document;
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue2::surface;
/// let arena = Allocator::default();
/// let owner = surface::parse_component(&arena, "{{ item }}").unwrap();
/// let document = vue2_text_document(owner.text_for(owner.children().next().unwrap()).unwrap(), &arena).unwrap();
/// let moved = owner;
/// println!("{:?}", document.original());
/// drop(moved);
/// ```
/// ```compile_fail
/// use vize_glyph::native_doc::{PrintOptions, print, vue2_text_document};
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue2::surface;
/// let arena = Allocator::default();
/// let owner = {
///     let source = String::from("{{ item }}");
///     surface::parse_component(&arena, &source).unwrap()
/// };
/// let document = vue2_text_document(owner.text_for(owner.children().next().unwrap()).unwrap(), &arena).unwrap();
/// println!("{}", print(document.document(), &PrintOptions::default()));
/// ```
/// ```compile_fail
/// use vize_glyph::native_doc::{PrintOptions, print, vue2_text_document};
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue2::surface;
/// let arena = Allocator::default();
/// let owner = surface::parse_component(&arena, "{{ item }}").unwrap();
/// let document = {
///     let doc_arena = Allocator::default();
///     vue2_text_document(owner.text_for(owner.children().next().unwrap()).unwrap(), &doc_arena).unwrap()
/// };
/// println!("{}", print(document.document(), &PrintOptions::default()));
/// ```
#[derive(Debug)]
pub struct Vue2TextDocument<'o, 'a> {
    original: TextView<'o, 'a>,
    document: Doc<'a>,
}

impl<'o, 'a> Vue2TextDocument<'o, 'a> {
    pub fn original(&self) -> &TextView<'o, 'a> {
        &self.original
    }
    pub fn document(&self) -> &Doc<'a> {
        &self.document
    }

    // Trusted whole-owner composition; no public original/Doc transfer.
    pub(super) fn into_parts(self) -> (TextView<'o, 'a>, Doc<'a>) {
        (self.original, self.document)
    }
}

/// Project the original admitted base and arguments through the same Glyph
/// callback, once each. Every intervening authored byte remains a source slice;
/// registry names and blank/trailing lists are never inferred or scanned again.
/// L1 admission failures remain at `ComponentParse::text_for`, before a view
/// can exist. Unsupported descendants refuse the complete Doc. This adds no
/// historical File, render-function ABI, whole grammar or default route.
pub fn vue2_text_document<'o, 'a>(
    original: TextView<'o, 'a>,
    allocator: &'a Allocator,
) -> Result<Vue2TextDocument<'o, 'a>, Vue2TextDocumentRefusal> {
    let block = original.child().component().block();
    let binding = original.binding().span();
    if !block.contains_block_span(binding) {
        return Err(Vue2TextDocumentRefusal::SourceMismatch { span: binding });
    }
    let mut cursor = binding.start;
    let mut parts = Vec::new_in(&allocator);
    append_operand(
        original.chain().base(),
        block,
        binding,
        allocator,
        &mut cursor,
        &mut parts,
    )?;
    for filter in original.chain().filters() {
        for argument in filter.arguments() {
            append_operand(argument, block, binding, allocator, &mut cursor, &mut parts)?;
        }
    }
    parts.push(Doc::text(authored(block, Span::new(cursor, binding.end))?));
    Ok(Vue2TextDocument {
        original,
        document: Doc::concat(parts),
    })
}

fn authored<'a>(block: SourceBlock<'a>, span: Span) -> Result<&'a str, Vue2TextDocumentRefusal> {
    if !block.contains_block_span(span) {
        return Err(Vue2TextDocumentRefusal::SourceMismatch { span });
    }
    block
        .root_source()
        .get(span.start as usize..span.end as usize)
        .ok_or(Vue2TextDocumentRefusal::SourceMismatch { span })
}

fn append_operand<'a>(
    original: &NativeSyntax<'a>,
    block: SourceBlock<'a>,
    binding: Span,
    allocator: &'a Allocator,
    cursor: &mut u32,
    parts: &mut Vec<'a, Doc<'a>>,
) -> Result<(), Vue2TextDocumentRefusal> {
    let span = original.source().span();
    if span.start < *cursor || span.start >= span.end || span.end > binding.end {
        return Err(Vue2TextDocumentRefusal::SourceMismatch { span });
    }
    let view = original
        .borrow_expression()
        .ok_or(Vue2TextDocumentRefusal::Operand {
            span,
            error: ExpressionRefusal::Unadmitted {
                hole: original.hole(),
            },
        })?;
    let document = borrowed_document(&view, block, allocator)
        .map_err(|error| Vue2TextDocumentRefusal::Operand { span, error })?;
    parts.push(Doc::text(authored(block, Span::new(*cursor, span.start))?));
    parts.push(document);
    *cursor = span.end;
    Ok(())
}

#[cfg(test)]
mod tests {
    //! Independent complete Doc trees and genuine private borrowed-origin controls.

    use super::*;
    use crate::native_doc::{Line, PrintOptions, document::Kind, print};
    use vize_l1::dialect::vue2::surface;

    #[derive(Debug, PartialEq, Eq)]
    enum Tree<'a> {
        Text(&'a str),
        Line(Line),
        HardLine,
        Concat(std::vec::Vec<Self>),
        Group(std::boxed::Box<Self>),
        Indent(usize, std::boxed::Box<Self>),
    }
    fn tree<'a>(doc: &Doc<'a>) -> Tree<'a> {
        match &doc.kind {
            Kind::Text(text) => Tree::Text(text),
            Kind::Line(line) => Tree::Line(*line),
            Kind::HardLine => Tree::HardLine,
            Kind::Concat(parts) => Tree::Concat(parts.iter().map(tree).collect()),
            Kind::Group(inner) => Tree::Group(std::boxed::Box::new(tree(inner))),
            Kind::Indent(levels, inner) => Tree::Indent(*levels, std::boxed::Box::new(tree(inner))),
        }
    }
    fn concat(parts: std::vec::Vec<Tree<'_>>) -> Tree<'_> {
        Tree::Concat(parts)
    }
    fn atom(text: &str) -> Tree<'_> {
        concat(vec![Tree::Text(""), Tree::Text(text), Tree::Text("")])
    }
    fn infix<'a>(left: &'a str, operator: &'a str, right: &'a str) -> Tree<'a> {
        concat(vec![
            Tree::Text(""),
            Tree::Group(std::boxed::Box::new(concat(vec![
                Tree::Text(left),
                Tree::Text(" "),
                Tree::Text(operator),
                Tree::Indent(
                    1,
                    std::boxed::Box::new(concat(vec![Tree::Line(Line::Space), Tree::Text(right)])),
                ),
            ]))),
            Tree::Text(""),
        ])
    }

    #[test]
    fn complete_doc_trees_keep_every_authored_gap_and_original_operand_structure() {
        for (source, expected) in [
            (
                "{{ a | f( b, ) }}",
                concat(vec![
                    Tree::Text("{{ "),
                    atom("a"),
                    Tree::Text(" | f( "),
                    atom("b"),
                    Tree::Text(", ) }}"),
                ]),
            ),
            (
                "{{ a+b | f( c*d, ) }}",
                concat(vec![
                    Tree::Text("{{ "),
                    infix("a", "+", "b"),
                    Tree::Text(" | f( "),
                    infix("c", "*", "d"),
                    Tree::Text(", ) }}"),
                ]),
            ),
            (
                "{{ a&#43;b &#124; f() }}",
                concat(vec![
                    Tree::Text("{{ "),
                    infix("a", "&#43;", "b"),
                    Tree::Text(" &#124; f() }}"),
                ]),
            ),
            (
                "{{ 日本 | upper () }}",
                concat(vec![
                    Tree::Text("{{ "),
                    atom("日本"),
                    Tree::Text(" | upper () }}"),
                ]),
            ),
        ] {
            let arena = Allocator::default();
            let owner = surface::parse_component(&arena, source).unwrap();
            let view = owner.text_for(owner.children().next().unwrap()).unwrap();
            let document = vue2_text_document(view, &arena).unwrap();
            assert_eq!(tree(document.document()), expected, "{source}");
            let Kind::Concat(parts) = &document.document().kind else {
                panic!("complete root")
            };
            let Kind::Text(prefix) = parts[0].kind else {
                panic!("complete prefix")
            };
            assert_eq!(prefix.as_ptr(), source.as_ptr());
            let Kind::Text(tail) = parts.last().unwrap().kind else {
                panic!("complete tail")
            };
            assert_eq!(
                tail.as_ptr(),
                source.get(source.len() - tail.len()..).unwrap().as_ptr()
            );
        }
    }
}

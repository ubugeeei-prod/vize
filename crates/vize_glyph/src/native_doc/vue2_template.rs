//! One original historical child/token visit, without a modern carrier.

use vize_l0::{Allocator, SourceBlock, SourceFrameError, Span, Vec};
use vize_l1::{
    SurfaceChild,
    dialect::vue2::surface::{ComponentParse, TextChild, TextChildren},
};

use super::template::{Cursor, close_element, verbatim};
use super::vue2_sfc::{NativeVue2SfcNode, NativeVue2SfcRefusal};
use super::{Doc, TemplateRefusal, UnsupportedSyntax, Vue2TextDocument, vue2_text_document};

#[path = "vue2_template/header.rs"]
mod header;

pub(super) fn document<'a>(
    component: &ComponentParse<'a>,
    allocator: &'a Allocator,
) -> Result<Doc<'a>, NativeVue2SfcRefusal> {
    if let Some(error) = component.errors().first() {
        return Err(NativeVue2SfcRefusal::ComponentRecovery {
            offset: error.offset,
        });
    }
    if component.authored().is_some() {
        return Err(NativeVue2SfcRefusal::ComponentRecovery {
            offset: component.block().start(),
        });
    }
    if let Some(boundary) = component.unsupported().first() {
        return Err(NativeVue2SfcRefusal::ComponentBoundary {
            span: boundary.span,
            kind: boundary.kind,
        });
    }
    if !core::ptr::eq(component.tree().source, component.block().source()) {
        return Err(TemplateRefusal::SourceMismatch { offset: 0 }.into());
    }
    let mut cursor = Cursor {
        source: component.block().source(),
        offset: 0,
    };
    let mut parts = Vec::new_in(&allocator);
    children(component.children(), &mut parts, &mut cursor, allocator, 0)?;
    if cursor.offset != cursor.source.len() {
        return Err(TemplateRefusal::SourceMismatch {
            offset: cursor.offset,
        }
        .into());
    }
    Ok(Doc::concat(parts))
}

fn children<'a>(
    children: TextChildren<'_, 'a>,
    parts: &mut Vec<'a, Doc<'a>>,
    cursor: &mut Cursor<'a>,
    allocator: &'a Allocator,
    depth: usize,
) -> Result<(), NativeVue2SfcRefusal> {
    if depth > 128 {
        return Err(TemplateRefusal::Unsupported {
            offset: cursor.offset,
            syntax: UnsupportedSyntax::NestingLimit,
        }
        .into());
    }
    for child in children {
        match child.surface() {
            SurfaceChild::Element(element) => {
                header::open(
                    element,
                    child.component().block(),
                    parts,
                    cursor,
                    allocator,
                    depth,
                )?;
                let nested = child.children().ok_or(TemplateRefusal::SourceMismatch {
                    offset: cursor.offset,
                })?;
                self::children(nested, parts, cursor, allocator, depth + 1)?;
                close_element(element, parts, cursor)?;
            }
            SurfaceChild::Interpolation(node) => {
                let block = child.component().block();
                let open = span(block, node.open.text)?;
                let close = span(block, node.close.text)?;
                let event = Span::new(open.start, close.end);
                cursor.token(&node.open)?;
                cursor.token(&node.content)?;
                cursor.token(&node.close)?;
                let view = child
                    .component()
                    .text_for(child.reborrow())
                    .map_err(|refusal| NativeVue2SfcRefusal::Text {
                        span: event,
                        refusal,
                    })?;
                let document =
                    vue2_text_document(view, allocator).map_err(NativeVue2SfcRefusal::TextDoc)?;
                parts.push(Doc::text(node.open.leading));
                append_text(&child, event, document, parts, allocator, depth)?;
            }
            SurfaceChild::Text(token) | SurfaceChild::Comment(token) => {
                cursor.token(token)?;
                verbatim(parts, token);
            }
            SurfaceChild::Unexpected(_) => {
                return Err(TemplateRefusal::Recovered {
                    offset: cursor.offset,
                }
                .into());
            }
            SurfaceChild::Cdata(token) | SurfaceChild::ProcessingInstruction(token) => {
                let kind = if matches!(child.surface(), SurfaceChild::Cdata(_)) {
                    NativeVue2SfcNode::Cdata
                } else {
                    NativeVue2SfcNode::ProcessingInstruction
                };
                return Err(NativeVue2SfcRefusal::UnsupportedNode {
                    span: span(child.component().block(), token.text)?,
                    kind,
                });
            }
        }
    }
    Ok(())
}

fn append_text<'a>(
    expected: &TextChild<'_, 'a>,
    event: Span,
    document: Vue2TextDocument<'_, 'a>,
    parts: &mut Vec<'a, Doc<'a>>,
    allocator: &'a Allocator,
    depth: usize,
) -> Result<(), NativeVue2SfcRefusal> {
    let (original, document) = document.into_parts();
    let actual = original.child();
    let same_parent = match (actual.parent_element(), expected.parent_element()) {
        (None, None) => true,
        (Some(a), Some(b)) => core::ptr::eq(a, b),
        _ => false,
    };
    if !core::ptr::eq(actual.component(), expected.component())
        || !core::ptr::eq(actual.surface(), expected.surface())
        || actual.ordinal() != expected.ordinal()
        || !same_parent
        || original.binding().span() != event
    {
        return Err(NativeVue2SfcRefusal::Frame {
            span: event,
            error: SourceFrameError::BlockNotRootSlice,
        });
    }
    parts.push(document.indent(depth, allocator));
    Ok(())
}

pub(super) fn span(block: SourceBlock<'_>, text: &str) -> Result<Span, NativeVue2SfcRefusal> {
    block.span_of(text).ok_or(NativeVue2SfcRefusal::Frame {
        span: block.span(),
        error: SourceFrameError::BlockNotRootSlice,
    })
}

#[cfg(test)]
#[path = "vue2_template/tests.rs"]
mod tests;

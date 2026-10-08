//! The shared original-child traversal and exact interpolation framing.

use vize_l0::{Allocator, Vec};
use vize_l1::SurfaceChild;
use vize_l1::markup::{
    NativeChild, NativeChildren, NativeInterpolationOperand, NativeTemplateComponent,
};

use super::super::template::{Cursor, close_element, open_element, verbatim};
use super::input::Input;
use super::value::ValuePolicy;
use super::{
    Doc, Line, NativeTemplateRefusal, TemplateRefusal, UnsupportedSyntax, expression_document,
};

pub(super) struct Builder<'p, 'a, I> {
    pub(super) selected: &'p NativeTemplateComponent<'a>,
    pub(super) input: I,
    pub(super) next: usize,
    pub(super) cursor: Cursor<'a>,
    pub(super) allocator: &'a Allocator,
    pub(super) values: ValuePolicy<'a>,
}

impl<'p, 'a, I: Input<'p, 'a>> Builder<'p, 'a, I> {
    pub(super) fn children(
        &mut self,
        children: NativeChildren<'p, 'a>,
        parts: &mut Vec<'a, Doc<'a>>,
        depth: usize,
    ) -> Result<(), NativeTemplateRefusal> {
        if depth > 128 {
            return Err(TemplateRefusal::Unsupported {
                offset: self.cursor.offset,
                syntax: UnsupportedSyntax::NestingLimit,
            }
            .into());
        }
        for child in children {
            match child.surface() {
                SurfaceChild::Element(_) => {
                    let element = child
                        .into_element()
                        .ok_or(TemplateRefusal::SourceMismatch {
                            offset: self.cursor.offset,
                        })?;
                    if self.values.formats_conditionals() && super::conditional::eligible(&element)
                    {
                        super::conditional::open_element(self, &element, parts, depth)?;
                    } else {
                        // Preserve the old strict/opaque visit order for recovered
                        // or verbatim headers; this is only negative routing.
                        open_element(
                            element.surface(),
                            parts,
                            &mut self.cursor,
                            self.allocator,
                            depth,
                            &self.values,
                        )?;
                    }
                    self.children(element.children(), parts, depth + 1)?;
                    close_element(element.surface(), parts, &mut self.cursor)?;
                }
                SurfaceChild::Interpolation(_) => self.interpolation(child, parts, depth)?,
                SurfaceChild::Unexpected(_) => {
                    return Err(TemplateRefusal::Recovered {
                        offset: self.cursor.offset,
                    }
                    .into());
                }
                SurfaceChild::Text(token)
                | SurfaceChild::Comment(token)
                | SurfaceChild::Cdata(token)
                | SurfaceChild::ProcessingInstruction(token) => {
                    self.cursor.token(token)?;
                    verbatim(parts, token);
                }
            }
        }
        Ok(())
    }

    fn interpolation(
        &mut self,
        child: NativeChild<'p, 'a>,
        parts: &mut Vec<'a, Doc<'a>>,
        depth: usize,
    ) -> Result<(), NativeTemplateRefusal> {
        let offset = self.cursor.offset;
        let operand = self
            .input
            .operand(self.selected, &child, self.next, offset)?;
        let view = operand.admitted_for(self.selected, child).ok_or(
            NativeTemplateRefusal::OperandRejected {
                offset,
                index: self.next,
                hole: operand.syntax().hole(),
            },
        )?;
        let SurfaceChild::Interpolation(interpolation) = view.child().surface() else {
            return Err(TemplateRefusal::SourceMismatch { offset }.into());
        };
        self.cursor.token(&interpolation.open)?;
        self.cursor.token(&interpolation.content)?;
        self.cursor.token(&interpolation.close)?;
        let expression = expression_document(
            view.operand().syntax(),
            self.selected.component().block(),
            self.allocator,
        )
        .map_err(|refusal| NativeTemplateRefusal::Expression { offset, refusal })?;
        let (_, expression) = expression.into_parts();
        parts.push(Doc::text(interpolation.open.leading));
        let mut framed = Vec::new_in(&self.allocator);
        framed.push(Doc::text(interpolation.open.text));
        let mut content = Vec::new_in(&self.allocator);
        content.push(Doc::line(Line::Space));
        content.push(expression);
        framed.push(Doc::concat(content).indent(depth + 1, self.allocator));
        framed.push(closing_edge(operand, offset, depth, self.allocator)?);
        framed.push(Doc::text(interpolation.close.leading));
        framed.push(Doc::text(interpolation.close.text));
        parts.push(Doc::concat(framed).group(self.allocator));
        self.next += 1;
        Ok(())
    }
}

/// The Descriptor scans authored bytes before text decoding. Decoded comment
/// or string kinds cannot classify that framing; preserve any actual trimmed
/// LF tail without inventing a terminator from an entity or private wrapper.
fn closing_edge<'a>(
    operand: &NativeInterpolationOperand<'a>,
    offset: usize,
    depth: usize,
    allocator: &'a Allocator,
) -> Result<Doc<'a>, NativeTemplateRefusal> {
    let source = operand.syntax().source();
    let original_tail = source
        .authored_root()
        .get(source.span().end as usize..operand.content_span().end as usize)
        .ok_or(TemplateRefusal::SourceMismatch { offset })?;
    if original_tail.contains('\n') {
        return Ok(Doc::text(original_tail));
    }
    Ok(Doc::line(Line::Space).indent(depth, allocator))
}

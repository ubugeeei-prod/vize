//! Original Descriptor-selected templates with genuine once-retained embeds.

use vize_l0::{Allocator, Vec};
use vize_l1::SurfaceChild;
use vize_l1::embed::syntax::EmbedHole;
use vize_l1::markup::{
    NativeChild, NativeChildren, NativeInterpolationOperand, NativeTemplateComponent,
};

use super::template::{Cursor, close_element, open_element, verbatim};
use super::{
    Doc, ExpressionRefusal, Line, TemplateRefusal, UnsupportedSyntax, expression_document,
};

/// Offsets are relative to the selected template block. Wrapped expression
/// refusals keep their provider's authored-file or decoded-relative coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTemplateRefusal {
    Template(TemplateRefusal),
    MissingOperand {
        offset: usize,
        index: usize,
    },
    ExtraOperand {
        index: usize,
    },
    OperandRejected {
        offset: usize,
        index: usize,
        hole: Option<EmbedHole>,
    },
    Expression {
        offset: usize,
        refusal: ExpressionRefusal,
    },
}

impl From<TemplateRefusal> for NativeTemplateRefusal {
    fn from(refusal: TemplateRefusal) -> Self {
        Self::Template(refusal)
    }
}

/// Original selected owner and operands remain borrowed for the Doc's lifetime.
/// The document covers only that template block, not its enclosing SFC.
pub struct NativeTemplateDocument<'p, 'a> {
    original: &'p NativeTemplateComponent<'a>,
    operands: &'p [&'p NativeInterpolationOperand<'a>],
    document: Doc<'a>,
}

impl core::fmt::Debug for NativeTemplateDocument<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("NativeTemplateDocument")
            .field("original", &self.original)
            .field("operand_count", &self.operands.len())
            .field("document", &self.document)
            .finish()
    }
}

impl<'p, 'a> NativeTemplateDocument<'p, 'a> {
    pub fn original(&self) -> &'p NativeTemplateComponent<'a> {
        self.original
    }
    pub fn operands(&self) -> &'p [&'p NativeInterpolationOperand<'a>] {
        self.operands
    }
    pub fn document(&self) -> &Doc<'a> {
        &self.document
    }
}

/// Lay out one genuine selected Vue 3 template and its ordered original embeds.
///
/// The caller retains each existing `observe_interpolation_expression` result
/// in depth-first source order. Each is rejoined at its exact original body
/// event with `admitted_for`; raw AST/source pairs cannot supply admission.
/// Missing, extra, foreign, recovered or unsupported operands refuse the whole
/// document without replacing any original observations. Native v-pre text
/// remains text and requires no operand. Opening-tag layout uses the unchanged
/// bare consumer's helpers. Interpolation framing gets breakable edge spaces;
/// original expression comments, entities and literal bytes stay authored.
/// Original trimmed tails containing LF/CRLF remain authored, preserving the
/// enclosing Descriptor's physical boundary scan independently of decoded AST kind.
/// This API performs no parse, decode, AST normalization or product default switch.
///
/// ```compile_fail
/// use vize_glyph::native_doc::native_template_document;
/// use vize_l0::Allocator;
/// use vize_l1::{embed::syntax::RetainedExpression, markup::NativeTemplateComponent};
/// fn raw_cannot_admit<'a>(selected: &NativeTemplateComponent<'a>,
///     raw: &RetainedExpression<'a>, arena: &'a Allocator) {
///     let _ = native_template_document(selected, &[raw], arena);
/// }
/// ```
pub fn native_template_document<'p, 'a>(
    original: &'p NativeTemplateComponent<'a>,
    operands: &'p [&'p NativeInterpolationOperand<'a>],
    allocator: &'a Allocator,
) -> Result<NativeTemplateDocument<'p, 'a>, NativeTemplateRefusal> {
    let carrier = original.component().carrier();
    if let Some(error) = carrier.errors.first() {
        return Err(TemplateRefusal::Recovered {
            offset: error.offset as usize,
        }
        .into());
    }
    if carrier.authored.is_some() || !carrier.unsupported.is_empty() {
        return Err(TemplateRefusal::Recovered { offset: 0 }.into());
    }
    let mut builder = Builder {
        selected: original,
        operands,
        next: 0,
        cursor: Cursor {
            source: original.component().block().source(),
            offset: 0,
        },
        allocator,
    };
    let mut parts = Vec::new_in(&allocator);
    builder.children(original.children(), &mut parts, 0)?;
    if builder.cursor.offset != builder.cursor.source.len() {
        return Err(TemplateRefusal::SourceMismatch {
            offset: builder.cursor.offset,
        }
        .into());
    }
    if builder.next != operands.len() {
        return Err(NativeTemplateRefusal::ExtraOperand {
            index: builder.next,
        });
    }
    Ok(NativeTemplateDocument {
        original,
        operands,
        document: Doc::concat(parts),
    })
}

struct Builder<'p, 'a> {
    selected: &'p NativeTemplateComponent<'a>,
    operands: &'p [&'p NativeInterpolationOperand<'a>],
    next: usize,
    cursor: Cursor<'a>,
    allocator: &'a Allocator,
}

impl<'p, 'a> Builder<'p, 'a> {
    fn children(
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
                    open_element(
                        element.surface(),
                        parts,
                        &mut self.cursor,
                        self.allocator,
                        depth,
                    )?;
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
        let Some(&operand) = self.operands.get(self.next) else {
            return Err(NativeTemplateRefusal::MissingOperand {
                offset,
                index: self.next,
            });
        };
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

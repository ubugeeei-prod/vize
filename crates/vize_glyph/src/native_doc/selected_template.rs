//! Original Descriptor-selected templates with genuine once-retained embeds.

use vize_l0::{Allocator, Span, Vec};
use vize_l1::embed::syntax::EmbedHole;
use vize_l1::markup::{NativeInterpolationOperand, NativeTemplateComponent};

use super::template::Cursor;
use super::{
    Doc, ExpressionRefusal, Line, TemplateRefusal, UnsupportedSyntax, expression_document,
};

#[path = "selected_template/builder.rs"]
mod builder;
#[path = "selected_template/input.rs"]
mod input;
#[path = "selected_template/observed.rs"]
mod observed;
#[path = "selected_template/value.rs"]
mod value;
use builder::Builder;
use input::Borrowed;
pub use value::NativeTemplateValuePolicy;
use value::ValuePolicy;

pub use observed::{
    ObservedNativeTemplateDocument, ObservedNativeTemplateFailure, ObservedNativeTemplateRefusal,
    observed_native_template_document, observed_native_template_document_with_policy,
};

/// Offsets are relative to the selected template block. Wrapped expression
/// refusals keep their provider's authored-file or decoded-relative coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTemplateRefusal {
    Template(TemplateRefusal),
    /// Exact original directive-value content in authored-file coordinates.
    DirectiveValue {
        span: Span,
    },
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
    check_selected(original)?;
    let mut builder = Builder {
        selected: original,
        input: Borrowed(operands),
        next: 0,
        cursor: Cursor {
            source: original.component().block().source(),
            offset: 0,
        },
        allocator,
        values: ValuePolicy::new(
            original.component().block(),
            NativeTemplateValuePolicy::PreserveOpaque,
        ),
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

fn check_selected(original: &NativeTemplateComponent<'_>) -> Result<(), NativeTemplateRefusal> {
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
    Ok(())
}

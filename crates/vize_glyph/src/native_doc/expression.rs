//! Native documents over the original compiler-profile expression tree.

use oxc_span::GetSpan;
use vize_l0::{Allocator, SourceBlock, Span, Vec};
use vize_l1::embed::{
    SourceError,
    syntax::{EmbedHole, NativeExpressionView, RetainedExpression},
};

use super::Doc;

#[path = "expression/array.rs"]
mod array;
#[path = "expression/ast.rs"]
mod ast;
#[path = "expression/object.rs"]
mod object;
#[path = "expression/origin.rs"]
mod origin;
#[path = "expression/sequence.rs"]
mod sequence;
#[path = "expression/source.rs"]
mod source;
use origin::Origin;
use source::{Context, Gap};

/// A bounded syntax consumer; larger L1 safety admissions remain independent.
pub const MAX_EXPRESSION_DOCUMENT_DEPTH: usize = 16;

/// Refusal retains the caller's original parse, diagnostics and comments.
/// SourceMismatch spans are authored-file coordinates; syntax/gap/depth spans
/// are decoded-relative coordinates in that retained expression's source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionRefusal {
    Unadmitted { hole: Option<EmbedHole> },
    SourceMismatch { span: Span },
    Projection { offset: u32, error: SourceError },
    UnsupportedNode { span: Span },
    InvalidGap { span: Span },
    InvalidFraming { span: Span },
    DepthLimit { span: Span },
}

/// The document borrows the same admitted expression and authored source.
/// Its original observation cannot be dropped while that borrow is returned.
///
/// ```compile_fail
/// use vize_glyph::native_doc::{ExpressionDocument, expression_document};
/// use vize_l0::{Allocator, SourceBlock};
/// use vize_l1::embed::syntax::RetainedExpression;
/// fn outlive_owner<'a>(
///     arena: &'a Allocator,
///     owner: RetainedExpression<'a>,
///     block: SourceBlock<'a>,
/// ) -> ExpressionDocument<'a, 'a> {
///     expression_document(&owner, block, arena).unwrap()
/// }
/// ```
#[derive(Debug)]
pub struct ExpressionDocument<'p, 'a> {
    original: &'p RetainedExpression<'a>,
    document: Doc<'a>,
}

impl<'p, 'a> ExpressionDocument<'p, 'a> {
    pub fn original(&self) -> &'p RetainedExpression<'a> {
        self.original
    }
    pub fn document(&self) -> &Doc<'a> {
        &self.document
    }
    /// Transfer both the same retained borrow and its document to a consumer.
    pub fn into_parts(self) -> (&'p RetainedExpression<'a>, Doc<'a>) {
        (self.original, self.document)
    }
}

#[cfg(test)]
#[path = "expression/projection_tests.rs"]
mod projection_tests;

/// Build directly from the original retained compiler-profile AST, once.
///
/// Identifier, numeric/BigInt/string/boolean/null atoms, authored parentheses and
/// binary/logical, prefix unary, ordinary static/computed member, call,
/// conditional, array, static explicit data object and sequence nodes are
/// supported. Object key spelling/order stays original; computed, shorthand,
/// methods/accessors, spreads, unsupported keys/values and decoded `__proto__`
/// keys refuse without claiming full language early-error validation.
/// Sequences retain all
/// original children, checked commas and authored reference parentheses. Original array elements and
/// elisions retain their order and exact comma custody, including checked
/// authored trailing commas; spread elements refuse. Plain unary/member gaps use separators so signs, keyword
/// operators and numeric literal spellings cannot fuse with adjacent tokens.
/// Nonoptional calls retain original callees and argument order, checked
/// delimiters, commas and optional authored trailing commas. Type arguments,
/// spreads, optional chains, private fields and unsupported descendants refuse.
/// Original literal/operator spellings, parentheses, complete entities and
/// typed comments remain source slices.
/// Only proven plain ASCII whitespace gaps change. Encoded whitespace and
/// gaps containing comments stay verbatim. With an original decode map,
/// checked gaps containing physical LF also remain authored, retaining native
/// container framing. This does not select a language,
/// parse, clone/normalize an AST or call an OXC/legacy formatter.
///
/// The checked caller block must cover the selected embed inside the same
/// physical authored root. It establishes byte custody, not document/version
/// identity or whole-product admission. Unsupported descendants refuse the
/// complete document while leaving original observations available.
pub fn expression_document<'p, 'a>(
    original: &'p RetainedExpression<'a>,
    block: SourceBlock<'a>,
    allocator: &'a Allocator,
) -> Result<ExpressionDocument<'p, 'a>, ExpressionRefusal> {
    let document = root_document(Origin::Retained(original), block, allocator)?;
    Ok(ExpressionDocument { original, document })
}

pub(super) fn borrowed_document<'p, 'a>(
    original: &'p NativeExpressionView<'p, 'a>,
    block: SourceBlock<'a>,
    allocator: &'a Allocator,
) -> Result<Doc<'a>, ExpressionRefusal> {
    root_document(Origin::Borrowed(original), block, allocator)
}

fn root_document<'p, 'a>(
    original: Origin<'p, 'a>,
    block: SourceBlock<'a>,
    allocator: &'a Allocator,
) -> Result<Doc<'a>, ExpressionRefusal> {
    let admitted = original.expression()?;
    let mut context = match original {
        Origin::Retained(retained) => Context::new(retained, block, allocator)?,
        Origin::Borrowed(_) => Context::from_origin(original, block, allocator)?,
    };
    let entire = Span::new(0, original.source().text().len() as u32);
    let root = context.decoded_span(admitted.span())?;
    let mut parts = Vec::new_in(&allocator);
    parts.push(context.gap(Span::new(0, root.start), Gap::Empty)?);
    parts.push(context.node(admitted, entire, 0)?);
    parts.push(context.gap(Span::new(root.end, entire.end), Gap::Empty)?);
    context.finish()?;
    Ok(Doc::concat(parts))
}

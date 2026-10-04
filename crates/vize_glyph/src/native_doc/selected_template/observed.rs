//! Owned original embed observations from the same document traversal.

use std::vec::Vec as OwnedVec;
use vize_l0::{Allocator, Vec};
use vize_l1::markup::{
    NativeAttributeExpression, NativeInterpolationOperand, NativeTemplateComponent,
};

use super::super::template::Cursor;
use super::input::Observed;
use super::{
    Builder, Doc, NativeTemplateValuePolicy, TemplateRefusal, ValuePolicy, check_selected,
};

#[path = "observed/failure.rs"]
mod failure;
pub use failure::{ObservedNativeTemplateFailure, ObservedNativeTemplateRefusal};

/// Borrowed original selection, owned once-observed operands and complete block Doc.
/// This covers only the selected template body, not the enclosing SFC.
///
/// ```compile_fail
/// use vize_glyph::native_doc::{ObservedNativeTemplateDocument, observed_native_template_document};
/// use vize_l0::Allocator;
/// use vize_l1::markup::NativeTemplateComponent;
/// fn outlive_selected<'a>(selected: NativeTemplateComponent<'a>, arena: &'a Allocator)
///     -> ObservedNativeTemplateDocument<'a, 'a> {
///     observed_native_template_document(&selected, arena).unwrap()
/// }
/// ```
pub struct ObservedNativeTemplateDocument<'p, 'a> {
    original: &'p NativeTemplateComponent<'a>,
    operands: OwnedVec<NativeInterpolationOperand<'a>>,
    attributes: OwnedVec<NativeAttributeExpression<'a>>,
    document: Doc<'a>,
}

impl core::fmt::Debug for ObservedNativeTemplateDocument<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ObservedNativeTemplateDocument")
            .field("original", &self.original)
            .field("operand_count", &self.operands.len())
            .field("attribute_count", &self.attributes.len())
            .field("document", &self.document)
            .finish()
    }
}

impl<'p, 'a> ObservedNativeTemplateDocument<'p, 'a> {
    pub fn original(&self) -> &'p NativeTemplateComponent<'a> {
        self.original
    }
    pub fn operands(&self) -> &[NativeInterpolationOperand<'a>] {
        &self.operands
    }
    pub fn attribute_operands(&self) -> &[NativeAttributeExpression<'a>] {
        &self.attributes
    }
    pub(in crate::native_doc) fn into_observations(self) -> (Observed<'a>, Doc<'a>) {
        (
            Observed {
                operands: self.operands,
                failure: None,
                attributes: self.attributes,
                attribute_failure: None,
            },
            self.document,
        )
    }
    pub fn document(&self) -> &Doc<'a> {
        &self.document
    }
    /// Transfer all genuine observations with the same selection borrow and Doc.
    /// The independent owned vectors preserve their own original event order.
    pub fn into_full_parts(
        self,
    ) -> (
        &'p NativeTemplateComponent<'a>,
        OwnedVec<NativeInterpolationOperand<'a>>,
        OwnedVec<NativeAttributeExpression<'a>>,
        Doc<'a>,
    ) {
        (self.original, self.operands, self.attributes, self.document)
    }
    /// Preserve the original interpolation-only transfer contract.
    /// This releases conditional attribute observations; retain the whole owner
    /// or use `into_full_parts` when those observations must survive.
    /// The transferred Doc alone does not retain the selection borrow or grant
    /// enclosing-SFC admission; its text/composition borrow authored source/arena.
    pub fn into_parts(
        self,
    ) -> (
        &'p NativeTemplateComponent<'a>,
        OwnedVec<NativeInterpolationOperand<'a>>,
        Doc<'a>,
    ) {
        (self.original, self.operands, self.document)
    }
}

/// Observe and lay out each interpolation at the same original child visit.
///
/// This uses the existing selected-template Builder traversal and the existing
/// genuine Vue preparation/stock-expression parse once per original interpolation.
/// The new observation is parked before its exact-child admission and native
/// expression/source/framing layout. Native v-pre content remains text. There is
/// no preliminary operand/body pass, second parse, AST normalization or legacy
/// formatting route. Existing preobserved-list behavior remains independent.
///
/// A first refusal retains the original selection and every observation already
/// made, including the current successfully minted but rejected operand. If the
/// original observer itself returns a failure, that actual failure is retained
/// instead of fabricating an operand. Remaining unvisited bytes stay in the
/// original component; a failed prefix is never a complete document.
pub fn observed_native_template_document<'p, 'a>(
    original: &'p NativeTemplateComponent<'a>,
    allocator: &'a Allocator,
) -> Result<ObservedNativeTemplateDocument<'p, 'a>, ObservedNativeTemplateFailure<'p, 'a>> {
    observed_native_template_document_with_policy(
        original,
        allocator,
        NativeTemplateValuePolicy::PreserveOpaque,
    )
}

/// Observe through the same original traversal with an explicit value policy.
///
/// Strict policy refuses every complete typed directive having an original
/// value, after the existing value token's source/recovery check and before
/// visiting that element's body. Static attribute values stay authored. The
/// opaque/strict routes add no value parse or preliminary scan. The explicit
/// conditional policy uses each real L1 head once for name layout and its one
/// original value observation, parked before quote/admission/Doc checks. Other
/// valued directives still refuse. The failure keeps both observed prefixes.
pub fn observed_native_template_document_with_policy<'p, 'a>(
    original: &'p NativeTemplateComponent<'a>,
    allocator: &'a Allocator,
    policy: NativeTemplateValuePolicy,
) -> Result<ObservedNativeTemplateDocument<'p, 'a>, ObservedNativeTemplateFailure<'p, 'a>> {
    if let Err(refusal) = check_selected(original) {
        return Err(ObservedNativeTemplateFailure::new(
            original,
            Observed::new(),
            ObservedNativeTemplateRefusal::Document { index: 0, refusal },
        ));
    }
    let mut builder = Builder {
        selected: original,
        input: Observed::new(),
        next: 0,
        cursor: Cursor {
            source: original.component().block().source(),
            offset: 0,
        },
        allocator,
        values: ValuePolicy::new(original.component().block(), policy),
    };
    let mut parts = Vec::new_in(&allocator);
    let result = builder
        .children(original.children(), &mut parts, 0)
        .and_then(|()| {
            if builder.cursor.offset == builder.cursor.source.len() {
                Ok(())
            } else {
                Err(TemplateRefusal::SourceMismatch {
                    offset: builder.cursor.offset,
                }
                .into())
            }
        });
    let index = builder.next;
    let offset = builder.cursor.offset;
    let observations = builder.input;
    if let Err(refusal) = result {
        let refusal = if let Some((span, index, failure)) = &observations.attribute_failure {
            ObservedNativeTemplateRefusal::Attribute {
                span: *span,
                index: *index,
                kind: failure.kind(),
            }
        } else if let Some(failure) = &observations.failure {
            ObservedNativeTemplateRefusal::Interpolation {
                offset,
                index,
                kind: failure.kind(),
            }
        } else {
            ObservedNativeTemplateRefusal::Document { index, refusal }
        };
        return Err(ObservedNativeTemplateFailure::new(
            original,
            observations,
            refusal,
        ));
    }
    Ok(ObservedNativeTemplateDocument {
        original,
        operands: observations.operands,
        attributes: observations.attributes,
        document: Doc::concat(parts),
    })
}

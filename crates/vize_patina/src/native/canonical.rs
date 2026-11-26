//! One shared lint query over genuinely native canonical markup owners.
//!
//! This opt-in query does not parse source, lower a compatibility AST, or change
//! the configured product route. Constructors retain the actual completed
//! producer borrow; a detached Artifact/File cannot grant native admission.

use vize_l0::{Span, diag::MessageLookup};
use vize_l1_to_l2::native_file::NativeSfc;
use vize_l2::{file::L2Ir, lang::js::JsxMarkupL2Ir, op::Op, walk::NodeRef};

use super::{NativeLintFinding, tabindex_no_positive};

mod img_alt;

/// Whole-query refusals retain the owner's original observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeCanonicalLintRefusal {
    /// The retained canonical file does not match its admitted source owner.
    SourceMismatch,
    /// A script comment requires suppression policy unavailable to this query.
    ScriptComment {
        /// Original comment range.
        span: Span,
    },
    /// A markup comment requires suppression policy unavailable to this query.
    MarkupComment {
        /// Original comment range.
        span: Span,
    },
    /// An attribute cannot be classified from the producer's canonical facts.
    IndeterminateAttribute {
        /// Original attribute range.
        span: Span,
    },
    /// An image element uses syntax outside the canonical lint contract.
    UnsupportedImageElement {
        /// Original image-element range.
        span: Span,
    },
    /// Traversal exceeded the bounded canonical node budget.
    NodeLimit,
}

/// An actual native Vue/JSX owner's canonical File, never caller-joined parts.
///
/// ```compile_fail
/// use vize_l2::file::L2Ir;
/// use vize_patina::native::canonical::NativeCanonicalLint;
/// fn forge<'a>(l2ir: &L2Ir<'a>) {
///     let _ = NativeCanonicalLint { l2ir };
/// }
/// ```
pub struct NativeCanonicalLint<'o, 'a> {
    l2ir: &'o L2Ir<'a>,
}

impl<'o, 'a> NativeCanonicalLint<'o, 'a> {
    /// Borrow only an admitted original Descriptor/Program/template assembly.
    #[must_use]
    pub fn from_vue(owner: NativeSfc<'o, 'a>) -> Self {
        Self {
            l2ir: owner.file().l2ir(),
        }
    }

    /// Borrow only the checked same-Program static JSX canonical view.
    /// Original comments require the host's suppression authority and refuse.
    pub fn from_jsx(owner: &'o JsxMarkupL2Ir<'a>) -> Result<Self, NativeCanonicalLintRefusal> {
        if let Some(comment) = owner.owner().observation().comments().first() {
            let base = owner
                .l2ir()
                .units()
                .first()
                .ok_or(NativeCanonicalLintRefusal::SourceMismatch)?
                .span
                .start;
            let start = base
                .checked_add(comment.span.start)
                .ok_or(NativeCanonicalLintRefusal::SourceMismatch)?;
            let end = base
                .checked_add(comment.span.end)
                .ok_or(NativeCanonicalLintRefusal::SourceMismatch)?;
            return Err(NativeCanonicalLintRefusal::ScriptComment {
                span: Span::new(start, end),
            });
        }
        Ok(Self { l2ir: owner.l2ir() })
    }

    /// Returns the authentic canonical file retained from the admitted owner.
    #[must_use]
    pub fn l2ir(&self) -> &'o L2Ir<'a> {
        self.l2ir
    }

    /// Deprecated spelling of [`Self::l2ir`], borrowing the same retained IR.
    #[deprecated(note = "use `l2ir` instead")]
    #[must_use]
    pub fn file(&self) -> &'o L2Ir<'a> {
        self.l2ir()
    }

    /// Shared static attribute policy and complete localized L0 diagnostics.
    /// Dynamic binding expressions remain opaque; only producer-decoded static
    /// values participate. No source decoding, AST parse or fix is introduced.
    pub fn tabindex_no_positive(
        &self,
        messages: &impl MessageLookup,
    ) -> Result<Vec<NativeLintFinding>, NativeCanonicalLintRefusal> {
        vize_l0::profile!(
            "patina.native.canonical.tabindex",
            self.check_tabindex(messages)
        )
    }

    fn check_tabindex(
        &self,
        messages: &impl MessageLookup,
    ) -> Result<Vec<NativeLintFinding>, NativeCanonicalLintRefusal> {
        let mut findings = Vec::new();
        let mut refusal = None;
        let mut rendered = None;
        visit_markup_nodes(self.l2ir, &mut |_, node| {
            if refusal.is_some() {
                return;
            }
            let attributes = match node {
                NodeRef::Op(Op::Element(element)) => &element.attributes,
                NodeRef::Op(Op::Component(component)) => &component.attributes,
                NodeRef::Op(Op::Comment(comment)) => {
                    refusal =
                        Some(NativeCanonicalLintRefusal::MarkupComment { span: comment.span });
                    return;
                }
                _ => return,
            };
            for attribute in attributes {
                if attribute.name.eq_ignore_ascii_case("tabindex")
                    && let Some(value) = attribute.value
                    && crate::attribute_policy::positive_tabindex(value)
                {
                    let rendered = rendered
                        .get_or_insert_with(|| tabindex_no_positive::Messages::new(messages));
                    findings.push(rendered.finding(attribute.span));
                }
            }
        })
        .map_err(|_| NativeCanonicalLintRefusal::NodeLimit)?;
        if let Some(refusal) = refusal {
            return Err(refusal);
        }
        // Match the public linter's authored range order even when a native
        // structural normalization groups regions in another semantic order.
        findings.sort_unstable_by_key(|finding| {
            let span = finding.diagnostic().span;
            (span.start, span.end)
        });
        Ok(findings)
    }
}

#[cfg(feature = "canonical-rule-group")]
mod accessibility;
#[cfg(feature = "canonical-rule-group")]
pub use accessibility::AccessibilityFindings;

#[inline]
fn visit_markup_nodes<'s, 'a>(
    l2ir: &'s L2Ir<'a>,
    visit: &mut impl FnMut(vize_l0::id::NodeId, NodeRef<'s, 'a>),
) -> Result<(), vize_l2::walk::NodeLimit> {
    #[cfg(feature = "canonical-rule-group")]
    {
        l2ir.artifact().visit_markup_preorder(visit)
    }
    #[cfg(not(feature = "canonical-rule-group"))]
    {
        l2ir.artifact().visit_nodes(visit)
    }
}

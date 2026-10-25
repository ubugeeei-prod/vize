//! One shared lint query over genuinely native canonical markup owners.
//!
//! This opt-in query does not parse source, lower a compatibility AST, or change
//! the configured product route. Constructors retain the actual completed
//! producer borrow; a detached Artifact/File cannot grant native admission.

use vize_l0::{Span, diag::MessageLookup};
use vize_l1_to_l2::native_file::NativeSfc;
use vize_l2::{file::FileArtifact, lang::js::JsxMarkupFile, op::Op, walk::NodeRef};

use super::{NativeLintFinding, tabindex_no_positive};

/// Whole-query refusals retain the owner's original observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeCanonicalLintRefusal {
    SourceMismatch,
    ScriptComment { span: Span },
    MarkupComment { span: Span },
    NodeLimit,
}

/// An actual native Vue/JSX owner's canonical File, never caller-joined parts.
///
/// ```compile_fail
/// use vize_l2::file::FileArtifact;
/// use vize_patina::native::canonical::NativeCanonicalLint;
/// fn forge<'a>(file: &FileArtifact<'a>) {
///     let _ = NativeCanonicalLint { file };
/// }
/// ```
pub struct NativeCanonicalLint<'o, 'a> {
    file: &'o FileArtifact<'a>,
}

impl<'o, 'a> NativeCanonicalLint<'o, 'a> {
    /// Borrow only an admitted original Descriptor/Program/template assembly.
    #[must_use]
    pub fn from_vue(owner: NativeSfc<'o, 'a>) -> Self {
        Self {
            file: owner.file().file(),
        }
    }

    /// Borrow only the checked same-Program static JSX canonical view.
    /// Original comments require the host's suppression authority and refuse.
    pub fn from_jsx(owner: &'o JsxMarkupFile<'a>) -> Result<Self, NativeCanonicalLintRefusal> {
        if let Some(comment) = owner.owner().observation().comments().first() {
            let base = owner
                .file()
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
        Ok(Self { file: owner.file() })
    }

    #[must_use]
    pub fn file(&self) -> &'o FileArtifact<'a> {
        self.file
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
        self.file
            .artifact()
            .visit_nodes(&mut |_, node| {
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

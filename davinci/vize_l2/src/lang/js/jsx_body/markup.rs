//! Opt-in canonical markup from the same original Program's recorded syntax.
//!
//! No parser, AST walker, foreign File, or caller-provided node list participates.
//! This first static HTML family grants analysis authority, not JSX module output.

use super::{JsxL2Ir, JsxL2IrError, JsxL2IrProducer, RejectedJsxL2Ir};
use crate::ir::L2Ir;
use crate::template::L2TemplateError;
use alloc::{boxed::Box, vec::Vec};
use vize_l0::{Span, profile};

mod emit;
mod prepare;
mod whitespace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsxMarkupIssueKind {
    File(JsxL2IrError),
    Artifact(L2TemplateError),
    InvalidRecord,
    RootExpression,
    Fragment,
    Component,
    Namespace,
    SpreadAttribute,
    SpreadChild,
    DynamicAttribute,
    DynamicChild,
    SpecialAttribute,
    ValueNormalization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsxMarkupIssue {
    pub kind: JsxMarkupIssueKind,
    pub span: Span,
}

/// Actual native canonical markup and all original same-Program facts together.
///
/// ```compile_fail
/// use vize_l2::lang::js::{JsxL2Ir, JsxMarkupL2Ir};
/// fn forge(owner: JsxL2Ir<'_>) { let _ = JsxMarkupL2Ir { owner }; }
/// ```
pub struct JsxMarkupL2Ir<'a> {
    owner: JsxL2Ir<'a>,
}

impl<'a> JsxMarkupL2Ir<'a> {
    #[must_use]
    pub fn owner(&self) -> &JsxL2Ir<'a> {
        &self.owner
    }

    #[must_use]
    pub fn l2ir(&self) -> &L2Ir<'a> {
        self.owner.l2ir()
    }

    /// Original fragment delimiters in authored preorder, with the same owner,
    /// parent fragment, source, unit and scope. No boundary buffer is allocated.
    /// Canonical markup transparently flattens fragments; output backends retain
    /// these delimiters to realize their own fragment boundaries.
    pub fn fragment_boundaries(&self) -> impl Iterator<Item = super::JsxNode<'_, 'a>> {
        self.owner.nodes().filter(|node| {
            matches!(
                node.kind(),
                Some(
                    crate::resolution::SyntaxKind::FragmentOpening
                        | crate::resolution::SyntaxKind::FragmentClosing
                )
            )
        })
    }
}

/// A typed whole-view refusal retains the actual completed or rejected owner.
/// Unsupported syntax is checked before any canonical markup is written.
pub struct RejectedJsxMarkupL2Ir<'a> {
    owner: Result<JsxL2Ir<'a>, Box<RejectedJsxL2Ir<'a>>>,
    issues: Vec<JsxMarkupIssue>,
}

impl<'a> RejectedJsxMarkupL2Ir<'a> {
    #[must_use]
    pub fn owner(&self) -> Option<&JsxL2Ir<'a>> {
        self.owner.as_ref().ok()
    }

    #[must_use]
    pub fn rejected_owner(&self) -> Option<&RejectedJsxL2Ir<'a>> {
        self.owner.as_ref().err().map(Box::as_ref)
    }

    #[must_use]
    pub fn issues(&self) -> &[JsxMarkupIssue] {
        &self.issues
    }
}

impl<'a> JsxL2IrProducer<'a> {
    /// Seal the original-owning static HTML family into its canonical File.
    ///
    /// Dynamic expressions, component/foreign namespaces, directives, spreads,
    /// and values requiring JSX normalization refuse this whole view.
    /// Ordinary script expressions remain in their genuine original File facts.
    pub fn finish_markup(mut self) -> Result<JsxMarkupL2Ir<'a>, Box<RejectedJsxMarkupL2Ir<'a>>> {
        let span = self.block.span();
        let (roots, mut issues) = profile!(
            "l2.jsx_markup.prepare",
            prepare::roots(&self.recorder.records, self.block.root_source())
        );
        if issues.is_empty() {
            profile!("l2.jsx_markup.emit", {
                let mut region = self.producer.jsx_region();
                for root in roots {
                    if let Err(error) = emit::node(&self.recorder.records, root, &mut region) {
                        issues.push(JsxMarkupIssue {
                            kind: JsxMarkupIssueKind::Artifact(error),
                            span,
                        });
                        break;
                    }
                }
            });
        }
        let owner = profile!("l2.jsx_markup.finish_file", self.finish());
        if let Err(rejected) = &owner {
            issues.push(JsxMarkupIssue {
                kind: JsxMarkupIssueKind::File(rejected.error()),
                span,
            });
        }
        match owner {
            Ok(owner) if issues.is_empty() => Ok(JsxMarkupL2Ir { owner }),
            owner => Err(Box::new(RejectedJsxMarkupL2Ir { owner, issues })),
        }
    }
}

#[cfg(test)]
mod tests;

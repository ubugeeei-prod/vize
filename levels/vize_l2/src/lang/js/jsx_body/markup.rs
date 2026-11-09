//! Opt-in canonical markup from the same original Program's recorded syntax.
//!
//! No parser, AST walker, foreign File, or caller-provided node list participates.
//! This first static HTML family grants analysis authority, not JSX module output.

use super::{JsxFile, JsxFileError, JsxFileProducer, RejectedJsxFile};
use crate::artifact::ArtifactError;
use crate::file::FileArtifact;
use alloc::{boxed::Box, vec::Vec};
use vize_l0::{Span, profile};

mod emit;
mod prepare;
mod whitespace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsxMarkupIssueKind {
    File(JsxFileError),
    Artifact(ArtifactError),
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
/// use vize_l2::lang::js::{JsxFile, JsxMarkupFile};
/// fn forge(owner: JsxFile<'_>) { let _ = JsxMarkupFile { owner }; }
/// ```
pub struct JsxMarkupFile<'a> {
    owner: JsxFile<'a>,
}

impl<'a> JsxMarkupFile<'a> {
    #[must_use]
    pub fn owner(&self) -> &JsxFile<'a> {
        &self.owner
    }

    #[must_use]
    pub fn file(&self) -> &FileArtifact<'a> {
        self.owner.file()
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
pub struct RejectedJsxMarkupFile<'a> {
    owner: Result<JsxFile<'a>, Box<RejectedJsxFile<'a>>>,
    issues: Vec<JsxMarkupIssue>,
}

impl<'a> RejectedJsxMarkupFile<'a> {
    #[must_use]
    pub fn owner(&self) -> Option<&JsxFile<'a>> {
        self.owner.as_ref().ok()
    }

    #[must_use]
    pub fn rejected_owner(&self) -> Option<&RejectedJsxFile<'a>> {
        self.owner.as_ref().err().map(Box::as_ref)
    }

    #[must_use]
    pub fn issues(&self) -> &[JsxMarkupIssue] {
        &self.issues
    }
}

impl<'a> JsxFileProducer<'a> {
    /// Seal the original-owning static HTML family into its canonical File.
    ///
    /// Dynamic expressions, component/foreign namespaces, directives, spreads,
    /// and values requiring JSX normalization refuse this whole view.
    /// Ordinary script expressions remain in their genuine original File facts.
    pub fn finish_markup(mut self) -> Result<JsxMarkupFile<'a>, Box<RejectedJsxMarkupFile<'a>>> {
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
            Ok(owner) if issues.is_empty() => Ok(JsxMarkupFile { owner }),
            owner => Err(Box::new(RejectedJsxMarkupFile { owner, issues })),
        }
    }
}

#[cfg(test)]
mod tests;

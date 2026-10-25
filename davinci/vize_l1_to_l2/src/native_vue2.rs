//! Original Vue 2 callback owners lowered into the shared canonical op family.
//!
//! This first, explicit family preserves ordinary single-line HTML text, static
//! attributes and non-filter interpolations. Unsupported semantics remain typed
//! holes beside the complete original owner. There is no modern Component/File
//! conversion, compatibility parser, reparse, or product fallback.

use alloc::vec::Vec;
use vize_l0::{Allocator, Span, id::NodeId};
use vize_l1::dialect::vue2::surface::{ComponentExpressionPool, RetainedTextRefusal};
use vize_l2::artifact::{Artifact, ArtifactError, Builder, RejectedArtifact};

mod children;
mod element;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vue2HoleKind {
    OriginalSurface,
    Source,
    UnsupportedMarkup,
    UnsupportedTag,
    UnsupportedAttribute,
    AttributeEncoding,
    WhitespacePolicy,
    Filter,
    OriginalText(RetainedTextRefusal),
    Expression(crate::native::NativeExpressionError),
    Construction(ArtifactError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vue2Hole {
    pub span: Span,
    pub kind: Vue2HoleKind,
}

/// Keep the genuine dialect owner beside its checked fragments and refusals.
/// Only this producer can grant whole-family admission.
#[derive(Debug)]
pub struct NativeVue2Observation<'a> {
    original: ComponentExpressionPool<'a>,
    artifact: Option<Result<Artifact<'a>, RejectedArtifact<'a>>>,
    holes: Vec<Vue2Hole>,
}

impl<'a> NativeVue2Observation<'a> {
    pub fn original(&self) -> &ComponentExpressionPool<'a> {
        &self.original
    }
    pub fn artifact(&self) -> Option<&Artifact<'a>> {
        self.artifact.as_ref()?.as_ref().ok()
    }
    pub fn rejected(&self) -> Option<&RejectedArtifact<'a>> {
        self.artifact.as_ref()?.as_ref().err()
    }
    pub fn holes(&self) -> &[Vue2Hole] {
        &self.holes
    }
    pub fn admitted(&self) -> Option<&Artifact<'a>> {
        self.holes.is_empty().then(|| self.artifact()).flatten()
    }
}

/// Consume the actual once-parsed Vue 2 expression pool.
///
/// Successful expression roots and decode coordinates pass to L2 through the
/// shared native handoff. The original complete tree, every failed callback,
/// and all later children stay readable even after a refusal.
pub fn lower_vue2_component<'a>(
    allocator: &'a Allocator,
    original: ComponentExpressionPool<'a>,
) -> NativeVue2Observation<'a> {
    let source = original.block().root_source();
    let mut context = Context {
        allocator,
        original: &original,
        holes: Vec::new(),
    };
    if !original.errors().is_empty() || !original.unsupported().is_empty() {
        context.hole(original.block().span(), Vue2HoleKind::OriginalSurface);
    }
    let artifact = match Builder::new(allocator, source) {
        Ok(mut builder) => {
            context.children(&mut builder.region(), original.children());
            Some(builder.finish())
        }
        Err(error) => {
            context.hole(original.block().span(), Vue2HoleKind::Construction(error));
            None
        }
    };
    let holes = context.holes;
    NativeVue2Observation {
        original,
        artifact,
        holes,
    }
}

struct Context<'o, 'a> {
    allocator: &'a Allocator,
    original: &'o ComponentExpressionPool<'a>,
    holes: Vec<Vue2Hole>,
}

impl Context<'_, '_> {
    fn hole(&mut self, span: Span, kind: Vue2HoleKind) {
        self.holes.push(Vue2Hole { span, kind });
    }
    fn span(&self, raw: &str) -> Span {
        self.original
            .block()
            .span_of(raw)
            .unwrap_or(self.original.block().span())
    }
    fn built(&mut self, result: Result<NodeId, ArtifactError>, span: Span) {
        if let Err(error) = result {
            self.hole(span, Vue2HoleKind::Construction(error));
        }
    }
}

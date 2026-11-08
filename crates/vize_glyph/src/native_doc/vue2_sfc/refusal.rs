use vize_l0::{SourceFrameError, Span};
use vize_l1::dialect::vue2::{surface::TextRefusal, text::TextBoundaryKind};

use super::super::{TemplateRefusal, Vue2TextDocumentRefusal};

/// Non-element nodes that the initial native Vue 2 SFC family refuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeVue2SfcNode {
    /// A CDATA section appeared in the template surface tree.
    Cdata,
    /// A processing instruction appeared in the template surface tree.
    ProcessingInstruction,
}

/// Original observations remain on the normally owning descriptor.
/// Template refusal offsets are block-relative; component recovery offsets and
/// other spans are authored-file coordinates. Nested expression errors retain
/// their original coordinate domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeVue2SfcRefusal {
    /// The Vue 2 descriptor observation itself refused.
    Descriptor,
    /// The historical component parser recovered input.
    ComponentRecovery {
        /// Authored-file byte offset of the recovery or component block start.
        offset: u32,
    },
    /// A text boundary cannot be represented without changing authored bytes.
    ComponentBoundary {
        /// Authored-file span at the incompatible boundary.
        span: Span,
        /// Historical boundary category reported by the parser.
        kind: TextBoundaryKind,
    },
    /// The template contains an element outside the initial native family.
    UnsupportedElement {
        /// Authored-file span of the unsupported element.
        span: Span,
    },
    /// The template contains an attribute outside the initial native family.
    UnsupportedAttribute {
        /// Authored-file span of the unsupported attribute.
        span: Span,
    },
    /// The template contains a non-element node that is not supported.
    UnsupportedNode {
        /// Authored-file span of the unsupported node.
        span: Span,
        /// Unsupported node category.
        kind: NativeVue2SfcNode,
    },
    /// Plain template layout refused before interpolation handling.
    Template(TemplateRefusal),
    /// The historical Vue 2 text observer refused an interpolation.
    Text {
        /// Authored-file span of the text binding.
        span: Span,
        /// Original historical observer refusal.
        refusal: TextRefusal,
    },
    /// A retained Vue 2 text view could not produce its document.
    TextDoc(Vue2TextDocumentRefusal),
    /// A descriptor range could not be framed in the authored source.
    Frame {
        /// Authored-file span whose frame was invalid.
        span: Span,
        /// Original source-frame error.
        error: SourceFrameError,
    },
}

impl From<TemplateRefusal> for NativeVue2SfcRefusal {
    fn from(refusal: TemplateRefusal) -> Self {
        Self::Template(refusal)
    }
}

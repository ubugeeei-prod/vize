use vize_l0::{SourceFrameError, Span};
use vize_l1::dialect::vue2::{surface::TextRefusal, text::TextBoundaryKind};

use super::super::{TemplateRefusal, Vue2TextDocumentRefusal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeVue2SfcNode {
    Cdata,
    ProcessingInstruction,
}

/// Original observations remain on the normally owning descriptor.
/// Template offsets are block-relative; other spans are authored-file ranges.
/// Nested expression errors retain their original coordinate domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeVue2SfcRefusal {
    Descriptor,
    ComponentRecovery { offset: u32 },
    ComponentBoundary { span: Span, kind: TextBoundaryKind },
    UnsupportedElement { span: Span },
    UnsupportedAttribute { span: Span },
    UnsupportedNode { span: Span, kind: NativeVue2SfcNode },
    Template(TemplateRefusal),
    Text { span: Span, refusal: TextRefusal },
    TextDoc(Vue2TextDocumentRefusal),
    Frame { span: Span, error: SourceFrameError },
}

impl From<TemplateRefusal> for NativeVue2SfcRefusal {
    fn from(refusal: TemplateRefusal) -> Self {
        Self::Template(refusal)
    }
}

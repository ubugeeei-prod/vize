//! Backend admission is a projection, not a boolean promise about a generic S3
//! graph. Only this module can construct the payload consumed by `into_ir`.

mod emit;
pub(super) mod validate;

use vize_carton::Allocator;
use vize_s2_to_s3::Lowered;

use super::AdmissionFailure;

#[derive(Debug)]
pub(super) struct NativeArtifact<'a> {
    nodes: std::vec::Vec<Node<'a>>,
    root: usize,
}

#[derive(Debug)]
struct Node<'a> {
    content: Content<'a>,
    children: std::vec::Vec<usize>,
    bindings: std::vec::Vec<Binding<'a>>,
}

/// Authored `[start, end)` byte span of a payload value (P3-9 source maps).
type AuthoredSpan = (u32, u32);

#[derive(Debug)]
enum Content<'a> {
    Element {
        tag: &'a str,
        tag_span: AuthoredSpan,
        /// Name, literal value, and the authored span of that value.
        attributes: std::vec::Vec<(&'a str, Option<&'a str>, AuthoredSpan)>,
    },
    Text {
        parts: std::vec::Vec<TextPart<'a>>,
        dynamic: bool,
    },
    /// Authored branch order. Each branch renders exactly one native element.
    If { branches: std::vec::Vec<Branch<'a>> },
    /// One element-carried loop. `children` holds its single body element.
    For(Loop<'a>),
}

#[derive(Debug)]
struct Branch<'a> {
    /// `None` only for a trailing unconditional (`v-else`) branch.
    condition: Option<&'a str>,
    /// Authored span of the untrimmed condition value.
    span: AuthoredSpan,
    region: vize_s3::op::RegionId,
    root: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
struct Loop<'a> {
    source: &'a str,
    value: &'a str,
    key: Option<&'a str>,
    index: Option<&'a str>,
    /// The body element's `:key`, lifted out of its ordinary bindings.
    key_prop: Option<&'a str>,
    /// Authored spans of the untrimmed source, the value/key/index aliases,
    /// and the `:key` value (P3-9 source maps).
    spans: LoopSpans,
}

#[derive(Debug, Clone, Copy, Default)]
struct LoopSpans {
    source: AuthoredSpan,
    aliases: [Option<AuthoredSpan>; 3],
    key_prop: Option<AuthoredSpan>,
}

#[derive(Debug)]
struct TextPart<'a> {
    value: &'a str,
    dynamic: bool,
    span: AuthoredSpan,
}

#[derive(Debug)]
struct Binding<'a> {
    name: &'a str,
    value: &'a str,
    event: bool,
    modifiers: std::vec::Vec<&'a str>,
    /// Authored spans of the name and of the untrimmed value.
    spans: [AuthoredSpan; 2],
}

impl<'a> NativeArtifact<'a> {
    pub(super) fn admit(s3: &Lowered<'a>) -> Result<Self, AdmissionFailure> {
        validate::admit(&s3.program)
    }

    /// Consuming the checked projection is the only production generation path
    /// for an accepted artifact. No source parsing or AST lowering occurs here.
    /// With `spans`, payload slices borrowed from `source` keep their authored
    /// spans and the template and control-flow anchors are returned (Davinci
    /// P3-9).
    pub(super) fn into_ir_with_spans(
        self,
        allocator: &'a Allocator,
        source: &'a str,
        scope_id: Option<&str>,
        spans: bool,
    ) -> (
        crate::ir::RootIRNode<'a>,
        Option<crate::generate::spans::VaporSourceSpans>,
    ) {
        emit::emit(self, allocator, source, scope_id, spans)
    }
}

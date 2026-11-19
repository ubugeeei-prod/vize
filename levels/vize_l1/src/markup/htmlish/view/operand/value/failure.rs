use super::{AttributeOperandError, AttributeView};
use vize_l0::{SourceBlock, Span};

/// Normally owned preparation failure with the actual token's authored input.
/// The selected Component still owns its complete original surface/diagnostics.
/// These readonly byte observations confer no original-token admission.
#[derive(Debug)]
pub struct AttributeValueFailure<'a> {
    kind: AttributeOperandError,
    block: SourceBlock<'a>,
    ordinal: usize,
    name: &'a str,
    value: Option<&'a str>,
}
impl<'a> AttributeValueFailure<'a> {
    pub(super) fn original(kind: AttributeOperandError, attribute: &AttributeView<'_, 'a>) -> Self {
        Self {
            kind,
            block: attribute.component().block(),
            ordinal: attribute.ordinal(),
            name: attribute.surface().name.text,
            value: attribute
                .surface()
                .value
                .as_ref()
                .map(|value| value.content.text),
        }
    }
    #[must_use]
    pub const fn kind(&self) -> AttributeOperandError {
        self.kind
    }
    /// The actual attribute's full original block, including for foreign input.
    #[must_use]
    pub const fn block(&self) -> SourceBlock<'a> {
        self.block
    }
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    #[must_use]
    pub const fn raw_name(&self) -> &'a str {
        self.name
    }
    #[must_use]
    pub const fn raw_value(&self) -> Option<&'a str> {
        self.value
    }
    #[must_use]
    pub fn name_span(&self) -> Option<Span> {
        self.block.span_of(self.name)
    }
    #[must_use]
    pub fn value_span(&self) -> Option<Span> {
        self.value.and_then(|value| self.block.span_of(value))
    }
}

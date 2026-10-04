//! Original selected-block custody for the attribute visitor's explicit policy.

use vize_l0::SourceBlock;
use vize_l1::Token;

use super::super::template::AttributeValuePolicy;
use super::{NativeTemplateRefusal, TemplateRefusal};

/// Whether typed directive values may remain opaque authored source.
/// Neither policy parses values or grants directive-value semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTemplateValuePolicy {
    PreserveOpaque,
    RefuseValuedDirectives,
}

pub(super) struct ValuePolicy<'a> {
    block: SourceBlock<'a>,
    policy: NativeTemplateValuePolicy,
}

impl<'a> ValuePolicy<'a> {
    pub(super) fn new(block: SourceBlock<'a>, policy: NativeTemplateValuePolicy) -> Self {
        Self { block, policy }
    }
}

impl<'a> AttributeValuePolicy<'a> for ValuePolicy<'a> {
    type Refusal = NativeTemplateRefusal;

    fn value(
        &self,
        directive: bool,
        content: &Token<'a>,
        offset: usize,
    ) -> Result<(), Self::Refusal> {
        if !directive || self.policy == NativeTemplateValuePolicy::PreserveOpaque {
            return Ok(());
        }
        let mismatch = || TemplateRefusal::SourceMismatch { offset };
        let span = self.block.span_of(content.text).ok_or_else(mismatch)?;
        if !self.block.contains_block_span(span) {
            return Err(mismatch().into());
        }
        let authored = self
            .block
            .root_source()
            .get(span.start as usize..span.end as usize)
            .ok_or_else(mismatch)?;
        if !core::ptr::eq(authored, content.text) {
            return Err(mismatch().into());
        }
        Err(NativeTemplateRefusal::DirectiveValue { span })
    }
}

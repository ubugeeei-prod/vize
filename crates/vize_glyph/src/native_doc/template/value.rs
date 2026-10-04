//! A policy hook at the existing checked attribute-value content visit.

use vize_l1::Token;

use super::TemplateRefusal;

pub(in crate::native_doc) trait AttributeValuePolicy<'a> {
    type Refusal: From<TemplateRefusal>;

    fn value(
        &self,
        directive: bool,
        content: &Token<'a>,
        offset: usize,
    ) -> Result<(), Self::Refusal>;
}

pub(in crate::native_doc) struct PreserveOpaque;

impl<'a> AttributeValuePolicy<'a> for PreserveOpaque {
    type Refusal = TemplateRefusal;

    fn value(
        &self,
        _directive: bool,
        _content: &Token<'a>,
        _offset: usize,
    ) -> Result<(), Self::Refusal> {
        Ok(())
    }
}

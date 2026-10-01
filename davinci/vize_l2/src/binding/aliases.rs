//! Checked dense native positions; the builder owns lexical scope enumeration.

use super::JsBinding;
use vize_l0::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AliasError {
    MissingKey,
    DifferentBlock,
    InvalidOrder,
    InvalidContext,
}

/// Actual native formal roots in dense value/key/index order, without rest.
#[derive(Debug, Clone, Copy)]
pub struct NativeForAliases<'a> {
    value: &'a JsBinding<'a>,
    key: Option<&'a JsBinding<'a>>,
    index: Option<&'a JsBinding<'a>>,
}

impl<'a> NativeForAliases<'a> {
    pub fn checked(
        value: &'a JsBinding<'a>,
        key: Option<&'a JsBinding<'a>>,
        index: Option<&'a JsBinding<'a>>,
    ) -> Result<Self, AliasError> {
        if index.is_some() && key.is_none() {
            return Err(AliasError::MissingKey);
        }
        let aliases = Self { value, key, index };
        let mut previous = None;
        for binding in aliases.iter() {
            let parameter = binding.parameter();
            if !parameter.decorators.is_empty()
                || parameter.accessibility.is_some()
                || parameter.readonly
                || parameter.r#override
            {
                return Err(AliasError::InvalidContext);
            }
            if !value.same_alias_block(binding) {
                return Err(AliasError::DifferentBlock);
            }
            if previous.is_some_and(|span: Span| span.end > binding.span().start) {
                return Err(AliasError::InvalidOrder);
            }
            previous = Some(binding.span());
        }
        Ok(aliases)
    }

    pub fn iter(self) -> impl Iterator<Item = &'a JsBinding<'a>> {
        core::iter::once(self.value)
            .chain(self.key)
            .chain(self.index)
    }
}

const _: () = assert!(!core::mem::needs_drop::<NativeForAliases<'static>>());

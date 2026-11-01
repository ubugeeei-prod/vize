//! Exact original modifier tokens, bound to the issuing callback and attribute.

use super::NativeTemplateLintContext;
use crate::Severity;
use crate::diagnostic::{Fix, TextEdit};
use crate::native::{
    attribute,
    template::{NativeTemplateAttribute, NativeTemplateAttributeKind, NativeTemplateLintRefusal},
};
use vize_l0::Span;

/// A borrowed modifier of one actual checked event attribute. No raw-parts mint.
///
/// ```compile_fail
/// use vize_patina::native::template::NativeTemplateEventModifier;
/// let token = NativeTemplateEventModifier {};
/// ```
pub struct NativeTemplateEventModifier<'s, 'e, 'a> {
    attribute: &'s NativeTemplateAttribute<'e, 'a>,
    issuer: (usize, usize),
    range: Span,
    text: &'a str,
}

impl<'a> NativeTemplateEventModifier<'_, '_, 'a> {
    #[must_use]
    pub fn text(&self) -> &'a str {
        self.text
    }
}

/// Iterates the retained checked modifier range in authored order, once.
pub struct NativeTemplateEventModifiers<'s, 'e, 'a> {
    attribute: &'s NativeTemplateAttribute<'e, 'a>,
    issuer: (usize, usize),
    remaining: &'a str,
    offset: u32,
}

impl<'s, 'e, 'a> Iterator for NativeTemplateEventModifiers<'s, 'e, 'a> {
    type Item = Result<NativeTemplateEventModifier<'s, 'e, 'a>, NativeTemplateLintRefusal>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        let (text, rest) = self
            .remaining
            .split_once('.')
            .unwrap_or((self.remaining, ""));
        let start = self.offset;
        self.remaining = rest;
        let length = match u32::try_from(text.len()) {
            Ok(length) => length,
            Err(_) => return Some(Err(NativeTemplateLintRefusal::SourceMismatch)),
        };
        let Some(end) = start.checked_add(length) else {
            return Some(Err(NativeTemplateLintRefusal::SourceMismatch));
        };
        self.offset = if rest.is_empty() {
            end
        } else {
            match end.checked_add(1) {
                Some(offset) => offset,
                None => return Some(Err(NativeTemplateLintRefusal::SourceMismatch)),
            }
        };
        Some(Ok(NativeTemplateEventModifier {
            attribute: self.attribute,
            issuer: self.issuer,
            range: Span::new(start, end),
            text,
        }))
    }
}

impl<'a> NativeTemplateLintContext<'_, 'a> {
    /// Mint only from this owner's genuine admitted event header. Equal source,
    /// another callback or a sibling attribute is not a substitute token.
    pub fn event_modifiers<'s, 'e>(
        &self,
        attribute: &'s NativeTemplateAttribute<'e, 'a>,
    ) -> Result<NativeTemplateEventModifiers<'s, 'e, 'a>, NativeTemplateLintRefusal> {
        if !core::ptr::eq(attribute.original().component(), self.root.component()) {
            return Err(NativeTemplateLintRefusal::SourceMismatch);
        }
        if !matches!(attribute.kind(), NativeTemplateAttributeKind::On { .. }) {
            return Err(NativeTemplateLintRefusal::UnsupportedAttribute {
                span: attribute.range(),
            });
        }
        let head = attribute
            .directive()
            .ok_or(NativeTemplateLintRefusal::SourceMismatch)?;
        let text = attribute::project(self.root.component().block(), head.modifiers)?;
        let remaining = if text.is_empty() {
            ""
        } else {
            text.strip_prefix('.')
                .ok_or(NativeTemplateLintRefusal::SourceMismatch)?
        };
        let offset = head
            .modifiers
            .start
            .checked_add(u32::from(!text.is_empty()))
            .ok_or(NativeTemplateLintRefusal::SourceMismatch)?;
        Ok(NativeTemplateEventModifiers {
            attribute,
            issuer: self.callback_issuer(attribute.range())?,
            remaining,
            offset,
        })
    }

    /// Report and optionally replace only the exact issued modifier token.
    /// Neither a raw diagnostic range nor an arbitrary edit range is accepted.
    pub fn error_event_modifier_with_help(
        &mut self,
        modifier: &NativeTemplateEventModifier<'_, '_, 'a>,
        message_key: &str,
        variables: &[(&str, &str)],
        help_key: &str,
        fix: Option<(&str, &'static str)>,
    ) -> Result<(), NativeTemplateLintRefusal> {
        if !core::ptr::eq(
            modifier.attribute.original().component(),
            self.root.component(),
        ) || modifier.issuer != self.callback_issuer(modifier.attribute.range())?
        {
            return Err(NativeTemplateLintRefusal::SourceMismatch);
        }
        let head = modifier
            .attribute
            .directive()
            .ok_or(NativeTemplateLintRefusal::SourceMismatch)?;
        if modifier.range.start < head.modifiers.start
            || modifier.range.end > head.modifiers.end
            || modifier.range.is_empty()
            || attribute::project(self.root.component().block(), modifier.range)? != modifier.text
            || fix.is_some_and(|(_, replacement)| {
                replacement.is_empty()
                    || !replacement
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            })
        {
            return Err(NativeTemplateLintRefusal::SourceMismatch);
        }
        self.report_with_help(
            Severity::Error,
            modifier.range,
            message_key,
            variables,
            help_key,
        );
        if let Some((message, replacement)) = fix {
            let diagnostic = self
                .diagnostics
                .last_mut()
                .ok_or(NativeTemplateLintRefusal::SourceMismatch)?;
            diagnostic.fix = Some(Fix::new(
                message,
                TextEdit::replace(modifier.range.start, modifier.range.end, replacement),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

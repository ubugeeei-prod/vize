//! Once-retained static event syntax and its original complete header event.
//!
//! The movable owner retains no borrow of a selected/root wrapper. Its private
//! Element/Attribute addresses identify immutable nonempty arena backing, are
//! compared only, and are never dereferenced. Admission reborrows the actual
//! selected owner and current complete Attribute. This is not File/body custody.

use alloc::boxed::Box;
use oxc_parser::AdmittedHandlerBody;
use vize_l0::Span;

use super::super::{NativeAttribute, NativeTemplateComponent, NativeTemplateGrammar};
use super::NativeAttributeOperandError;
use crate::dialect::vue3::operand::static_event_head;
use crate::embed::syntax::{NativeSyntax, RetainedHandlerBody, parse_once};
use crate::embed::{Embed, Grammar, Lang, Shape, SourceError, prepare_attribute_value};

use super::origin::Origin;

/// Any unexpectedly shaped parser artifact remains normally owned and intact.
#[derive(Debug)]
pub struct NativeAttributeHandlerFailure<'a> {
    kind: NativeAttributeOperandError,
    syntax: Option<Box<NativeSyntax<'a>>>,
}
impl NativeAttributeHandlerFailure<'_> {
    #[must_use]
    pub const fn kind(&self) -> NativeAttributeOperandError {
        self.kind
    }
    #[must_use]
    pub fn syntax(&self) -> Option<&NativeSyntax<'_>> {
        self.syntax.as_deref()
    }
}
impl<'a> From<NativeAttributeOperandError> for NativeAttributeHandlerFailure<'a> {
    fn from(kind: NativeAttributeOperandError) -> Self {
        Self { kind, syntax: None }
    }
}

/// The original decoded value, one stock handler-body parse and private origin.
/// No caller AST, language, map, source range or allocator can assemble it.
///
/// ```compile_fail
/// use vize_l1::markup::NativeAttributeHandler;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeAttributeHandler<'static>>();
/// ```
pub struct NativeAttributeHandler<'a> {
    origin: Origin<'a>,
    argument: Span,
    syntax: RetainedHandlerBody<'a>,
}

impl<'a> NativeTemplateComponent<'a> {
    /// Replace the current header visit's source preparation and handler parse.
    /// This operation calls each once, deriving Module JS/TS from this selection.
    /// Handler syntax holes retain complete source/comments/diagnostics in the result.
    pub fn observe_attribute_handler(
        &self,
        attribute: NativeAttribute<'_, 'a>,
    ) -> Result<NativeAttributeHandler<'a>, NativeAttributeHandlerFailure<'a>> {
        Origin::check_original_header(self, &attribute)?;
        let block = self.component().block();
        let name_offset = block.offset_of(attribute.surface().name.text).ok_or(
            NativeAttributeOperandError::Source(SourceError::InvalidAuthoredSpan),
        )?;
        let argument = static_event_head(
            attribute.surface().name.text,
            name_offset,
            block.root_source(),
        )
        .map_err(NativeAttributeOperandError::Directive)?
        .ok_or(NativeAttributeOperandError::UnsupportedDirective)?;
        let origin = Origin::from_attribute(self, &attribute)?;
        let source = prepare_attribute_value(
            self.component().allocator(),
            origin.block.root_source(),
            origin.value_span,
        )
        .map_err(NativeAttributeOperandError::Source)?;
        let lang = match self.grammar() {
            NativeTemplateGrammar::JavaScriptModule => Lang::Js,
            NativeTemplateGrammar::TypeScriptModule => Lang::Ts,
        };
        let syntax = parse_once(
            self.component().allocator(),
            Embed {
                grammar: Grammar {
                    shape: Shape::HandlerBody,
                    lang,
                },
                source,
            },
        )
        .into_handler_body()
        .map_err(|syntax| NativeAttributeHandlerFailure {
            kind: NativeAttributeOperandError::UnexpectedShape,
            syntax: Some(syntax),
        })?;
        Ok(NativeAttributeHandler {
            origin,
            argument,
            syntax,
        })
    }
}

impl<'a> NativeAttributeHandler<'a> {
    #[must_use]
    pub const fn argument_span(&self) -> Span {
        self.argument
    }
    /// Complete authored event argument, without dialect prefix normalization.
    #[must_use]
    pub fn argument(&self) -> &'a str {
        self.argument.slice(self.origin.block.root_source())
    }
    /// Readonly retained observations, also available for local syntax holes.
    #[must_use]
    pub fn syntax(&self) -> &RetainedHandlerBody<'a> {
        &self.syntax
    }
    /// The original full value, before entity decoding and without quote bytes.
    #[must_use]
    pub const fn raw_value(&self) -> &'a str {
        self.origin.raw_value
    }
    #[must_use]
    pub const fn value_span(&self) -> Span {
        self.origin.value_span
    }
    #[must_use]
    pub const fn name_span(&self) -> Span {
        self.origin.name_span
    }
    /// Reborrow this movable owner at its actual selected header event.
    /// Foreign/sibling/copied parses cannot confer the original event's origin.
    pub fn admitted_for<'s>(
        &'s self,
        selected: &'s NativeTemplateComponent<'a>,
        attribute: NativeAttribute<'s, 'a>,
    ) -> Option<NativeAttributeHandlerView<'s, 'a>> {
        if !self.origin.matches(selected, &attribute) {
            return None;
        }
        self.syntax.admitted_body()?;
        Some(NativeAttributeHandlerView {
            owner: self,
            selected,
            attribute,
        })
    }
    /// Moving out raw retained syntax deliberately discards header authority.
    #[must_use]
    pub fn into_syntax(self) -> RetainedHandlerBody<'a> {
        self.syntax
    }
}

/// A short actual header/operand join, not a native body completion certificate.
pub struct NativeAttributeHandlerView<'s, 'a> {
    owner: &'s NativeAttributeHandler<'a>,
    selected: &'s NativeTemplateComponent<'a>,
    attribute: NativeAttribute<'s, 'a>,
}
impl<'s, 'a> NativeAttributeHandlerView<'s, 'a> {
    #[must_use]
    pub fn selected(&self) -> &'s NativeTemplateComponent<'a> {
        self.selected
    }
    #[must_use]
    pub fn attribute(&self) -> &NativeAttribute<'s, 'a> {
        &self.attribute
    }
    #[must_use]
    pub fn operand(&self) -> &'s NativeAttributeHandler<'a> {
        self.owner
    }
    /// This proof is always borrowed from the same private original stock owner.
    #[must_use]
    pub fn handler_body(&self) -> Option<AdmittedHandlerBody<'s, 'a>> {
        self.owner.syntax.admitted_body()
    }
}

#[cfg(test)]
mod tests;

//! Once-retained static event syntax and its original complete header event.
//!
//! The movable owner retains no borrow of a selected/root wrapper. Its private
//! Element/Attribute addresses identify immutable nonempty arena backing, are
//! compared only, and are never dereferenced. Admission reborrows the actual
//! selected owner and current complete Attribute. This is not File/body custody.

use alloc::boxed::Box;
use oxc_parser::AdmittedHandlerBody;
use vize_l0::Span;

use super::super::{AttributeView, TemplateComponent, TemplateGrammar};
use super::AttributeOperandError;
use crate::dialect::vue3::operand::static_event_head;
use crate::embed::syntax::{EmbedSyntax, RetainedHandlerBody, parse_once};
use crate::embed::{Embed, Grammar, Lang, Shape, SourceError, prepare_attribute_value};

use super::origin::Origin;

/// Any unexpectedly shaped parser artifact remains normally owned and intact.
#[derive(Debug)]
pub struct AttributeHandlerFailure<'a> {
    kind: AttributeOperandError,
    syntax: Option<Box<EmbedSyntax<'a>>>,
}
impl AttributeHandlerFailure<'_> {
    #[must_use]
    pub const fn kind(&self) -> AttributeOperandError {
        self.kind
    }
    #[must_use]
    pub fn syntax(&self) -> Option<&EmbedSyntax<'_>> {
        self.syntax.as_deref()
    }
}
impl<'a> From<AttributeOperandError> for AttributeHandlerFailure<'a> {
    fn from(kind: AttributeOperandError) -> Self {
        Self { kind, syntax: None }
    }
}

/// The original decoded value, one stock handler-body parse and private origin.
/// No caller AST, language, map, source range or allocator can assemble it.
///
/// ```compile_fail
/// use vize_l1::markup::AttributeHandler;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<AttributeHandler<'static>>();
/// ```
pub struct AttributeHandler<'a> {
    origin: Origin<'a>,
    argument: Span,
    syntax: RetainedHandlerBody<'a>,
}

impl<'a> TemplateComponent<'a> {
    /// Replace the current header visit's source preparation and handler parse.
    /// This operation calls each once, deriving Module JS/TS from this selection.
    /// Handler syntax holes retain complete source/comments/diagnostics in the result.
    pub fn observe_attribute_handler(
        &self,
        attribute: AttributeView<'_, 'a>,
    ) -> Result<AttributeHandler<'a>, AttributeHandlerFailure<'a>> {
        Origin::check_original_header(self, &attribute)?;
        let block = self.component().block();
        let name_offset =
            block
                .offset_of(attribute.surface().name.text)
                .ok_or(AttributeOperandError::Source(
                    SourceError::InvalidAuthoredSpan,
                ))?;
        let argument = static_event_head(
            attribute.surface().name.text,
            name_offset,
            block.root_source(),
        )
        .map_err(AttributeOperandError::Directive)?
        .ok_or(AttributeOperandError::UnsupportedDirective)?;
        let origin = Origin::from_attribute(self, &attribute)?;
        let source = prepare_attribute_value(
            self.component().allocator(),
            origin.block.root_source(),
            origin.value_span,
        )
        .map_err(AttributeOperandError::Source)?;
        let lang = match self.grammar() {
            TemplateGrammar::JavaScriptModule => Lang::Js,
            TemplateGrammar::TypeScriptModule => Lang::Ts,
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
        .map_err(|syntax| AttributeHandlerFailure {
            kind: AttributeOperandError::UnexpectedShape,
            syntax: Some(syntax),
        })?;
        Ok(AttributeHandler {
            origin,
            argument,
            syntax,
        })
    }
}

impl<'a> AttributeHandler<'a> {
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
        selected: &'s TemplateComponent<'a>,
        attribute: AttributeView<'s, 'a>,
    ) -> Option<AttributeHandlerView<'s, 'a>> {
        if !self.origin.matches(selected, &attribute) {
            return None;
        }
        self.syntax.admitted_body()?;
        Some(AttributeHandlerView {
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
pub struct AttributeHandlerView<'s, 'a> {
    owner: &'s AttributeHandler<'a>,
    selected: &'s TemplateComponent<'a>,
    attribute: AttributeView<'s, 'a>,
}
impl<'s, 'a> AttributeHandlerView<'s, 'a> {
    #[must_use]
    pub fn selected(&self) -> &'s TemplateComponent<'a> {
        self.selected
    }
    #[must_use]
    pub fn attribute(&self) -> &AttributeView<'s, 'a> {
        &self.attribute
    }
    #[must_use]
    pub fn operand(&self) -> &'s AttributeHandler<'a> {
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

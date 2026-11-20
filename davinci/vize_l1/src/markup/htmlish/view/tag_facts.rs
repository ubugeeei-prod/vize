//! Tag facts of the default Vue 3 grammar, retained by original complete-header parsing.
//! This fact is not DOM membership, runtime component resolution or File admission.

use vize_l0::Span;

use super::{ElementView, OwnedComponent};
use crate::markup::{Element, htmlish::surface::TagKindFact};

/// The original default Vue 3 tag category; no runtime tag resolution is implied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagFactsKind {
    Element,
    Component,
    Slot,
    Template,
}

/// No kind is fabricated when a constructor or lexical policy cannot prove it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagFactsRefusal {
    Unavailable,
    AmbiguousVerbatim,
    InheritedTemplate,
    IncompleteHeader,
    SourceMismatch,
}

/// Readonly classification of the exact original element and original owner.
/// No raw tag, category, element, source or profile can be supplied by a caller.
///
/// ```compile_fail
/// use vize_l1::markup::TagFacts;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<TagFacts<'static, 'static>>();
/// ```
pub struct TagFacts<'o, 'a> {
    component: &'o OwnedComponent<'a>,
    element: &'o Element<'a>,
    kind: TagFactsKind,
    span: Span,
    header_literal: bool,
    table_context: bool,
    recovery_context: bool,
}

impl core::fmt::Debug for TagFacts<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("TagFacts")
            .field("kind", &self.kind)
            .field("span", &self.span)
            .field("header_literal", &self.header_literal)
            .field("table_context", &self.table_context)
            .field("recovery_context", &self.recovery_context)
            .finish_non_exhaustive()
    }
}

impl<'o, 'a> TagFacts<'o, 'a> {
    #[must_use]
    pub fn component(&self) -> &'o OwnedComponent<'a> {
        self.component
    }
    #[must_use]
    pub fn element(&self) -> &'o Element<'a> {
        self.element
    }
    #[must_use]
    pub fn kind(&self) -> TagFactsKind {
        self.kind
    }
    /// Absolute original tag-name range, without the opening `<`.
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }

    /// Whether inherited lexical pre made this original header literal before
    /// its own attributes were lexed. Own `v-pre` freezes only after that header.
    #[must_use]
    pub fn header_is_literal(&self) -> bool {
        self.header_literal
    }

    /// Whether this original opening inherited an authored `table` ancestor
    /// from the live parser stack, after its actual structural recovery.
    /// The name is ASCII-insensitive; this conservative source fact does not
    /// establish DOM namespace, foster parenting or legacy parser semantics.
    #[must_use]
    pub fn in_table_context(&self) -> bool {
        self.table_context
    }

    /// Original authored ancestry whose optional-closing, ignored-start or
    /// formatting behavior is outside this selected header provider's proof.
    /// This conservative source fact neither resolves legacy tree recovery nor
    /// substitutes a legacy inherited pre mode for the actual original mode.
    #[must_use]
    pub fn in_recovery_context(&self) -> bool {
        self.recovery_context
    }
}

impl<'o, 'a> ElementView<'o, 'a> {
    pub fn tag_facts(&self) -> Result<TagFacts<'o, 'a>, TagFactsRefusal> {
        let element = self.surface();
        if element.open.lt_name.is_missing() || element.open.gt.is_missing() {
            return Err(TagFactsRefusal::IncompleteHeader);
        }
        let kind = match element.open.tag_facts() {
            Some(TagKindFact::Element) => TagFactsKind::Element,
            Some(TagKindFact::Component) => TagFactsKind::Component,
            Some(TagKindFact::Slot) => TagFactsKind::Slot,
            Some(TagKindFact::Template) => TagFactsKind::Template,
            Some(TagKindFact::AmbiguousVerbatim) => {
                return Err(TagFactsRefusal::AmbiguousVerbatim);
            }
            Some(TagKindFact::InheritedTemplate) => {
                return Err(TagFactsRefusal::InheritedTemplate);
            }
            None => return Err(TagFactsRefusal::Unavailable),
        };
        let block = self.component().block();
        let span = block
            .span_of(element.tag())
            .filter(|span| block.contains_block_span(*span))
            .ok_or(TagFactsRefusal::SourceMismatch)?;
        Ok(TagFacts {
            component: self.component(),
            element,
            kind,
            span,
            header_literal: element.open.header_is_literal(),
            table_context: element.open.in_table_context(),
            recovery_context: element.open.in_recovery_context(),
        })
    }
}

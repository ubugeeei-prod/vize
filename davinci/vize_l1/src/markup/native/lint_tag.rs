//! Default Vue 3 lint tag grammar, retained by original complete-header parsing.
//! This fact is not DOM membership, runtime component resolution or File admission.

use vize_l0::Span;

use super::{NativeComponent, NativeElement};
use crate::{Element, surface::LintTagFact};

/// Original default Vue 3 lint category; no runtime tag resolution is implied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeLintTagKind {
    Element,
    Component,
    Slot,
    Template,
}

/// No kind is fabricated when a constructor or lexical policy cannot prove it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeLintTagRefusal {
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
/// use vize_l1::markup::NativeLintTag;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeLintTag<'static, 'static>>();
/// ```
pub struct NativeLintTag<'o, 'a> {
    component: &'o NativeComponent<'a>,
    element: &'o Element<'a>,
    kind: NativeLintTagKind,
    span: Span,
    header_literal: bool,
}

impl core::fmt::Debug for NativeLintTag<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("NativeLintTag")
            .field("kind", &self.kind)
            .field("span", &self.span)
            .field("header_literal", &self.header_literal)
            .finish_non_exhaustive()
    }
}

impl<'o, 'a> NativeLintTag<'o, 'a> {
    #[must_use]
    pub fn component(&self) -> &'o NativeComponent<'a> {
        self.component
    }
    #[must_use]
    pub fn element(&self) -> &'o Element<'a> {
        self.element
    }
    #[must_use]
    pub fn kind(&self) -> NativeLintTagKind {
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
}

impl<'o, 'a> NativeElement<'o, 'a> {
    pub fn lint_tag(&self) -> Result<NativeLintTag<'o, 'a>, NativeLintTagRefusal> {
        let element = self.surface();
        if element.open.lt_name.is_missing() || element.open.gt.is_missing() {
            return Err(NativeLintTagRefusal::IncompleteHeader);
        }
        let kind = match element.open.lint_tag() {
            Some(LintTagFact::Element) => NativeLintTagKind::Element,
            Some(LintTagFact::Component) => NativeLintTagKind::Component,
            Some(LintTagFact::Slot) => NativeLintTagKind::Slot,
            Some(LintTagFact::Template) => NativeLintTagKind::Template,
            Some(LintTagFact::AmbiguousVerbatim) => {
                return Err(NativeLintTagRefusal::AmbiguousVerbatim);
            }
            Some(LintTagFact::InheritedTemplate) => {
                return Err(NativeLintTagRefusal::InheritedTemplate);
            }
            None => return Err(NativeLintTagRefusal::Unavailable),
        };
        let block = self.component().block();
        let span = block
            .span_of(element.tag())
            .filter(|span| block.contains_block_span(*span))
            .ok_or(NativeLintTagRefusal::SourceMismatch)?;
        Ok(NativeLintTag {
            component: self.component(),
            element,
            kind,
            span,
            header_literal: element.open.lint_header_is_literal(),
        })
    }
}

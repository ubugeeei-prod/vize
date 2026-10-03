//! Authored tag names borrowed from one original Element CST projection.

use super::{NativeComponent, NativeElement, Parent};
use crate::{Element, ElementClose, SurfaceChild, Token};
use vize_l0::{SourceBlock, Span};

/// The original construction outcome, without an invented closing name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeElementClosingName {
    Present(Span),
    Missing,
    Implicit,
    SelfClosing,
    Void,
}

/// No source or completed grammar is inferred from recovered syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeElementNameRefusal {
    Recovered { offset: u32 },
    UnsupportedComponent,
    IncompleteOpening,
    IncompleteClosing,
    SourceMismatch,
}

/// Exact original Component, Element and direct-child identity remain borrowed.
/// Spans are authored UTF-8 bytes and exclude tag delimiters and gap bytes.
/// This is lexical CST custody, not namespace, runtime or File admission.
///
/// Raw elements and source coordinates cannot mint this receipt:
/// ```compile_fail
/// use vize_l1::markup::NativeElementNames;
/// fn forge() { let _ = NativeElementNames { opening: Default::default() }; }
/// ```
/// The actual original owner must remain alive:
/// ```compile_fail
/// use vize_l0::{Allocator, SourceRoot};
/// use vize_l1::markup::NativeComponent;
/// let arena = Allocator::default();
/// let owner = NativeComponent::parse_in(&arena, SourceRoot::new("<p></p>").unwrap().whole_block()).unwrap();
/// let element = owner.children().next().unwrap().into_element().unwrap();
/// let names = element.names().unwrap();
/// drop(owner);
/// let _ = names.opening();
/// ```
pub struct NativeElementNames<'o, 'a> {
    component: &'o NativeComponent<'a>,
    element: &'o Element<'a>,
    parent: Parent<'o, 'a>,
    ordinal: usize,
    opening: Span,
    closing: NativeElementClosingName,
}

impl core::fmt::Debug for NativeElementNames<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("NativeElementNames")
            .field("opening", &self.opening)
            .field("closing", &self.closing)
            .finish_non_exhaustive()
    }
}

impl<'o, 'a> NativeElementNames<'o, 'a> {
    #[must_use]
    pub fn component(&self) -> &'o NativeComponent<'a> {
        self.component
    }
    #[must_use]
    pub fn element(&self) -> &'o Element<'a> {
        self.element
    }
    #[must_use]
    pub fn block(&self) -> SourceBlock<'a> {
        self.component.block()
    }
    #[must_use]
    pub const fn opening(&self) -> Span {
        self.opening
    }
    #[must_use]
    pub const fn closing(&self) -> NativeElementClosingName {
        self.closing
    }
    /// Rejoin the same actual element; equal source buffers/ordinals do not suffice.
    #[must_use]
    pub fn accepts(&self, element: &NativeElement<'_, 'a>) -> bool {
        core::ptr::eq(self.component, element.component)
            && core::ptr::eq(self.element, element.element)
            && self.ordinal == element.ordinal
            && match (self.parent, element.parent) {
                (Parent::Root, Parent::Root) => true,
                (Parent::Element(left), Parent::Element(right)) => core::ptr::eq(left, right),
                _ => false,
            }
    }
}

impl<'o, 'a> NativeElement<'o, 'a> {
    /// Project the original CST at this existing child visit, without a walk,
    /// parser, source scan, allocation or stored per-node name record.
    ///
    /// Only `ElementClose::Present` grants a second name. Its original builder
    /// matched the authored end-tag event; its name is the token suffix of the
    /// opening name's byte length. Existing ASCII-insensitive matching preserves
    /// that length, including non-ASCII bytes. Delimiter/gap bytes are excluded
    /// without rescanning them or changing the original matching policy.
    /// Missing/implicit/void/self-closing outcomes remain distinct. Every
    /// original spelling survives; no namespace or dialect case fold is added.
    pub fn names(&self) -> Result<NativeElementNames<'o, 'a>, NativeElementNameRefusal> {
        let component = self.component;
        let block = component.block();
        let carrier = component.carrier();
        if let Some(error) = carrier.errors.first() {
            return Err(NativeElementNameRefusal::Recovered {
                offset: block
                    .start()
                    .checked_add(error.offset)
                    .ok_or(NativeElementNameRefusal::SourceMismatch)?,
            });
        }
        if carrier.authored.is_some() || !carrier.unsupported.is_empty() {
            return Err(NativeElementNameRefusal::UnsupportedComponent);
        }
        let original = match self.parent {
            Parent::Root => component.root_child_at(self.ordinal),
            Parent::Element(parent) => parent.children.get(self.ordinal),
        };
        if !core::ptr::eq(carrier.tree.source, block.source())
            || !matches!(original, Some(SurfaceChild::Element(element))
                if core::ptr::eq(element.as_ref(), self.element))
        {
            return Err(NativeElementNameRefusal::SourceMismatch);
        }
        let open = &self.element.open;
        if open.lt_name.is_missing() || open.gt.is_missing() {
            return Err(NativeElementNameRefusal::IncompleteOpening);
        }
        let opening_token = token_span(block, &open.lt_name)?;
        let opening = open
            .lt_name
            .text
            .strip_prefix('<')
            .filter(|name| !name.is_empty())
            .ok_or(NativeElementNameRefusal::SourceMismatch)?;
        let opening = block
            .span_of(opening)
            .filter(|span| block.contains_block_span(*span))
            .ok_or(NativeElementNameRefusal::SourceMismatch)?;
        let open_gt = token_span(block, &open.gt)?;
        if opening.start != opening_token.start + 1
            || open.gt.text != ">"
            || opening.end > open_gt.start
        {
            return Err(NativeElementNameRefusal::SourceMismatch);
        }
        let closing = match &self.element.close {
            ElementClose::Missing => NativeElementClosingName::Missing,
            ElementClose::Implicit => NativeElementClosingName::Implicit,
            ElementClose::NotExpected if open.slash.is_some() => {
                let slash = open
                    .slash
                    .as_ref()
                    .ok_or(NativeElementNameRefusal::SourceMismatch)?;
                let span = token_span(block, slash)?;
                if slash.is_missing() || slash.text != "/" || span.end > open_gt.start {
                    return Err(NativeElementNameRefusal::IncompleteOpening);
                }
                NativeElementClosingName::SelfClosing
            }
            ElementClose::NotExpected => NativeElementClosingName::Void,
            ElementClose::Present(close) => {
                if close.lt_slash_name.is_missing() || close.gt.is_missing() {
                    return Err(NativeElementNameRefusal::IncompleteClosing);
                }
                let token = token_span(block, &close.lt_slash_name)?;
                let gt = token_span(block, &close.gt)?;
                let start = close
                    .lt_slash_name
                    .text
                    .len()
                    .checked_sub((opening.end - opening.start) as usize)
                    .filter(|start| *start >= 2)
                    .ok_or(NativeElementNameRefusal::SourceMismatch)?;
                let name = close
                    .lt_slash_name
                    .text
                    .get(start..)
                    .ok_or(NativeElementNameRefusal::SourceMismatch)?;
                let span = block
                    .span_of(name)
                    .filter(|span| block.contains_block_span(*span))
                    .ok_or(NativeElementNameRefusal::SourceMismatch)?;
                if !close.lt_slash_name.text.starts_with("</")
                    || close.gt.text != ">"
                    || span.end != token.end
                    || token.start < open_gt.end
                    || span.end > gt.start
                {
                    return Err(NativeElementNameRefusal::SourceMismatch);
                }
                NativeElementClosingName::Present(span)
            }
        };
        Ok(NativeElementNames {
            component,
            element: self.element,
            parent: self.parent,
            ordinal: self.ordinal,
            opening,
            closing,
        })
    }
}

fn token_span(block: SourceBlock<'_>, token: &Token<'_>) -> Result<Span, NativeElementNameRefusal> {
    block
        .span_of(token.text)
        .filter(|span| block.contains_block_span(*span))
        .ok_or(NativeElementNameRefusal::SourceMismatch)
}

#[cfg(test)]
mod tests;
